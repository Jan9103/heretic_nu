use nu_cli::gather_parent_env_vars;
use nu_cmd_lang::create_default_context;
use nu_command::add_shell_command_context;
use nu_engine::eval_block_with_early_return;
use nu_protocol::debugger::WithoutDebug;
use nu_protocol::engine::{EngineState, Stack, StateWorkingSet};
use nu_protocol::{PipelineData, Span, Value};
use std::fs::File;
use std::io::Write;
use std::io::{self, Read};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine_state = create_default_context();
    engine_state = add_shell_command_context(engine_state);

    let init_cwd = std::env::current_dir()?;
    gather_parent_env_vars(&mut engine_state, init_cwd.as_ref());

    let mut stack = Stack::new();

    let mut args = std::env::args();
    match args.nth(1) {
        Some(arg1) => {
            let mut script = String::new();
            if arg1 == "-c" {
                script = args.next().expect("Missing command-argument after '-c'");
            } else {
                File::open(arg1)
                    .expect("File not found.")
                    .read_to_string(&mut script)?;
            }
            exec_nu(&script, &mut engine_state, &mut stack)?;
            return Ok(());
        }
        None => {}
    }

    loop {
        print!("$ ");
        let _ = io::stdout().flush(); // continue even if flush fails
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                // crtl+d sends a 0 length (no newline) input
                println!("Bye.");
                return Ok(());
            }
            Ok(_) => {
                let line = input.trim();
                if line.is_empty() {
                    println!("\n");
                    continue;
                }
                if line == "exit" {
                    println!("Bye.");
                    return Ok(());
                }

                exec_nu(line, &mut engine_state, &mut stack)?;
            }
            Err(e) => {
                println!("IO-Error: {}", e);
                return Ok(());
            }
        }
    }
}

fn exec_nu(
    line: &str,
    engine_state: &mut EngineState,
    stack: &mut Stack,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut working_set = StateWorkingSet::new(&engine_state);
    let block = nu_parser::parse(&mut working_set, None, line.as_bytes(), false);
    engine_state.merge_delta(working_set.render())?;

    match eval_block_with_early_return::<WithoutDebug>(
        &engine_state,
        stack,
        &block,
        PipelineData::Empty,
    ) {
        Ok(pipeline_data) => match pipeline_data.into_value(Span::test_data()) {
            Ok(value) => println!("{}", render_value(&value)),
            Err(e) => eprintln!("Conversion-Error: {:?}", e),
        },
        Err(e) => {
            eprintln!("Nu-Error: {:?}", e);
        }
    }
    Ok(())
}

fn render_value(value: &Value) -> String {
    match value {
        Value::String { val, .. } => val.clone(),
        Value::Nothing { .. } => "".into(),
        other => other.to_debug_string(),
    }
}
