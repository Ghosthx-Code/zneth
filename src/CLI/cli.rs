use std::fs;
use std::io::Write;

pub mod F {}
pub mod B {}
use std::{
    path::{Path, PathBuf},
    process::Command,
    process::Stdio,
};

use inkwell::llvm_sys::target;
fn run_clang(bc_path: &Path, out_path: &Path) -> Result<(), String> {
    let out = Command::new("clang")
        .arg("-static")
        .arg("-Wl,-Bstatic")
        .arg("-lc")
        .arg(bc_path)
        .arg("-o")
        .arg(out_path)
        .arg("-O3")
        .arg("-g0")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .output()
        .map_err(|e| e.to_string())?;

    if out.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let stdout = String::from_utf8_lossy(&out.stdout);
        Err(format!(
            "clang failed with {}\nstdout:\n{}\nstderr:\n{}",
            out.status, stdout, stderr
        ))
    }
}

// usage:
// run_clang(format!("./target/{target_name}.bc").into(), format!("./target/{target_name}").into())?;

pub fn parse(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let command = match args.get(0) {
        Some(cmd) => cmd.as_str(),
        None => {
            println!("Usage: <command> [Option<dirname>]");
            println!("Use: -h --help for help");
            return Ok(());
        }
    };

    let mut v: i32 = 1;

    match command {
        "new" => {
            let dirname = match args.get(1) {
                Some(name) => name,
                None => return Err("Error: 'new' command requires a directory name.".into()),
            };

            fs::create_dir_all(format!("./{dirname}/target/etc"))?;
            fs::create_dir_all(format!("./{dirname}/target/Debug/{v}"))?;
            fs::create_dir_all(format!("./{dirname}/src"))?;

            let mut src = fs::File::create(format!("./{dirname}/src/main.z"))?;
            src.write_all(
                b"signed static fn main() ?i32 {\n\tprintf(\"Hello world\");\n\tret 0;\n}",
            )?;

            fs::File::create(format!("./{dirname}/target/etc/build.do"))?;
            let mut vdo = fs::File::create(format!("./{dirname}/.vdo"))?;
            vdo.write_all(
                format!("package name \"{dirname}\"\npackage version \"v0.0.0\"").as_bytes(),
            )?;

            fs::copy(
                format!("./{dirname}/src/main.z"),
                format!("./{dirname}/target/Debug/{v}/debug.z"),
            )?;

            Ok(())
        }
        "build" => {
            use std::{
                fs,
                path::{Path, PathBuf},
                process::Command,
            };

            let source = fs::read_to_string("./src/main.z")?;

            if source.is_empty() {
                println!(
                    "Error Empty File:\nFix: Type in file `signed static fn main() ?i32 {{\n printf(\"Hello world\");\nret 0;\n}}` "
                );
                std::process::exit(0);
            }

            let tokens = crate::F::lexer::Lexer::new(source.clone()).scan_tokens();
            let nodes = crate::F::parser::Parser::new(tokens).parse_program();

            let ctx = inkwell::context::Context::create();
            let line_count = source.lines().count();
            let opt_level = if line_count >= 100 {
                inkwell::OptimizationLevel::None
            } else {
                inkwell::OptimizationLevel::None
            };

            let mut codegen = crate::B::codegen::CodeGen::new(&ctx, "main", opt_level);
            let _ir_text = codegen.compile_program(nodes.expect("REASON"));

            fs::create_dir_all("./target")?;

            // directory name (safe filename)
            let cur_dir = std::env::current_dir()?;
            let name = cur_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("zneth");

            let bc_path: PathBuf = format!("./target/{name}.bc").into();
            let bin_path: PathBuf = format!("./target/{name}").into();
            let ll_path: PathBuf = format!("./target/{name}.ll").into();

            // emit bitcode
            // (adjust if your emit_binary signature differs)
            codegen.emit_binary(bc_path.to_string_lossy().as_ref());

            // link
            let out = Command::new("clang")
                .arg("-static")
                .arg(&bc_path)
                .arg("-o")
                .arg(&bin_path)
                .arg("-O3")
                .arg("-g0")
                .output()?;

            if !out.status.success() {
                let stderr = String::from_utf8_lossy(&out.stderr);
                let stdout = String::from_utf8_lossy(&out.stdout);
                return Err(format!(
                    "clang failed: {}\nstdout:\n{}\nstderr:\n{}",
                    out.status, stdout, stderr
                )
                .into());
            }

            // dump LLVM IR for inspection
            let ll = codegen.module.print_to_string().to_string();
            fs::write(&ll_path, ll)?;

            // ---- your existing debug caching / debug.z writing logic ----
            let mut v = 0;
            if let Ok(entries) = fs::read_dir("./target/Debug") {
                for entry in entries.flatten() {
                    if let Some(name_str) = entry.file_name().to_str() {
                        if let Ok(num) = name_str.parse::<i32>() {
                            if num >= v {
                                v = num + 1;
                            }
                        }
                    }
                }
            }

            let out_dir = format!("./target/Debug/{v}");
            fs::create_dir_all(&out_dir)?;
            let debug_z_path = format!("{out_dir}/debug.z");
            let debug_ll_path = format!("{out_dir}/debug.ll");
            let debug_bc_path = format!("{out_dir}/debug.bc");

            // Cache: best-effort header check
            if Path::new(&debug_ll_path).exists() {
                if let Ok(existing) = fs::read_to_string(&debug_ll_path) {
                    if existing
                        .lines()
                        .next()
                        .is_some_and(|l| l.starts_with(";;zneth:src-hash="))
                    {
                        if !existing.is_empty() {
                            println!("\n==== Cached LLVM IR (debug.ll) ===\n{existing}\n");
                            return Ok(());
                        }
                    }
                }
            }

            fs::copy("./src/main.z", &debug_z_path)?;
            fs::write(&debug_ll_path, codegen.module.print_to_string().to_bytes())?;

            let _ = debug_bc_path; // placeholder if you don't write debug.bc yet
            Ok(())
        }

        "run" => {
            use std::{
                fs,
                path::{Path, PathBuf},
                process::Command,
            };

            let source = fs::read_to_string("./src/main.z")?;

            if source.is_empty() {
                println!(
                    "Error Empty File:\nFix: Type in file `signed static fn main() ?i32 {{\n printf(\"Hello world\");\nret 0;\n}}` "
                );
                std::process::exit(0);
            }

            let tokens = crate::F::lexer::Lexer::new(source.clone()).scan_tokens();
            let nodes = crate::F::parser::Parser::new(tokens).parse_program();

            let ctx = inkwell::context::Context::create();
            let line_count = source.lines().count();
            let opt_level = if line_count >= 100 {
                inkwell::OptimizationLevel::None
            } else {
                inkwell::OptimizationLevel::None
            };

            let mut codegen = crate::B::codegen::CodeGen::new(&ctx, "main", opt_level);
            let _ir_text = codegen.compile_program(nodes.expect("REASON"));

            fs::create_dir_all("./target")?;

            // directory name (safe filename)
            let cur_dir = std::env::current_dir()?;
            let name = cur_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("zneth");

            let bc_path: PathBuf = format!("./target/{name}.bc").into();
            let bin_path: PathBuf = format!("./target/{name}").into();
            let ll_path: PathBuf = format!("./target/{name}.ll").into();

            // emit bitcode
            // (adjust if your emit_binary signature differs)
            codegen.emit_binary(bc_path.to_string_lossy().as_ref());

            // link
            let out = Command::new("clang")
                .arg("-static")
                .arg(&bc_path)
                .arg("-o")
                .arg(&bin_path)
                .arg("-O3")
                .arg("-g0")
                .output()?;

            if !out.status.success() {
                let stderr = String::from_utf8_lossy(&out.stderr);
                let stdout = String::from_utf8_lossy(&out.stdout);
                return Err(format!(
                    "clang failed: {}\nstdout:\n{}\nstderr:\n{}",
                    out.status, stdout, stderr
                )
                .into());
            }

            // dump LLVM IR for inspection
            let ll = codegen.module.print_to_string().to_string();
            fs::write(&ll_path, ll)?;

            // ---- your existing debug caching / debug.z writing logic ----
            let mut v = 0;
            if let Ok(entries) = fs::read_dir("./target/Debug") {
                for entry in entries.flatten() {
                    if let Some(name_str) = entry.file_name().to_str() {
                        if let Ok(num) = name_str.parse::<i32>() {
                            if num >= v {
                                v = num + 1;
                            }
                        }
                    }
                }
            }

            let out_dir = format!("./target/Debug/{v}");
            fs::create_dir_all(&out_dir)?;
            let debug_z_path = format!("{out_dir}/debug.z");
            let debug_ll_path = format!("{out_dir}/debug.ll");
            let debug_bc_path = format!("{out_dir}/debug.bc");

            // Cache: best-effort header check
            if Path::new(&debug_ll_path).exists() {
                if let Ok(existing) = fs::read_to_string(&debug_ll_path) {
                    if existing
                        .lines()
                        .next()
                        .is_some_and(|l| l.starts_with(";;zneth:src-hash="))
                    {
                        if !existing.is_empty() {
                            println!("\n==== Cached LLVM IR (debug.ll) ===\n{existing}\n");
                            return Ok(());
                        }
                    }
                }
            }

            fs::copy("./src/main.z", &debug_z_path)?;
            fs::write(&debug_ll_path, codegen.module.print_to_string().to_bytes())?;

            let _ = debug_bc_path; // placeholder if you don't write debug.bc yet
            std::process::Command::new("bash")
                .arg("-c")
                .arg("./target/{name}");
            std::process::Command::new("rm")
                .arg("-rf")
                .arg("./target/{name}")
                .arg("./target/{name}.bc")
                .arg("./target/{name}.ll");
            Ok(())
        }

        "switch" => {
            let x = match args.get(1) {
                Some(val) => val,
                None => {
                    println!("\x1b[1;31mError:\x1b[0m zneth switch <Or<num>, -cc>");
                    return Ok(());
                }
            };

            match x.as_str() {
                "-cc" => {
                    let mut current_v: i32 = 1;
                    if let Ok(entries) = fs::read_dir("./target/Debug") {
                        for entry in entries.flatten() {
                            if let Some(name_str) = entry.file_name().to_str() {
                                if let Ok(num) = name_str.parse::<i32>() {
                                    if num > current_v {
                                        current_v = num;
                                    }
                                }
                            }
                        }
                    }

                    let target_path = format!("./target/Debug/{current_v}/debug.z");
                    if Path::new(&target_path).exists() {
                        println!("\x1b[1;32mFile Exists:\x1b[0m Version {current_v} is available.");
                    } else {
                        println!(
                            "\x1b[1;31mError:\x1b[0m Latest build (Version {current_v}) file does not exist."
                        );
                    }
                }
                _ => {
                    let source_path = format!("./target/Debug/{x}/debug.z");
                    if !Path::new(&source_path).exists() {
                        println!("\x1b[1;31mError:\x1b[0m Version {x} does not exist.");
                        return Ok(());
                    }

                    fs::copy(&source_path, "./src/main.z")?;
                    println!("\x1b[1;32mSuccess:\x1b[0m Switched main.z back to version {x}.");
                }
            }

            Ok(())
        }

        "-h" | "--help" => {
            println!("\x1b[1;35m==== [ZNETH] Build System Tools ====\x1b[0m\n");
            println!("\x1b[1;33mUsage:\x1b[0m zneth <command> [arguments]\n");
            println!("\x1b[1;36mAvailable Commands:\x1b[0m");
            println!(
                "  \x1b[1;32mnew    \x1b[0;32m<dirname>\x1b[0m   Creates a new project environment inside <dirname>"
            );
            println!(
                "  \x1b[1;32mbuild  \x1b[0m             Increments version folder and builds src/main.z into debug.z"
            );
            println!(
                "  \x1b[1;32mrun    \x1b[0m             Executes the latest built version of your program"
            );
            println!(
                "  \x1b[1;32mswitch \x1b[0m             Switches between active built versions"
            );
            println!("\n\x1b[1;34mExample:\x1b[0m");
            println!("  zneth new my_project");
            println!("  cd my_project && zneth build\n");
            Ok(())
        }

        _ => {
            println!("Unknown command.");
            Ok(())
        }
    }
}
