pub mod ast {}
pub mod lexer {}

use crate::F::ast::{DataType, Expression, Function, Program, stmt};
use crate::F::lexer::{Token, TokenKind};
#[derive(Debug)]
pub struct Parser {
    current_token: usize,
    tokens: Vec<Token>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            current_token: 0,
            tokens,
        }
    }

    pub fn parse_program(&mut self) -> Option<Program> {
        let mut module_name: String = "unknown".to_string();
        let mut functions: Vec<Function> = Vec::new();

        // 1. Parse optional module declaration
        if self.current_type() == TokenKind::Module {
            self.adv(); // Move past 'module' keyword

            if self.current_type() == TokenKind::Id {
                // FIX: Extract the value FIRST, while current_token points at the ID
                module_name = self.tokens[self.current_token].value.clone();

                self.adv(); // Now safe to move past the ID to the semicolon

                if self.current_type() == TokenKind::SemiColoun {
                    self.adv(); // Move past the semicolon
                }
            } else {
                eprintln!(
                    "Syntax Error: Expected module identifier, found '{:?}' with value '{}'.",
                    self.current_type(),
                    self.tokens[self.current_token].value
                );
                return None;
            }
        }

        // 2. Loop through all top-level functions until End-Of-File
        while self.current_type() != TokenKind::Eof {
            match self.tokens[self.current_token].typ {
                TokenKind::Signed => {
                    self.adv();
                    if let Some(fun) = self.parse_fn(false) {
                        functions.push(fun);
                    } else {
                        return None; // Stop parsing if function compilation fails
                    }
                }
                TokenKind::Unsigned => {
                    self.adv();
                    if let Some(fun) = self.parse_fn(true) {
                        functions.push(fun);
                    } else {
                        return None;
                    }
                }
                // 3. Prevent infinite loops and panics on unexpected syntax
                _ => {
                    let current = &self.tokens[self.current_token];
                    eprintln!(
                        "Syntax Error: Unexpected token '{:?}' with value '{}' at token index {}.",
                        current.typ, current.value, self.current_token
                    );
                    // Advance past the bad token so the parser doesn't freeze
                    self.adv();
                    return None;
                }
            }
        }

        // 4. Return the fully populated Program node
        Some(Program {
            module_name,
            functions,
        })
    }

    fn parse_fn(&mut self, heap: bool) -> Option<Function> {
        let mut damtic: bool = false;
        let mut is_option: bool = false;

        // Check modifiers
        if self.current_type() == TokenKind::Static {
            self.adv();
        } else if self.current_type() == TokenKind::Damtic {
            damtic = true;
            self.adv();
        }

        // Validate 'fn' keyword
        if self.current_type() != TokenKind::Fn {
            eprintln!(
                "Syntax Error: Expected 'fn' keyword, found {:?}",
                self.current_type()
            );
            self.adv();
            return None;
        }
        self.adv();

        // Get function name
        if self.current_type() != TokenKind::Id {
            eprintln!(
                "Syntax Error: Expected function name, found {:?}",
                self.current_type()
            );
            self.adv();
            return None;
        }
        let name = self.tokens[self.current_token].value.to_string();
        self.adv();

        // Validate '('
        if self.current_type() != TokenKind::LeftParen {
            eprintln!(
                "Syntax Error: Expected '(', found {:?}",
                self.current_type()
            );
            self.adv();
            return None;
        }
        self.adv();

        // Validate ')'
        if self.current_type() != TokenKind::RightParen {
            panic!();
            self.adv();
            return None;
        }
        self.adv();

        // Check for optional return type marker '?'
        if self.current_type() == TokenKind::Question {
            is_option = true;
            self.adv();
        }

        // Capture return type token
        let return_type = self.data_type();

        // FIX: Only advance past the return_type if it isn't part of the statement body.
        // If your language uses a return type token like 'Int' before the block, advance:
        self.adv();

        // Parse the function body
        // Now self.current_token points exactly at the start of your statement (e.g., '{')
        let body = self.parse_stmt();

        Some(Function {
            name,
            is_option,
            return_type,
            body,
            heap,
            damtic,
        })
    }

    pub fn parse_stmt(&mut self) -> Option<stmt> {
        let current_token_obj = &self.tokens[self.current_token];
        let line = current_token_obj.line;

        match current_token_obj.typ {
            TokenKind::Printf => self.parse_print(),
            TokenKind::Ret => self.parse_ret(),
            TokenKind::LeftCurl => self.parse_block(),
            TokenKind::Signed => self.parse_var(false),
            TokenKind::Unsigned => self.parse_var(true),
            _ => {
                eprintln!(
                    "Syntax Error: Unexpected token '{:?}' with value '{}' on line {}.",
                    current_token_obj.typ, current_token_obj.value, line
                );
                self.adv();
                None
            }
        }
    }

    fn parse_var(&mut self, heap: bool) -> Option<stmt> {
        let current_token_obj = &self.tokens[self.current_token];
        let line = current_token_obj.line;
        let mut errors: Vec<String> = Vec::new();
        let mut is_mut: bool = false;
        self.adv();
        if self.current_type() == TokenKind::Mut {
            is_mut = true;
            self.adv();
        }
        let name = self.tokens[self.current_token].value.clone();
        self.adv();
        let ty: DataType = self.data_type();
        self.adv();
        if self.current_type() != TokenKind::Equal {
            if heap {
                if is_mut {
                    errors.push(format!(
                        "\x1b[31m[!] \x1b[32mZneth Compiler \x1b[32mError: \x1b[36mMissing-Equal:
\x1b[90m |   \x1b[32mFile: \x1b[31msrc/main.z \x1b[38;5;208m-> \x1b[33mLine \x1b[33m{line}
\x1b[90m |                           
\x1b[90m |       \x1b[33m{line} \x1b[90m| \x1b[31m`unsigned mut {name} {}  <data>;`
\x1b[90m |
\x1b[34m[?] Fix: \x1b[34m`unsigned mut {name} {} = <data>;`\x1b[0m",
                        format!("{:?}", ty),
                        format!("{:?}", ty)
                    ));
                } else {
                    errors.push(format!(
                        "\x1b[31m[!] \x1b[32mZneth Compiler \x1b[32mError: \x1b[36mMissing-Equal:
\x1b[90m |   \x1b[32mFile: \x1b[31msrc/main.z \x1b[38;5;208m-> \x1b[33mLine \x1b[33m{line}
\x1b[90m |
\x1b[90m |       \x1b[33m{line} \x1b[90m| \x1b[31m`unsigned {name} {}  <data>;`
\x1b[90m |
\x1b[34m[?] Fix: \x1b[34m`unsigned {name} {} = <data>;`\x1b[0m",
                        format!("{:?}", ty),
                        format!("{:?}", ty)
                    ));
                }
            } else {
                if is_mut {
                    errors.push(format!(
                        "\x1b[31m[!] \x1b[32mZneth Compiler \x1b[32mError: \x1b[36mMissing-Equal:
\x1b[90m |   \x1b[32mFile: \x1b[31msrc/main.z \x1b[38;5;208m-> \x1b[33mLine \x1b[33m{line}
\x1b[90m |
\x1b[90m |       \x1b[33m{line} \x1b[90m| \x1b[31m`signed mut {name} {}  <data>;`
\x1b[90m |
\x1b[34m[?] Fix: \x1b[34m`signed mut {name} {} = <data>;`\x1b[0m",
                        format!("{:?}", ty),
                        format!("{:?}", ty)
                    ));
                } else {
                    errors.push(format!(
                        "\x1b[31m[!] \x1b[32mZneth Compiler \x1b[32mError: \x1b[36mMissing-Equal:
\x1b[90m |   \x1b[32mFile: \x1b[31msrc/main.z \x1b[38;5;208m-> \x1b[33mLine \x1b[33m{line}
\x1b[90m |
\x1b[90m |       \x1b[33m{line} \x1b[90m| \x1b[31m`signed {name} {}  <data>;`
\x1b[90m |
\x1b[34m[?] Fix: \x1b[34m`signed {name} {} = <data>;`\x1b[0m",
                        format!("{:?}", ty),
                        format!("{:?}", ty),
                    ));
                }
            }
        }
        self.adv();
        let value = self.tokens[self.current_token].value.clone();
        self.adv();
        if self.current_type() == TokenKind::SemiColoun {
            self.adv();
        } else {
            if heap {
                if is_mut {
                    errors.push(format!(
                        "\x1b[31m[!] \x1b[32mZneth Compiler \x1b[32mError: \x1b[36mMissing-Semi:
\x1b[90m |   \x1b[32mFile: \x1b[31msrc/main.z \x1b[38;5;208m-> \x1b[33mLine \x1b[33m{line}
\x1b[90m |
\x1b[90m |       \x1b[33m{line} \x1b[90m| \x1b[31m`signed {name} {} = <data>`
\x1b[90m |
\x1b[34m[?] Fix: \x1b[34m`signed {name} {} = <data>;`\x1b[0m",
                        format!("{:?}", ty),
                        format!("{:?}", ty),
                    ));
                } else {
                    errors.push(format!(
                        "\x1b[31m[!] \x1b[32mZneth Compiler \x1b[32mError: \x1b[36mMissing-Semi:
\x1b[90m |   \x1b[32mFile: \x1b[31msrc/main.z \x1b[38;5;208m-> \x1b[33mLine \x1b[33m{line}
\x1b[90m |
\x1b[90m |       \x1b[33m{line} \x1b[90m| \x1b[31m`signed {name} {} = <data>`
\x1b[90m |
\x1b[34m[?] Fix: \x1b[34m`signed {name} {} = <data>;`\x1b[0m",
                        format!("{:?}", ty),
                        format!("{:?}", ty),
                    ));
                }
            } else {
                if is_mut {
                    errors.push(format!(
                        "\x1b[31m[!] \x1b[32mZneth Compiler \x1b[32mError: \x1b[36mMissing-Semi:
\x1b[90m |   \x1b[32mFile: \x1b[31msrc/main.z \x1b[38;5;208m-> \x1b[33mLine \x1b[33m{line}
\x1b[90m |
\x1b[90m |       \x1b[33m{line} \x1b[90m| \x1b[31m`signed {name} {} = <data>`
\x1b[90m |
\x1b[34m[?] Fix: \x1b[34m`signed {name} {} = <data>;`\x1b[0m",
                        format!("{:?}", ty),
                        format!("{:?}", ty),
                    ));
                } else {
                    errors.push(format!(
                        "\x1b[31m[!] \x1b[32mZneth Compiler \x1b[32mError: \x1b[36mMissing-Semi:
\x1b[90m |   \x1b[32mFile: \x1b[31msrc/main.z \x1b[38;5;208m-> \x1b[33mLine \x1b[33m{line}
\x1b[90m |
\x1b[90m |       \x1b[33m{line} \x1b[90m| \x1b[31m`signed {name} {} = <data>`
\x1b[90m |
\x1b[34m[?] Fix: \x1b[34m`signed {name} {} = <data>;`\x1b[0m",
                        format!("{:?}", ty),
                        format!("{:?}", ty),
                    ));
                }
            }
        }

        if !errors.is_empty() {
            for e in &errors {
                println!("{}", e);
                print!("\n");
            }
            println!("\x1b[31mErrors: {}\x1b[0m", errors.len());
            std::process::exit(0);
        }

        Some(stmt::Var {
            heap,
            is_mut,
            name: name.to_string(),
            r#type: ty,
            value: value.to_string(),
            line,
        })
    }

    fn data_type(&mut self) -> DataType {
        match self.current_type() {
            TokenKind::I32 => DataType::I32,
            TokenKind::I64 => DataType::I64,
            TokenKind::I128 => DataType::I128,
            TokenKind::F32 => DataType::F32,
            TokenKind::F64 => DataType::F64,
            TokenKind::Str => DataType::Str,
            TokenKind::I8 => DataType::I8,
            TokenKind::I1 => DataType::I1,
            _ => todo!(),
        }
    }

    fn adv(&mut self) {
        if self.current_token < self.tokens.len() {
            self.current_token += 1;
        }
    }

    fn current_type(&mut self) -> TokenKind {
        self.tokens[self.current_token].typ.clone()
    }

    fn parse_ret(&mut self) -> Option<stmt> {
        let mut errors: Vec<String> = Vec::new();
        let current_token_obj = &self.tokens[self.current_token];
        let line = current_token_obj.line;
        self.adv(); // Move past 'ret'

        // Assume returning a number, string, or identifier value
        let value = self.tokens[self.current_token].value.clone();
        self.adv(); // Move past the returned value

        if self.current_type() == TokenKind::SemiColoun {
            self.adv(); // Move past ';'
        } else {
            errors.push(format!(
                "\x1b[31m[!] \x1b[32mZneth Compiler \x1b[32mError: \x1b[36mMissing-Semi:
\x1b[90m |   \x1b[32mFile: \x1b[31msrc/main.z \x1b[38;5;208m-> \x1b[33mLine \x1b[33m{line}
\x1b[90m |
\x1b[90m |       \x1b[33m{line} \x1b[90m| \x1b[31m`ret {value} `
\x1b[90m |
\x1b[34m[?] Fix: \x1b[34m`ret {value};`\x1b[0m",
            ));
        }

        if !errors.is_empty() {
            for e in &errors {
                println!("{}", e);
                print!("\n");
            }
            println!("\x1b[31mErrors: {}\x1b[0m", errors.len());
            std::process::exit(0);
        }

        Some(stmt::Ret { value, line })
    }

    fn parse_print(&mut self) -> Option<stmt> {
        let current_token_obj = &self.tokens[self.current_token];
        let line = current_token_obj.line;
        let mut errors: Vec<String> = Vec::new();
        self.adv(); // Move past 'printf'

        if self.current_type() != TokenKind::LeftParen {
            errors.push(format!(
                "\x1b[31m[!] \x1b[32mZneth Compiler \x1b[32mError: \x1b[36mMissing-Paren:
\x1b[90m |   \x1b[32mFile: \x1b[31msrc/main.z \x1b[38;5;208m-> \x1b[33mLine \x1b[33m{line}
\x1b[90m |
\x1b[90m |       \x1b[33m{line} \x1b[90m| \x1b[31m`printf \"<data>\");`
\x1b[90m |
\x1b[34m[?] Fix: \x1b[34m`printf(\"<data>\");`\x1b[0m",
            ));
        }
        self.adv(); // Move past '('

        // `printf` accepts either a string literal, numeric/char literal, or an identifier.
        // We'll accept:
        // - TokenKind::String
        // - TokenKind::Num
        // - TokenKind::Float
        // - TokenKind::Char
        // - TokenKind::Id
        if !matches!(
            self.current_type(),
            TokenKind::String | TokenKind::Num | TokenKind::Float | TokenKind::Char | TokenKind::Id
        ) {
            errors.push(format!(
                "Syntax Error: Expected value inside printf on line {} (string/number/char/id)",
                line
            ));
        }
        let value = self.parse_expr();

        // After parse_expr(), the next token must be ')'.
        if self.current_type() != TokenKind::RightParen {
            errors.push(format!(

                "\x1b[31m[!] \x1b[32mZneth Compiler \x1b[32mError: \x1b[36mMissing-Paren:
\x1b[90m |   \x1b[32mFile: \x1b[31msrc/main.z \x1b[38;5;208m-> \x1b[33mLine \x1b[33m{line}
\x1b[90m |
\x1b[90m |       \x1b[33m{line} \x1b[90m| \x1b[31m`printf(\"{value:?}\" ;`
\x1b[90m |
\x1b[34m[?] Fix: \x1b[34m`printf(\"{value:?}\");`\x1b[0m",
            ));
        }
        self.adv(); // Move past ')'

        if self.current_type() == TokenKind::SemiColoun {
            self.adv(); // Move past ';'
        } else {
            errors.push(format!(
                "\x1b[31m[!] \x1b[32mZneth Compiler \x1b[32mError: \x1b[36mMissing-Semi:
\x1b[90m |   \x1b[32mFile: \x1b[31msrc/main.z \x1b[38;5;208m-> \x1b[33mLine \x1b[33m{line}
\x1b[90m |
\x1b[90m |       \x1b[33m{line} \x1b[90m| \x1b[31m`printf(\"{value:?}\")`
\x1b[90m |
\x1b[34m[?] Fix: \x1b[34m`printf(\"{value:?}\");`\x1b[0m",
            ));
        }

        if !errors.is_empty() {
            for e in &errors {
                println!("{}", e);
                print!("\n");
            }
            println!("\x1b[31mErrors: {}\x1b[0m", errors.len());
            std::process::exit(0);
        }

        Some(stmt::Printf {
            value: value?,
            line,
        })
    }
    fn parse_block(&mut self) -> Option<stmt> {
        let current_token_obj = &self.tokens[self.current_token];
        let line = current_token_obj.line;
        if self.current_type() != TokenKind::LeftCurl {
            eprintln!(
                "{}",
                format!(
                    "\x1b[31m[!] \x1b[32mZneth Compiler \x1b[32mError: \x1b[36mMissing-Curl:
\x1b[90m |   \x1b[32mFile: \x1b[31msrc/main.z \x1b[38;5;208m-> \x1b[33mLine \x1b[33m{line}
\x1b[90m |
\x1b[90m |       \x1b[33m{line} \x1b[90m| \x1b[31m` `
\x1b[90m |
\x1b[34m[?] Fix: \x1b[34m`{{",
                )
            );
            std::process::exit(0);
        }
        self.adv(); // Move past '{'
        let mut block_statements = Vec::new();

        // Loop until we hit the closing brace or End of File
        while self.current_type() != TokenKind::RightCurl && self.current_type() != TokenKind::Eof {
            if let Some(s) = self.parse_stmt() {
                block_statements.push(s);
            } else {
                // CRITICAL: If a nested statement fails to parse,
                // we MUST advance to prevent an infinite recursive freeze loop!
                eprintln!("Skipping bad token inside block: {:?}", self.current_type());
                self.adv();
            }
        }
        let current_token_obj = &self.tokens[self.current_token];
        let line1 = current_token_obj.line;
        if self.current_type() == TokenKind::RightCurl {
            self.adv(); // Safely move past the closing '}'
        } else {
            eprintln!(
                "{}",
                format!(
                    "\x1b[31m[!] \x1b[32mZneth Compiler \x1b[32mError: \x1b[36mMissing-Curl:
\x1b[90m |   \x1b[32mFile: \x1b[31msrc/main.z \x1b[38;5;208m-> \x1b[33mLine \x1b[33m{line1}
\x1b[90m |
\x1b[90m |       \x1b[33m{line} \x1b[90m| \x1b[31m`{{`
\x1b[90m |
\x1b[90m |       \x1b[33m{line1} \x1b[90m| \x1b[31m` `
\x1b[90m |
\x1b[34m[?] Fix: \x1b[34m`}}",
                )
            );
            std::process::exit(0);
        }

        Some(stmt::Block(block_statements))
    }
    fn peek(&mut self) -> Option<&Token> {
        self.tokens.get(self.current_token)
    }
    fn bump(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.current_token);
        self.current_token += 1;
        tok
    }
    fn parse_char_lit(&mut self) -> Option<Expression> {
        let text = self.tokens[self.current_token].value.clone();
        self.adv();
        let inner_char = text.chars().nth(1);
        Some(Expression::CharLit(inner_char?))
    }
    fn parse_expr(&mut self) -> Option<Expression> {
        match self.current_type() {
            TokenKind::String => {
                let val = self.tokens[self.current_token].value.clone();
                self.adv();
                return Some(Expression::StringLit(val));
            }
            TokenKind::Float => {
                let val = self.tokens[self.current_token]
                    .value
                    .parse::<f64>()
                    .unwrap_or(0.0);
                self.adv();
                return Some(Expression::FloatLit(val));
            }
            TokenKind::Num => {
                let val = self.tokens[self.current_token]
                    .value
                    .parse::<i64>()
                    .unwrap_or(0);
                self.adv();
                return Some(Expression::IntLit(val));
            }

            TokenKind::Char => self.parse_char_lit(),
            TokenKind::Id => {
                let name = self.tokens[self.current_token].value.clone();
                self.adv();
                return Some(Expression::Id(name));
            }
            _ => return None,
        }
    }
}
