use crate::F::ast::{DataType, Function, Program, stmt};

#[derive(Debug, Clone)]
pub enum TypecheckErrorKind {
    UndefinedVariable,
    TypeMismatch,
    InvalidLiteral,
    MissingReturnValue,
}

#[derive(Debug, Clone)]
pub struct TypecheckError {
    pub kind: TypecheckErrorKind,
    pub line: i32,
    pub message: String,
}

impl TypecheckError {
    fn new(kind: TypecheckErrorKind, line: i32, message: impl Into<String>) -> Self {
        Self {
            kind,
            line,
            message: message.into(),
        }
    }

    pub fn format_pretty(&self) -> String {
        // Keep formatting consistent with existing parser errors (ANSI colors).
        format!(
            "\\x1b[31m[!] \\x1b[32mZneth Compiler \\x1b[32mError: \\x1b[36m{}\\n\\x1b[90m |   \\x1b[32mLine \\x1b[33m{}\\x1b[0m\\n\\x1b[90m |   \\x1b[37m{}\\x1b[0m",
            self.kind_name(),
            self.line,
            self.message
        )
    }

    fn kind_name(&self) -> &'static str {
        match self.kind {
            TypecheckErrorKind::UndefinedVariable => "Undefined variable",
            TypecheckErrorKind::TypeMismatch => "Type mismatch",
            TypecheckErrorKind::InvalidLiteral => "Invalid literal",
            TypecheckErrorKind::MissingReturnValue => "Missing return value",
        }
    }
}

