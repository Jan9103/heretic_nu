pub mod commands;
pub mod debug_x;

use nu_engine::eval_block_with_early_return;
use nu_protocol::ast::{Expr, Expression};
use nu_protocol::engine::{EngineState, Stack, StateWorkingSet};
use nu_protocol::{PipelineData, ShellError, Span, Value};
use std::sync::Arc;

#[allow(clippy::result_large_err)]
pub fn exec_nu(
    line: &str,
    engine_state: &mut EngineState,
    stack: &mut Stack,
    pipeline_data: Option<PipelineData>,
) -> Result<PipelineData, ShellError> {
    let mut working_set = StateWorkingSet::new(engine_state);
    let mut block = nu_parser::parse(&mut working_set, None, line.as_bytes(), false);
    // block.pipelines.iter().map(|pip| {
    //     pip.elements.iter().map(|pipe| match pipe.expr {
    //         Expr::ExternalCall(expr, earg) => {
    //         }
    //         _ => todo!(),
    //     })
    // });
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

    Ok(
        eval_block_with_early_return::<nu_protocol::debugger::WithDebug>(
            engine_state,
            stack,
            &block,
            pipeline_data.unwrap_or(PipelineData::Empty),
        )?
        .body,
    )
}

pub fn render(
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

pub fn add_missing_commands(
    engine_state: &mut EngineState,
) -> Result<(), Box<dyn std::error::Error>> {
    let delta = {
        let mut working_set = StateWorkingSet::new(engine_state);
        working_set.add_decl(Box::new(nu_cli::Print));
        working_set.add_decl(Box::new(nu_cli::NuHighlight));

        working_set.add_decl(Box::new(commands::evil::Evil));

        working_set.render()
    };
    engine_state.merge_delta(delta)?;
    Ok(())
}
