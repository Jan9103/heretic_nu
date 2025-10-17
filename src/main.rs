use nu_cli::gather_parent_env_vars;
use nu_cmd_lang::create_default_context;
use nu_command::add_shell_command_context;
use nu_engine::eval_block_with_early_return;
use nu_protocol::debugger::WithoutDebug;
use nu_protocol::engine::{EngineState, Stack, StateWorkingSet};
use nu_protocol::{PipelineData, ShellError, Span, Value};
#[cfg(not(feature = "embed-app"))]
use std::io::Read;
use std::sync::Arc;

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
            Some(PipelineData::ByteStream(
                nu_protocol::ByteStream::stdin(Span::unknown())
                    .expect("something, something, stdin is broken"),
                None,
            )),
            true,
        )?;
        Ok(())
    }

    #[cfg(not(feature = "embed-app"))]
    {
        let mut args = std::env::args();
        if let Some(arg1) = args.nth(1) {
            let mut script = String::new();
            if arg1 == "-c" {
                script = args.next().expect("Missing command-argument after '-c'");
            } else {
                std::fs::File::open(arg1)
                    .expect("File not found.")
                    .read_to_string(&mut script)?;
            }
            let res = exec_nu(
                &script,
                &mut engine_state,
                &mut stack,
                Some(PipelineData::ByteStream(
                    nu_protocol::ByteStream::stdin(Span::unknown())
                        .expect("something, something, stdin is broken"),
                    None,
                )),
            );
            render(&mut engine_state, &mut stack, res);
            return Ok(());
        }

        exec_nu(
            include_str!("default_config.nu"),
            &mut engine_state,
            &mut stack,
            None,
        )
        .expect("Default config is invalid");

        loop {
            match exec_nu("_mini_nu_prompt", &mut engine_state, &mut stack, None) {
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
            let input: String = match exec_nu("_mini_nu_input", &mut engine_state, &mut stack, None)
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
            let res = exec_nu(&input, &mut engine_state, &mut stack, None);
            render(&mut engine_state, &mut stack, res);
        }
    }
}

#[allow(clippy::result_large_err)]
fn exec_nu(
    line: &str,
    engine_state: &mut EngineState,
    stack: &mut Stack,
    pipeline_data: Option<PipelineData>,
) -> Result<PipelineData, ShellError> {
    let mut working_set = StateWorkingSet::new(engine_state);
    let mut block = nu_parser::parse(&mut working_set, None, line.as_bytes(), false);
    if block.ir_block.is_none() {
        let block_mut = Arc::make_mut(&mut block);
        match nu_engine::compile(&working_set, block_mut) {
            Ok(ir_block) => {
                block_mut.ir_block = Some(ir_block);
            }
            Err(err) => {
                working_set.compile_errors.push(err);
            }
        };
    }
    engine_state.merge_delta(working_set.render())?;

    Ok(eval_block_with_early_return::<WithoutDebug>(
        engine_state,
        stack,
        &block,
        pipeline_data.unwrap_or(PipelineData::Empty),
    )?
    .body)
}

fn render(
    engine_state: &mut EngineState,
    stack: &mut Stack,
    result: Result<PipelineData, ShellError>,
) {
    match result {
        Ok(pipeline_data) => match pipeline_data.into_value(Span::unknown()) {
            Ok(value) => {
                // dbg!(&value);
                match value {
                    Value::Nothing { .. } => println!(),
                    _ => {
                        match exec_nu(
                            "print",
                            engine_state,
                            stack,
                            Some(PipelineData::Value(value, None)),
                        ) {
                            Ok(_) => (),
                            Err(e) => {
                                eprintln!("RENDER FAILED:");
                                render(engine_state, stack, Err(e));
                            }
                        }
                    }
                }
            }
            Err(e) => eprintln!("Conversion-Error (into_value): {:?}", e),
        },
        Err(render_error) => {
            eprintln!("Nu-Error: {:?}", render_error);
            #[allow(clippy::single_match)]
            match render_error {
                ShellError::VariableNotFoundAtRuntime { span } => {
                    let span_contents = engine_state.get_span_contents(span);
                    if let Ok(a) = std::str::from_utf8(span_contents) {
                        eprintln!("Span contents: {a}");
                    }
                }
                _ => (),
            }
        }
    }
}

fn add_missing_commands(engine_state: &mut EngineState) -> Result<(), Box<dyn std::error::Error>> {
    let delta = {
        let mut working_set = StateWorkingSet::new(engine_state);
        working_set.add_decl(Box::new(nu_cli::Print));
        working_set.add_decl(Box::new(nu_cli::NuHighlight));
        working_set.render()
    };
    engine_state.merge_delta(delta)?;
    Ok(())
}
