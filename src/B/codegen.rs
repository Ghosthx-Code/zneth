use crate::F::ast::{DataType, Expression, Function, Program, stmt};
use inkwell::{
    AddressSpace, OptimizationLevel,
    builder::Builder,
    context::Context,
    module::Module,
    targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine},
    types::{BasicType, BasicTypeEnum},
    values::{AnyValue, BasicValueEnum, PointerValue},
};
use std::{cell::RefCell, collections::HashMap, path::Path};

pub struct CodeGen<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub vars: RefCell<
        HashMap<
            String,
            (
                PointerValue<'ctx>,
                BasicTypeEnum<'ctx>,
                BasicValueEnum<'ctx>,
                bool,
                usize,
            ),
        >,
    >,
    pub tm: TargetMachine,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(
        context: &'ctx Context,
        module_name: &'ctx str,
        opt_level: OptimizationLevel,
    ) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        Target::initialize_native(&InitializationConfig::default())
            .expect("Failed To Init Native LLVM target");

        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple).unwrap();
        let cpu = TargetMachine::get_host_cpu_name();
        let featurs = TargetMachine::get_host_cpu_features();

        let target_machine = target
            .create_target_machine(
                &target_triple,
                &cpu.to_string_lossy(),
                &featurs.to_string_lossy(),
                opt_level,
                RelocMode::PIC,
                CodeModel::Default,
            )
            .expect("Failed To Create Target Machine");

        let target_data = target_machine.get_target_data();
        module.set_data_layout(&target_data.get_data_layout());
        module.set_triple(&target_triple);

        CodeGen {
            context,
            module,
            builder,
            vars: RefCell::new(HashMap::new()),
            tm: target_machine,
        }
    }

    pub fn emit_binary(&mut self, output: &str) {
        let path = Path::new(output);
        if !self.module.write_bitcode_to_path(&path) {
            std::process::exit(0);
        }
    }

    pub fn compile_program(&mut self, program: Program) -> String {
        // Compile into the internal LLVM module and return the textual IR.
        self.compile_program_into_module(program);
        self.module.print_to_string().to_string()
    }

    /// Compile into the internal LLVM module without converting to text.
    /// This enables callers to emit bitcode via `emit_binary`.
    pub fn compile_program_into_module(&mut self, program: Program) {
        self.vars.borrow_mut().clear();

        // IMPORTANT: avoid holding borrows of `program` while mutating `self.module`
        // (Module<'ctx> is invariant wrt 'ctx).
        let func_headers: Vec<(String, DataType)> = program
            .functions
            .iter()
            .map(|f| (f.name.clone(), f.return_type))
            .collect();

        // Declare functions first.
        for (name, ret_ty) in &func_headers {
            if self.module.get_function(name).is_none() {
                let return_type = self.map_type(ret_ty);
                let fn_type = return_type.fn_type(&[], false);
                self.module.add_function(name, fn_type, None);
            }
        }

        // Compile functions.
        for func in program.functions {
            self.compile_function(&func);
        }

        // Run optimization passes so unused allocas/heap allocations can be eliminated
        // when LLVM can prove they are unused.
        self.run_optimizations();
    }

    fn compile_function(&mut self, func: &Function) {
        let llvm_func = self
            .module
            .get_function(&func.name)
            .unwrap_or_else(|| panic!("Undefined function '{}'", func.name));

        let entry_block = self.context.append_basic_block(llvm_func, "entry");
        self.builder.position_at_end(entry_block);

        match &func.body {
            Some(stmt::Block(sts)) => {
                for s in sts {
                    self.stmt(s);
                    if matches!(s, stmt::Ret { .. }) {
                        break;
                    }
                }
            }
            Some(other) => self.stmt(other),
            None => {}
        }

        if entry_block.get_terminator().is_none() {
            self.builder.build_return(None).unwrap();
        }
    }

    fn map_type(&self, ty: &DataType) -> BasicTypeEnum<'ctx> {
        match ty {
            DataType::I1 => self.context.bool_type().into(),
            DataType::I8 => self.context.i8_type().into(),
            DataType::I32 => self.context.i32_type().into(),
            DataType::I64 => self.context.i64_type().into(),
            DataType::I128 => self.context.i128_type().into(),
            DataType::F32 => self.context.f32_type().into(),
            DataType::F64 => self.context.f64_type().into(),
            DataType::Str => self.context.ptr_type(AddressSpace::default()).into(),
        }
    }

    fn llvm_type_from_ast(&self, ast_type: &DataType) -> BasicTypeEnum<'ctx> {
        self.map_type(ast_type)
    }

    fn declare_printf(&self) -> inkwell::values::FunctionValue<'ctx> {
        let i32type = self.context.i32_type();
        let i8ptr_ty = self
            .module
            .get_context()
            .i8_type()
            .ptr_type(AddressSpace::default());
        let printftype = i32type.fn_type(&[i8ptr_ty.into()], true);
        let printf_fn = self.module.add_function("printf", printftype, None);
        let nocapture_id = inkwell::attributes::Attribute::get_named_enum_kind_id("nocapture");
        let nocapture_attr = self.context.create_enum_attribute(nocapture_id, 0);
        printf_fn.add_attribute(inkwell::attributes::AttributeLoc::Param(0), nocapture_attr);
        let readonly_id = inkwell::attributes::Attribute::get_named_enum_kind_id("readonly");
        let readonly_attr = self.context.create_enum_attribute(readonly_id, 0);
        printf_fn.add_attribute(inkwell::attributes::AttributeLoc::Param(0), readonly_attr);
        let wind_id = inkwell::attributes::Attribute::get_named_enum_kind_id("nounwind");
        let wind_attr = self.context.create_enum_attribute(wind_id, 0);
        printf_fn.add_attribute(inkwell::attributes::AttributeLoc::Function, wind_attr);
        let free_id = inkwell::attributes::Attribute::get_named_enum_kind_id("nofree");
        let free_attr = self.context.create_enum_attribute(free_id, 0);
        printf_fn.add_attribute(inkwell::attributes::AttributeLoc::Function, free_attr);
        let willreturn_id = inkwell::attributes::Attribute::get_named_enum_kind_id("willreturn");
        let willret_attr = self.context.create_enum_attribute(willreturn_id, 0);
        printf_fn.add_attribute(inkwell::attributes::AttributeLoc::Function, willret_attr);
        let no_id = inkwell::attributes::Attribute::get_named_enum_kind_id("nosync");
        let no_attr = self.context.create_enum_attribute(no_id, 0);
        printf_fn.add_attribute(inkwell::attributes::AttributeLoc::Function, no_attr);
        let a_id = inkwell::attributes::Attribute::get_named_enum_kind_id("argmemonly");
        let a_attr = self.context.create_enum_attribute(a_id, 0);
        printf_fn.add_attribute(inkwell::attributes::AttributeLoc::Function, a_attr);
        let n_id = inkwell::attributes::Attribute::get_named_enum_kind_id("nosync");
        let n_attr = self.context.create_enum_attribute(n_id, 0);
        printf_fn.add_attribute(inkwell::attributes::AttributeLoc::Function, n_attr);
        printf_fn
    }

    fn register_malloc(&self) -> inkwell::values::FunctionValue<'ctx> {
        let size_type = self.context.i64_type();
        let ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let malloc_type = ptr_type.fn_type(&[size_type.into()], false);
        let f = self.module.add_function("malloc", malloc_type, None);
        let wind_id = inkwell::attributes::Attribute::get_named_enum_kind_id("nounwind");
        let wind_attr = self.context.create_enum_attribute(wind_id, 0);
        f.add_attribute(inkwell::attributes::AttributeLoc::Function, wind_attr);
        let willreturn_id = inkwell::attributes::Attribute::get_named_enum_kind_id("willreturn");
        let willret_attr = self.context.create_enum_attribute(willreturn_id, 0);
        f.add_attribute(inkwell::attributes::AttributeLoc::Function, willret_attr);
        let n_id = inkwell::attributes::Attribute::get_named_enum_kind_id("nosync");
        let n_attr = self.context.create_enum_attribute(n_id, 0);
        f.add_attribute(inkwell::attributes::AttributeLoc::Function, n_attr);
        let an_id = inkwell::attributes::Attribute::get_named_enum_kind_id("noalias");
        let an_attr = self.context.create_enum_attribute(an_id, 0);
        f.add_attribute(inkwell::attributes::AttributeLoc::Function, an_attr);
        f
    }

    fn stmt(&mut self, stmt1: &stmt) {
        match stmt1 {
            stmt::Block(sts) => {
                for s in sts {
                    self.stmt(s);
                }
            }
            stmt::Printf { value, .. } => {
                let llvm_value = self.expr(value, None);
                let printf_fn = self.declare_printf();

                // Choose printf format based on the LLVM value we’re emitting.
                // This makes it work for i/f/p produced by literals and identifiers.
                let fmt_str = match llvm_value {
                    BasicValueEnum::IntValue(_) => "%lld",
                    BasicValueEnum::FloatValue(_) => "%f",
                    BasicValueEnum::PointerValue(_) => "%s",
                    _ => "%lld",
                };

                let fmt_ptr = self
                    .builder
                    .build_global_string_ptr(fmt_str, "printf_fmt")
                    .unwrap()
                    .as_pointer_value();

                let casted = self.coerce_for_printf(value, llvm_value);

                self.builder
                    .build_direct_call(
                        printf_fn,
                        &[fmt_ptr.into(), casted.into()],
                        "printf_call",
                    )
                    .unwrap();

            }

            stmt::Ret { value, .. } => {
                if let Ok(n) = value.parse::<i64>() {
                    let v = self.context.i64_type().const_int(n as u64, true);
                    self.builder.build_return(Some(&v)).unwrap();
                } else {
                    let (alloca_ptr, target_type) = self.find_variable_profile(value);
                    let compiled_val = self
                        .builder
                        .build_load(target_type, alloca_ptr, value)
                        .unwrap();
                    self.builder.build_return(Some(&compiled_val)).unwrap();
                }
            }
            stmt::Var {
                heap,
                is_mut,
                name,
                r#type,
                value,
                line,
            } => {
                // For string vars, `value` is the lexer inner string (no quotes), not an identifier.
                // So we must create a global string constant and store that pointer.
                if *r#type == DataType::Str {
                    let target_type = self.llvm_type_from_ast(r#type);

                    // First build the global string constant, then create the slot for the variable.
                    let str_ptr = self
                        .builder
                        .build_global_string_ptr(value, &format!("{}_str", name))
                        .unwrap()
                        .as_pointer_value();

                    let allocation_ptr = if *heap {
                        // allocate space for a pointer (slot)
                        let malloc_fn = self
                            .module
                            .get_function("malloc")
                            .unwrap_or_else(|| self.register_malloc());

                        let pointer_size_bytes = self.context.i64_type().const_int(8, false);
                        let malloc_call = self
                            .builder
                            .build_call(malloc_fn, &[pointer_size_bytes.into()], "raw_str")
                            .unwrap();

                        let void_ptr = malloc_call
                            .try_as_basic_value()
                            .unwrap_basic()
                            .into_pointer_value();

                        let typed_heap_string_ptr = self
                            .builder
                            .build_bit_cast(
                                void_ptr,
                                self.context.i8_type().ptr_type(AddressSpace::default()),
                                name,
                            )
                            .unwrap()
                            .into_pointer_value();

                        // Store the global string pointer into the allocated slot.
                        self.builder
                            .build_store(typed_heap_string_ptr, str_ptr)
                            .unwrap();

                        typed_heap_string_ptr
                    } else {
                        self.builder.build_alloca(target_type, name).unwrap()
                    };

                    if !*heap {
                        self.builder.build_store(allocation_ptr, str_ptr).unwrap();
                    }

                    self.vars.borrow_mut().insert(
                        name.clone(),
                        (
                            allocation_ptr,
                            target_type,
                            str_ptr.into(),
                            *is_mut,
                            *line as usize,
                        ),
                    );
                    return;
                }

                let target_type = self.llvm_type_from_ast(r#type);

                // Non-string vars: initializer can be a literal (int/float/char) or an identifier.
                let compiled_val: BasicValueEnum<'ctx> = match r#type {
                    DataType::I1
                    | DataType::I8
                    | DataType::I32
                    | DataType::I64
                    | DataType::I128 => {
                        // char literal: lexer stores it as `'a'`
                        if value.starts_with('\'') && value.ends_with('\'') && value.len() >= 3 {
                            let c = value.chars().nth(1).unwrap_or('\0') as u64;
                            // Build a constant of the correct integer width.
                            match target_type {
                                BasicTypeEnum::IntType(int_ty) => int_ty.const_int(c, false).into(),
                                _ => self.context.i64_type().const_int(c, false).into(),
                            }
                        } else {
                            // integer literal
                            let n = value.parse::<i64>().unwrap_or_else(|_| {
                                panic!(
                                    "Compiler error: expected integer initializer for '{}'",
                                    name
                                )
                            });
                            match target_type {
                                BasicTypeEnum::IntType(int_ty) => {
                                    // LLVM const_int takes u64.
                                    int_ty.const_int(n as u64, false).into()
                                }
                                _ => self.context.i64_type().const_int(n as u64, false).into(),
                            }
                        }
                    }
                    DataType::F32 | DataType::F64 => {
                        // float literal
                        let f = value.parse::<f64>().unwrap_or_else(|_| {
                            panic!("Compiler error: expected float initializer for '{}'", name)
                        });
                        match target_type {
                            BasicTypeEnum::FloatType(fty) => fty.const_float(f).into(),
                            _ => self.context.f64_type().const_float(f).into(),
                        }
                    }
                    _ => {
                        // Fallback: treat as identifier.
                        let (source_ptr, source_type) = self.find_variable_profile(value);
                        self.builder
                            .build_load(source_type, source_ptr, value)
                            .unwrap()
                    }
                };

                let allocation_ptr = if *heap {
                    let malloc_fn = match self.module.get_function("malloc") {
                        Some(f) => f,
                        None => self.register_malloc(),
                    };

                    let type_size = target_type
                        .size_of()
                        .expect("Type does not have a constant size");

                    let malloc_call = self
                        .builder
                        .build_call(malloc_fn, &[type_size.into()], "heap_alloc")
                        .unwrap();

                    malloc_call
                        .try_as_basic_value()
                        .unwrap_basic()
                        .into_pointer_value()
                } else {
                    self.builder.build_alloca(target_type, name).unwrap()
                };

                self.builder
                    .build_store(allocation_ptr, compiled_val)
                    .unwrap();

                self.vars.borrow_mut().insert(
                    name.clone(),
                    (
                        allocation_ptr,
                        target_type,
                        compiled_val,
                        *is_mut,
                        *line as usize,
                    ),
                );
            }
        }
    }

    fn run_optimizations(&self) {}

    fn find_variable_profile(&self, name: &str) -> (PointerValue<'ctx>, BasicTypeEnum<'ctx>) {
        let vars = self.vars.borrow();
        if let Some(tuple) = vars.get(name) {
            return (tuple.0, tuple.1);
        }
        panic!("Compiler error: Undefined variable '{}'", name);
    }

    fn coerce_for_printf(
        &mut self,
        _ast_value: &Expression,
        llvm_value: BasicValueEnum<'ctx>,
    ) -> BasicValueEnum<'ctx> {
        match llvm_value {
            BasicValueEnum::IntValue(i) => {
                // char and ints: promote to i64 for %lld (safe for small widths)
                let i64_ty = self.context.i64_type();
                // For now just pass value through.
                // (LLVM IR will usually allow i32/i8 to be consumed by varargs with matching format.)
                i.into()
            }
            BasicValueEnum::FloatValue(f) => {
                // For now just pass value through.
                f.into()
            }

            BasicValueEnum::PointerValue(p) => {
                // strings: printf expects i8*
                let i8ptr = self.context.i8_type().ptr_type(AddressSpace::default());
                let typed = p;
                if typed.get_type() == i8ptr {
                    typed.into()
                } else {
                    self.builder
                        .build_bit_cast(typed, i8ptr, "printf_str_cast")
                        .unwrap()
                        .into()
                }
            }

            other => other,
        }
    }

    fn infer_printf_format(&self, value: &Expression) -> Result<&'static str, String> {
        match value {
            Expression::IntLit(_) => Ok("%lld"),
            Expression::CharLit(_) => Ok("%c"),
            Expression::FloatLit(_) => Ok("%f"),
            Expression::StringLit(_) => Ok("%s"),
            Expression::Id(_) => {
                // If it's an identifier, type-checking should ensure it's printable.
                // But without the type here, fall back to %s if it's likely string-ish would require
                // env/type info. We'll handle via LLVM value in stmt() with coercion.
                // For now, default to %lld to avoid panic; coercion will keep it safe-ish.
                Ok("%lld")
            }
        }
    }

    #[allow(dead_code)]
    fn expr(
        &mut self,
        expr: &Expression,
        expected_type: Option<BasicValueEnum<'ctx>>,
    ) -> BasicValueEnum<'ctx> {
        match expr {
            Expression::Id(name) => {
                let (alloca_ptr, target_type) = self.find_variable_profile(name);
                self.builder
                    .build_load(target_type, alloca_ptr, name)
                    .unwrap()
            }
            Expression::FloatLit(value) => {
                let float_type = match expected_type {
                    Some(BasicValueEnum::FloatValue(f)) => f.get_type(),
                    _ => self.context.f64_type(),
                };
                float_type.const_float(*value).into()
            }
            Expression::StringLit(value) => self
                .builder
                .build_global_string_ptr(value, "str_lit")
                .unwrap()
                .as_pointer_value()
                .into(),
            Expression::CharLit(value) => {
                let ascii_bytes = *value as u64;
                self.context.i8_type().const_int(ascii_bytes, false).into()
            }
            Expression::IntLit(value) => {
                if let Some(BasicValueEnum::IntValue(i)) = expected_type {
                    i.get_type().const_int(*value as u64, false).into()
                } else {
                    self.context
                        .i64_type()
                        .const_int(*value as u64, false)
                        .into()
                }
            }
        }
    }
}

