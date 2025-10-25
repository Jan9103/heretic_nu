use nu_engine::command_prelude::*;
use nu_protocol::{debugger::WithoutDebug, engine::StateWorkingSet, PipelineData};

use crate::NuInstance;

#[derive(Clone)]
pub struct Evil;

impl Command for Evil {
    fn name(&self) -> &str {
        "evil"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required("code", SyntaxShape::String, "")
            .input_output_type(Type::Any, Type::Any)
            .category(Category::Debug)
    }

    fn description(&self) -> &str {
        "evaluate a string as nu code.\n\
         it will be executed in its own scope, but you can still pipe in and out.\n\
         but it is pretty expensive since it clones the engine-state as a workaround to enable this.\n\
         use `heretic const evil` if you need it to happen at const-time\n\
         \n\
         PART OF HERETIC-NU"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let val_r = call.req::<Spanned<String>>(engine_state, stack, 0)?;
        let val: String = val_r.item;
        let mut engine_state = engine_state.clone();

        let mut working_set = StateWorkingSet::new(&engine_state);
        let mut block: std::sync::Arc<nu_protocol::ast::Block> =
            nu_parser::parse(&mut working_set, None, val.as_bytes(), false);
        if block.ir_block.is_none() {
            let block_mut = std::sync::Arc::make_mut(&mut block);
            block_mut.ir_block = Some(match nu_engine::compile(&working_set, block_mut) {
                Ok(v) => v,
                Err(_err) => {
                    return Err(ShellError::IncorrectValue {
                        msg: "Failed to compile code in the evil command".into(),
                        val_span: val_r.span,
                        call_span: call.span(),
                    });
                }
            });
        }

        engine_state.merge_delta(working_set.render())?;

        let pipeline_data: PipelineData = nu_engine::eval_block_with_early_return::<WithoutDebug>(
            &engine_state,
            stack,
            &block,
            input,
        )?
        .body;
        Ok(pipeline_data)
    }
}

#[cfg(feature = "heretic_const_evil")]
#[derive(Clone)]
pub struct ConstEvil;

#[cfg(feature = "heretic_const_evil")]
impl ConstEvil {
    #[allow(clippy::result_large_err)]
    fn run_const_evil(call: &Call, code: String) -> Result<PipelineData, ShellError> {
        eprintln!("RUNNING EVAL CODE");
        let mut ni = NuInstance::new().expect("Failed to create new nu instance");

        let result = match ni.exec(&code, None) {
            Ok(v) => v,
            Err(err) => {
                return Err(ShellError::NushellFailedSpanned {
                    msg: format!("Failed to const-eval code: {err}"),
                    label: "here".into(),
                    span: call.span(),
                });
            }
        };
        let result = match match result {
            PipelineData::Empty => {
                return Ok(PipelineData::Empty);
            }
            PipelineData::Value(value, ..) => Ok(value),
            PipelineData::ListStream(list_stream, ..) => list_stream.into_value(),
            PipelineData::ByteStream(byte_stream, ..) => byte_stream.into_value(),
        } {
            Ok(v) => v,
            Err(e) => {
                return Err(ShellError::NushellFailedSpanned {
                    msg: format!("Failed to const-eval code (error during collection): {e}"),
                    label: "here".into(),
                    span: call.span(),
                });
            }
        };
        let result_nuon =
            match nuon::to_nuon(&ni.engine_state, &result, nuon::ToStyle::Raw, None, false) {
                Ok(v) => v,
                Err(e) => {
                    return Err(ShellError::NushellFailedSpanned {
                        msg: format!(
                            "Failed to const-eval code (error during data serialization): {e}"
                        ),
                        label: "here".into(),
                        span: call.span(),
                    });
                }
            };

        Ok(PipelineData::Value(
            nuon::from_nuon(&result_nuon, Some(call.span()))?,
            None,
        ))
    }
}

#[cfg(feature = "heretic_const_evil")]
impl Command for ConstEvil {
    fn name(&self) -> &str {
        "heretic const evil"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required("code", SyntaxShape::String, "")
            .input_output_type(Type::Nothing, Type::Any)
            .category(Category::Debug)
    }

    fn description(&self) -> &str {
        "evaluate a string as nu code.\n\
         runs in a whole seperate nu instance - so full scope change.\n\
         this can be run at `const`, but not sure when nu does something else. So do not trust the current-directory, etc.\n\
         if you do not need it to be `const`-compatible use `evil` instead.\n\
         \n\
         PART OF HERETIC-NU"
    }

    fn is_const(&self) -> bool {
        true
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let code = call.req::<Spanned<String>>(engine_state, stack, 0)?.item;
        Self::run_const_evil(call, code)
    }

    fn run_const(
        &self,
        working_set: &StateWorkingSet,
        call: &Call,
        _input: PipelineData,
    ) -> std::result::Result<PipelineData, ShellError> {
        let code: String = call.req_const::<String>(working_set, 0)?;
        Self::run_const_evil(call, code)
    }
}
