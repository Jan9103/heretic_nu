use nu_engine::command_prelude::*;
use nu_protocol::{debugger::WithoutDebug, engine::StateWorkingSet, PipelineData};

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
         \n\
         PART OF HERETIC-NU"
    }

    fn extra_description(&self) -> &str {
        ""
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
