use heretic_nu as h;

use nu_cli::gather_parent_env_vars;
use nu_cmd_lang::create_default_context;
use nu_command::add_shell_command_context;
use nu_protocol::engine::Stack;
use nu_protocol::{PipelineData, Span, Value};
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
    let mut engine_state = create_default_context();
    engine_state = add_shell_command_context(engine_state);
    h::add_missing_commands(&mut engine_state)?;

    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mut command: Option<String> = None;
    let mut exec_file: Option<PathBuf> = None;
    while !args.is_empty() {
        let arg: String = args.remove(0);
        match arg.as_str() {
            "-xx" => {
                engine_state.debugger =
                    Arc::new(Mutex::new(Box::new(h::debug_x::HereticDebuggerX {
                        log_target: h::debug_x::HereticDebuggerLogTarget::StdErr,
                        very_verbose: true,
                    })));
            }
            "-x" => {
                engine_state.debugger = Arc::new(Mutex::new(Box::new(
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

    let init_cwd = std::env::current_dir()?;
    gather_parent_env_vars(&mut engine_state, init_cwd.as_ref());

    let mut stack = Stack::new();

    if let Some(script) = command {
        let res = h::exec_nu(
            &script,
            &mut engine_state,
            &mut stack,
            Some(PipelineData::ByteStream(
                nu_protocol::ByteStream::stdin(Span::unknown())
                    .expect("something, something, stdin is broken"),
                None,
            )),
        );
        let was_ok = res.is_ok();
        h::render(&mut engine_state, &mut stack, res);
        exit(if was_ok { 0 } else { 1 });
    }
    if let Some(filepath) = exec_file {
        let mut script = String::new();
        std::fs::File::open(filepath)
            .expect("File not found.")
            .read_to_string(&mut script)?;
        let res = h::exec_nu(
            &script,
            &mut engine_state,
            &mut stack,
            Some(PipelineData::ByteStream(
                nu_protocol::ByteStream::stdin(Span::unknown())
                    .expect("something, something, stdin is broken"),
                None,
            )),
        );
        let was_ok = res.is_ok();
        h::render(&mut engine_state, &mut stack, res);
        exit(if was_ok { 0 } else { 1 });
    }

    h::exec_nu(
        include_str!("default_config.nu"),
        &mut engine_state,
        &mut stack,
        None,
    )
    .expect("Default config is invalid");

    loop {
        match h::exec_nu("_mini_nu_prompt", &mut engine_state, &mut stack, None) {
            Ok(PipelineData::Value(Value::String { val, .. }, _)) => {
                print!("{}", val);
            }
            Ok(_) => {
                eprintln!("Error: invalid _mini_nu_prompt return type (not a string)");
                print!("> ");
            }
            Err(e) => {
                eprintln!("Error in _mini_nu_prompt: {e}");
                print!("> ");
            }
        }
        let input: String = match h::exec_nu("_mini_nu_input", &mut engine_state, &mut stack, None)
        {
            Ok(PipelineData::Value(Value::String { val, .. }, _)) => val,
            Ok(_) => {
                eprintln!("Error: invalid _mini_nu_input return type (not a string)");
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("Error in _mini_nu_input: {e}");
                std::process::exit(1);
            }
        };
        let res = h::exec_nu(&input, &mut engine_state, &mut stack, None);
        h::render(&mut engine_state, &mut stack, res);
    }
}
