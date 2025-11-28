use heretic_nu as h;

use nu_protocol::{PipelineData, ShellError, Span, Value};
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
    #[cfg(feature = "nu_stor")]
    h::fix_stor().expect("Failed to initialize nu stor");

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
        nu_instance.load_default_config();
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
        nu_instance.load_default_config();
        nu_instance.run_file(
            String::from(filepath.as_os_str().to_str().unwrap()),
            &args,
            Some(PipelineData::ByteStream(
                nu_protocol::ByteStream::stdin(Span::unknown())
                    .expect("something, something, stdin is broken"),
                None,
            )),
        )?;
        exit(0);
    }

    nu_instance.load_all_configs()?;

    loop {
        match nu_instance.exec("_heretic_nu_prompt", None) {
            Ok(PipelineData::Value(Value::String { val, .. }, _)) => {
                print!("{}", val);
            }
            Ok(PipelineData::Value(v, _)) => {
                eprintln!(
                    "Error: invalid _heretic_nu_prompt return type (not a string): {}",
                    v.get_type()
                );
                print!("> ");
            }
            Ok(PipelineData::ListStream(_, _)) => {
                eprintln!(
                    "Error: invalid _heretic_nu_prompt return type (PipelineData::ListStream)"
                );
                print!("> ");
            }
            Ok(PipelineData::ByteStream(_, _)) => {
                eprintln!(
                    "Error: invalid _heretic_nu_prompt return type (PipelineData::ByteStream)"
                );
                print!("> ");
            }
            Ok(PipelineData::Empty) => {}
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
        nu_instance.set_interactive(true);
        let start_time = std::time::Instant::now();
        let res = nu_instance.exec(&input, None);
        nu_instance.engine_state.add_env_var(
            "CMD_DURATION_MS".into(),
            Value::string(
                format!("{}", start_time.elapsed().as_millis()),
                Span::unknown(),
            ),
        );
        let exitcode: (i32, Span) = match res {
            Ok(_) => (0, Span::unknown()),
            Err(ShellError::NonZeroExitCode { exit_code, span }) => (exit_code.get(), span),
            Err(_) => (1, Span::unknown()),
        };
        nu_instance.set_exitcode(exitcode.0, exitcode.1);
        nu_instance.set_interactive(false);
        nu_instance.render(res);
    }
}
