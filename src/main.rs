use nu_cli::gather_parent_env_vars;
use nu_cmd_lang::create_default_context;
use nu_command::add_shell_command_context;
use nu_engine::eval_block_with_early_return;
use nu_protocol::debugger::WithoutDebug;
use nu_protocol::engine::{EngineState, Stack, StateWorkingSet};
use nu_protocol::{PipelineData, Span, Value};
#[cfg(not(feature = "embed-app"))]
use std::io::{Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine_state = create_default_context();
    engine_state = add_shell_command_context(engine_state);
    add_missing_commands(&mut engine_state)?;

    let init_cwd = std::env::current_dir()?;
    gather_parent_env_vars(&mut engine_state, init_cwd.as_ref());

    let mut stack = Stack::new();

    #[cfg(feature = "embed-app")]
    {
        let script = include_str!("script.nu");
        exec_nu(&script, &mut engine_state, &mut stack, None, false)?;
        exec_nu(
            &format!(
                "main {}",
                std::env::args()
                    .map(|i| format!("\"{}\" ", i.replace("\\", "\\\\").replace("\"", "\\\"")))
                    .fold(String::new(), |a, b| a + &b)
            ),
            &mut engine_state,
            &mut stack,
            None,
            true,
        )?;
        Ok(())
    }

    #[cfg(not(feature = "embed-app"))]
    {
        let mut args = std::env::args();
        match args.nth(1) {
            Some(arg1) => {
                let mut script = String::new();
                if arg1 == "-c" {
                    script = args.next().expect("Missing command-argument after '-c'");
                } else {
                    std::fs::File::open(arg1)
                        .expect("File not found.")
                        .read_to_string(&mut script)?;
                }
                exec_nu(&script, &mut engine_state, &mut stack, None, true)?;
                return Ok(());
            }
            None => {}
        }

        loop {
            print!("$ ");
            let _ = std::io::stdout().flush(); // continue even if flush fails
            let mut input = String::new();
            match std::io::stdin().read_line(&mut input) {
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

                    exec_nu(
                        // format!("print ({})", line).as_str(),
                        line,
                        &mut engine_state,
                        &mut stack,
                        None,
                        false,
                    )?;
                }
                Err(e) => {
                    println!("IO-Error: {}", e);
                    return Ok(());
                }
            }
        }
    }
}

fn exec_nu(
    line: &str,
    engine_state: &mut EngineState,
    stack: &mut Stack,
    pipeline_data: Option<PipelineData>,
    do_print: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut working_set = StateWorkingSet::new(&engine_state);
    let block = nu_parser::parse(&mut working_set, None, line.as_bytes(), false);
    engine_state.merge_delta(working_set.render())?;

    match eval_block_with_early_return::<WithoutDebug>(
        &engine_state,
        stack,
        &block,
        pipeline_data.unwrap_or(PipelineData::Empty),
    ) {
        Ok(pipeline_data) => {
            if do_print {
                match pipeline_data.into_value(Span::test_data()) {
                    // Ok(value) => println!("{}", render_value(&value)),
                    Ok(value) => {
                        dbg!(&value);
                        match value {
                            // Value::String { val, .. } => println!("{}", val),
                            Value::Nothing { .. } => println!(""),
                            // Value::Record { .. } | Value::List { .. } => exec_nu(
                            //     "$in | table",
                            //     engine_state,
                            //     stack,
                            //     Some(PipelineData::Value(value, None)),
                            //     true,
                            // )?,
                            // _ => exec_nu(
                            //     "$in | to nuon",
                            //     engine_state,
                            //     stack,
                            //     Some(PipelineData::Value(value, None)),
                            //     true,
                            // )?,
                            _ => exec_nu(
                                "print $in",
                                engine_state,
                                stack,
                                Some(PipelineData::Value(value, None)),
                                false, // no need, already printed by the `print` implementation
                            )?,
                        }
                    }
                    Err(e) => eprintln!("Conversion-Error (into_value): {:?}", e),
                }
            }
        }
        Err(e) => {
            eprintln!("Nu-Error: {:?}", e);
        }
    }
    Ok(())
}

// fn render_value(value: &Value) -> String {
//     match value {
//         Value::String { val, .. } => val.clone(),
//         Value::Nothing { .. } => "".into(),
//         other => other.to_debug_string(),
//     }
// }

fn add_missing_commands(engine_state: &mut EngineState) -> Result<(), Box<dyn std::error::Error>> {
    let delta = {
        let mut working_set = StateWorkingSet::new(&engine_state);
        working_set.add_decl(Box::new(nu_cli::Print));
        working_set.render()
    };
    engine_state.merge_delta(delta)?;
    Ok(())
}