pub fn typecheck_program(program: &Program) -> Result<(), Vec<TypecheckError>> {
    let mut errors = Vec::new();

    for f in &program.functions {
        errors.extend(typecheck_function(f));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn typecheck_function(f: &Function) -> Vec<TypecheckError> {
    let mut errors = Vec::new();
    let mut env = TypeEnv::new();

    if let Some(body) = &f.body {
        typecheck_stmt(body, f.return_type, &mut env, &mut errors);
    } else {
        // For now: if there is no body, keep as error.
        errors.push(TypecheckError::new(
            TypecheckErrorKind::MissingReturnValue,
            0,
            format!("Function '{}' has no body", f.name),
        ));
    }

    errors
}

#[derive(Default)]
struct TypeEnv {
    vars: std::collections::HashMap<String, VarInfo>,
}

#[derive(Debug, Clone)]
struct VarInfo {
    ty: DataType,
    is_mut: bool,
}

impl TypeEnv {
    fn new() -> Self {
        Self {
            vars: std::collections::HashMap::new(),
        }
    }

    fn insert(&mut self, name: String, ty: DataType, is_mut: bool) {
        self.vars.insert(name, VarInfo { ty, is_mut });
    }

    fn get(&self, name: &str) -> Option<&VarInfo> {
        self.vars.get(name)
    }
}

fn typecheck_stmt(
    s: &stmt,
    fn_ret_ty: DataType,
    env: &mut TypeEnv,
    errors: &mut Vec<TypecheckError>,
) {
    match s {
        stmt::Block(sts) => {
            for st in sts {
                typecheck_stmt(st, fn_ret_ty, env, errors);
            }
        }
        stmt::Printf { value, line } => {
            // `printf` supports printing string, numbers, chars, or identifiers.
            // We validate that `value` is either:
            // - a string literal: "..."
            // - an integer literal
            // - a float literal
            // - a char literal: 'a'
            // - an identifier previously declared.

            // String literal (parser/lexer stores without quotes for TokenKind::String)
            if value.starts_with('"') && value.ends_with('"') {
                return;
            }

            // Char literal: lexer stores it as `'a'`
            if value.starts_with('\'') && value.ends_with('\'') && value.len() >= 3 {
                return;
            }

            // Integer literal
            if value.parse::<i64>().is_ok() {
                return;
            }

            // Float literal
            if value.parse::<f64>().is_ok() {
                return;
            }

            // Identifier
            if env.get(value).is_some() {
                return;
            }

            errors.push(TypecheckError::new(
                TypecheckErrorKind::UndefinedVariable,
                *line,
                format!("printf argument '{}' is not a known variable and not a supported literal.", value),
            ));
        }
        stmt::Var {
            heap: _,
            is_mut,
            name,
            r#type,
            value,
            line,
        } => {
        // Validate initializer.
        // NOTE: lexer stores TokenKind::String as *inner* content without quotes.
        match infer_literal_or_id_type(value, *line, *r#type, env) {
            Ok(_) => {
                env.insert(name.clone(), *r#type, *is_mut);
            }
            Err(e) => errors.push(e),
        }
        }
        stmt::Ret { value, line } => {
            // Validate returned expression type.
            match infer_literal_or_id_return_type(value, *line, fn_ret_ty, env) {
                Ok(_) => {}
                Err(e) => errors.push(e),
            }
        }
    }
}

fn infer_literal_or_id_type(
    value: &str,
    line: i32,
    expected_ty: DataType,
    env: &TypeEnv,
) -> Result<DataType, TypecheckError> {
    // Try integer.
    if let Ok(_) = value.parse::<i64>() {
        // Only accept for integer-ish types.
        return match expected_ty {
            DataType::I8 | DataType::I32 | DataType::I64 | DataType::I128 | DataType::I1 => Ok(expected_ty),
            _ => Err(TypecheckError::new(
                TypecheckErrorKind::TypeMismatch,
                line,
                format!("Expected {:?} but found integer literal '{}'.", expected_ty, value),
            )),
        };
    }

    // Try float.
    if let Ok(_) = value.parse::<f64>() {
        return match expected_ty {
            DataType::F32 | DataType::F64 => Ok(expected_ty),
            _ => Err(TypecheckError::new(
                TypecheckErrorKind::TypeMismatch,
                line,
                format!("Expected {:?} but found float literal '{}'.", expected_ty, value),
            )),
        };
    }

    // Try char literal: lexer stores it as `'a'`.
    if value.starts_with('\'') && value.ends_with('\'') && value.len() >= 3 {
        return match expected_ty {
            DataType::I8 | DataType::I32 | DataType::I64 | DataType::I128 | DataType::I1 => Ok(expected_ty),
            _ => Err(TypecheckError::new(
                TypecheckErrorKind::TypeMismatch,
                line,
                format!("Expected {:?} but found char literal {}.", expected_ty, value),
            )),
        };
    }

    // String literal:
    // lexer stores TokenKind::String as *inner* content without quotes.
    // So `str x = Hello world;` token value is `Hello world`.
    // If expected type is Str, accept any non-numeric, non-char literal here.
    // (If you actually want to forbid plain identifiers as string literals, this is where to tighten it.)
    if expected_ty == DataType::Str {
        // If it looks like a number/char, earlier branches would have matched.
        // Treat everything else as string content.
        return Ok(DataType::Str);
    }

    // Otherwise treat as identifier.
    if let Some(var) = env.get(value) {
        if var.ty != expected_ty {
            return Err(TypecheckError::new(
                TypecheckErrorKind::TypeMismatch,
                line,
                format!(
                    "Type mismatch: declared {:?} but initializer '{}' has type {:?}.",
                    expected_ty, value, var.ty
                ),
            ));
        }
        return Ok(var.ty);
    }

    Err(TypecheckError::new(
        TypecheckErrorKind::UndefinedVariable,
        line,
        format!("Use of undefined variable '{}'.", value),
    ))
}

fn infer_literal_or_id_return_type(
    value: &str,
    line: i32,
    expected_ty: DataType,
    env: &TypeEnv,
) -> Result<DataType, TypecheckError> {
    // Identifier
    if let Some(var) = env.get(value) {
        if var.ty != expected_ty {
            return Err(TypecheckError::new(
                TypecheckErrorKind::TypeMismatch,
                line,
                format!(
                    "Return type mismatch: function returns {:?} but '{}' is {:?}.",
                    expected_ty, value, var.ty
                ),
            ));
        }
        return Ok(var.ty);
    }

    // String literal (lexer stores inner contents without quotes)
    if expected_ty == DataType::Str {
        return Ok(DataType::Str);
    }

    // Integer literal
    if value.parse::<i64>().is_ok() {

        match expected_ty {
            DataType::I8 | DataType::I32 | DataType::I64 | DataType::I128 | DataType::I1 => return Ok(expected_ty),
            _ => {
                return Err(TypecheckError::new(
                    TypecheckErrorKind::TypeMismatch,
                    line,
                    format!(
                        "Return type mismatch: function returns {:?} but got integer literal '{}'.",
                        expected_ty, value
                    ),
                ));
            }
        }
    }

    // Float literal
    if value.parse::<f64>().is_ok() {
        match expected_ty {
            DataType::F32 | DataType::F64 => return Ok(expected_ty),
            _ => {
                return Err(TypecheckError::new(
                    TypecheckErrorKind::TypeMismatch,
                    line,
                    format!(
                        "Return type mismatch: function returns {:?} but got float literal '{}'.",
                        expected_ty, value
                    ),
                ));
            }
        }
    }

    // Otherwise
    Err(TypecheckError::new(
        TypecheckErrorKind::UndefinedVariable,
        line,
        format!(
            "Invalid return expression '{}': not a known variable and not a supported literal.",
            value
        ),
    ))
}

