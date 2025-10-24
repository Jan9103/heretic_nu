use nu_engine::command_prelude::*;

#[derive(Copy, Clone)]
pub struct HereticVersion;

impl Command for HereticVersion {
    fn name(&self) -> &str {
        "version"
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name())
            .input_output_types(vec![(Type::Nothing, Type::record())])
            .allow_variants_without_examples(true)
            .category(Category::Core)
    }

    fn description(&self) -> &str {
        ""
    }

    fn is_const(&self) -> bool {
        true
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> std::result::Result<PipelineData, ShellError> {
        let pd = (nu_cmd_lang::Version {}).run(engine_state, stack, call, input)?;
        match pd {
            PipelineData::Value(Value::Record { val, .. }, ..) => {
                let mut val: nu_protocol::Record = val.into_owned();
                val.insert("is_heretic_nu", Value::bool(true, call.head));
                Ok(PipelineData::Value(Value::record(val, call.head), None))
            }
            _ => {
                panic!("Nu's 'version' command did not return a record");
            }
        }
    }

    fn run_const(
        &self,
        working_set: &StateWorkingSet,
        call: &Call,
        input: PipelineData,
    ) -> std::result::Result<PipelineData, ShellError> {
        let pd = (nu_cmd_lang::Version {}).run_const(working_set, call, input)?;
        match pd {
            PipelineData::Value(Value::Record { val, .. }, ..) => {
                let mut val: nu_protocol::Record = val.into_owned();
                val.insert("is_heretic_nu", Value::bool(true, call.head));
                Ok(PipelineData::Value(Value::record(val, call.head), None))
            }
            _ => {
                panic!("Nu's 'version' command did not return a record");
            }
        }
    }
}
