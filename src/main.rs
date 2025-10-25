use heretic_nu as h;

use nu_protocol::{PipelineData, ShellError, Span, Value};
use std::io::Read;
use std::path::PathBuf;
use std::process::exit;
use std::sync::{Arc, Mutex};

const HELP_TEXT: &str = "
HERETIC NU

Usages:
* INLINE: heretic_nu [flags] -c CODE
* SCRIPT: heretic_nu [flags] SCRIPT_FILE_PATH ...ARGS
* REPL:   heretic_nu [flags]

Flags:
* -x:  debug mode
* -xx: verbose debug mode
* --help | -h: show this text
";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    nu_command::tls::CRYPTO_PROVIDER.default();

    let mut nu_instance = h::NuInstance::new()?;

    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mut command: Option<String> = None;
    let mut exec_file: Option<PathBuf> = None;
    #[cfg(feature = "nu_std")]
    let mut use_nu_std: bool = true;
    while !args.is_empty() {
        let arg: String = args.remove(0);
        match arg.as_str() {
            "-xx" => {
                nu_instance.engine_state.debugger =
                    Arc::new(Mutex::new(Box::new(h::debug_x::HereticDebuggerX {
                        log_target: h::debug_x::HereticDebuggerLogTarget::StdErr,
                        very_verbose: true,
                    })));
            }
            "-x" => {
                nu_instance.engine_state.debugger = Arc::new(Mutex::new(Box::new(
                    h::debug_x::HereticDebuggerX::default(),
                )));
            }
            "--commands" | "-c" => {
                if args.is_empty() {
                    println!("'--commands' is missing argument");
                    exit(1);
                }
                command = Some(args.remove(0));
            }
            "--help" | "-h" => {
                println!("{HELP_TEXT}");
                exit(0);
            }
            #[cfg(feature = "heretic_step_debug")]
            "--step-debug-ui" => {
                if args.is_empty() {
                    println!("'--step-debug-ui' is missing argument");
                    exit(1);
                }
                nu_instance.engine_state.add_env_var(
                    "socket_dir".into(),
                    Value::string(args.remove(0), Span::unknown()),
                );
                command = Some(include_str!("step_debug_server.nu").into());
            }
            #[cfg(feature = "nu_std")]
            "--no-std-lib" => {
                use_nu_std = false;
            }
            _ if arg.starts_with('-') => {
                println!("Usage error: unknown argument: {arg}\n\nHelp text:\n\n{HELP_TEXT}");
                exit(0);
            }
            _ => {
                exec_file = Some(PathBuf::from(arg));
                break;
            }
        }
    }

    #[cfg(feature = "nu_std")]
    if use_nu_std {
        nu_instance.add_stdlib()?;
    }

    if let Some(script) = command {
        let res = nu_instance.exec(
            &script,
            Some(PipelineData::ByteStream(
                nu_protocol::ByteStream::stdin(Span::unknown())
                    .expect("something, something, stdin is broken"),
                None,
            )),
        );
        let was_ok = res.is_ok();
        nu_instance.render(res);
        exit(if was_ok { 0 } else { 1 });
    }
    if let Some(filepath) = exec_file {
        let mut script = String::new();
        std::fs::File::open(filepath)
            .expect("File not found.")
            .read_to_string(&mut script)?;
        let res = nu_instance.exec(
            &script,
            Some(PipelineData::ByteStream(
                nu_protocol::ByteStream::stdin(Span::unknown())
                    .expect("something, something, stdin is broken"),
                None,
            )),
        );
        let was_ok = res.is_ok();
        nu_instance.render(res);
        exit(if was_ok { 0 } else { 1 });
    }

    nu_instance
        .exec(include_str!("default_config.nu"), None)
        .expect("Default config is invalid");

    if let Some(home_dir) = std::env::home_dir() {
        let config_file = home_dir
            .join(".config")
            .join("heretic_nu")
            .join("config.nu");
        if config_file.is_file() {
            let mut script = String::new();
            std::fs::File::open(config_file)
                .expect("File not found.")
                .read_to_string(&mut script)?;
            nu_instance.exec(&script, None)?;
        }
        let ev = nu_instance
            .engine_state
            .get_env_var("heretic_nu_autoload_dirs")
            .cloned();
        match ev {
            Some(Value::List { vals, .. }) => {
                for val in vals {
                    match val {
                        Value::String { val, .. } => {
                            let fp = PathBuf::from(val);
                            if fp.is_dir() {
                                for f in std::fs::read_dir(fp)
                                    .expect("Failed to read autoload-dir contents")
                                {
                                    let f: std::fs::DirEntry =
                                        f.expect("Failed to read autoload-dir contents");
                                    let p = f.path();
                                    if p.is_file()
                                        && p.extension() == Some(std::ffi::OsStr::new("nu"))
                                    {
                                        let mut script = String::new();
                                        std::fs::File::open(p)
                                            .expect("File not found.")
                                            .read_to_string(&mut script)?;
                                        nu_instance.exec(&script, None)?;
                                    }
                                }
                            }
                        }
                        _ => {
                            return Err(Box::new(ShellError::TypeMismatch {
                                err_message: "$env.heretic_nu_autoload_dirs has to be a list<path>"
                                    .into(),
                                span: Span::unknown(),
                            }));
                        }
                    }
                }
            }
            Some(_) => {
                return Err(Box::new(ShellError::TypeMismatch {
                    err_message: "$env.heretic_nu_autoload_dirs has to be a list<path>".into(),
                    span: Span::unknown(),
                }));
            }
            None => {}
        }
    }

    loop {
        match nu_instance.exec("_heretic_nu_prompt", None) {
            Ok(PipelineData::Value(Value::String { val, .. }, _)) => {
                print!("{}", val);
            }
            Ok(_) => {
                eprintln!("Error: invalid _heretic_nu_prompt return type (not a string)");
                print!("> ");
            }
            Err(e) => {
                eprintln!("Error in _heretic_nu_prompt: {e}");
                print!("> ");
            }
        }
        let input: String = match nu_instance.exec("_heretic_nu_input", None) {
            Ok(PipelineData::Value(Value::String { val, .. }, _)) => val,
            Ok(_) => {
                eprintln!("Error: invalid _heretic_nu_input return type (not a string)");
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("Error in _heretic_nu_input: {e}");
                std::process::exit(1);
            }
        };
        let res = nu_instance.exec(&input, None);
        nu_instance.render(res);
    }
}
