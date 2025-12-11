use nu_engine::command_prelude::*;
use nu_protocol::{PipelineData, Range};

#[derive(Copy, Clone, Debug)]
pub struct HereSpanCommand;

impl Command for HereSpanCommand {
    fn name(&self) -> &str {
        "heretic span here"
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name()).input_output_type(Type::Nothing, Type::Range)
    }

    fn description(&self) -> &str {
        "get the span of here"
    }

    fn run(
        &self,
        _engine_state: &EngineState,
        _stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> std::result::Result<PipelineData, ShellError> {
        Ok(PipelineData::Value(
            Value::range(
                Range::new(
                    Value::int(call.head.start as i64, call.head),
                    Value::int(call.head.start as i64 + 1, call.head),
                    Value::int(call.head.end as i64, call.head),
                    nu_protocol::ast::RangeInclusion::Inclusive,
                    call.head,
                )?,
                call.head,
            ),
            None,
        ))
    }

    fn is_const(&self) -> bool {
        true
    }

    fn run_const(
        &self,
        _working_set: &StateWorkingSet,
        call: &Call,
        _input: PipelineData,
    ) -> std::result::Result<PipelineData, ShellError> {
        Ok(PipelineData::Value(
            Value::range(
                Range::new(
                    Value::int(call.head.start as i64, call.head),
                    Value::int(call.head.start as i64 + 1, call.head),
                    Value::int(call.head.end as i64, call.head),
                    nu_protocol::ast::RangeInclusion::Inclusive,
                    call.head,
                )?,
                call.head,
            ),
            None,
        ))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct GetSpanCommand;

impl GetSpanCommand {}

impl Command for GetSpanCommand {
    fn name(&self) -> &str {
        "heretic span contents"
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name())
            .input_output_type(Type::Nothing, Type::Any)
            .required("span", SyntaxShape::Range, "")
    }

    fn description(&self) -> &str {
        ""
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> std::result::Result<PipelineData, ShellError> {
        let range = call.req::<Range>(engine_state, stack, 0)?;
        let range: (i64, i64) = match range {
            Range::IntRange(int_range) => (
                int_range.start(),
                match int_range.end() {
                    std::ops::Bound::Included(i) => i,
                    std::ops::Bound::Excluded(i) => i - 1,
                    std::ops::Bound::Unbounded => {
                        return Err(ShellError::IncorrectValue {
                            msg: String::from("span needs a end"),
                            val_span: Span::unknown(),
                            call_span: call.head,
                        });
                    }
                },
            ),
            Range::FloatRange(_) => {
                return Err(ShellError::IncorrectValue {
                    msg: String::from("expected a integer range, got a float range"),
                    val_span: Span::unknown(),
                    call_span: call.head,
                });
            }
        };
        let span: Span = Span::new(range.0 as usize, range.1 as usize); // i don't use a 32bit system - not my problem
        let data: &[u8] = engine_state.get_span_contents(span);

        Ok(PipelineData::Value(Value::binary(data, call.head), None))
    }

    fn is_const(&self) -> bool {
        true
    }

    fn run_const(
        &self,
        working_set: &StateWorkingSet,
        call: &Call,
        _input: PipelineData,
    ) -> std::result::Result<PipelineData, ShellError> {
        let range = call.req_const::<Range>(working_set, 0)?;
        let range: (i64, i64) = match range {
            Range::IntRange(int_range) => (
                int_range.start(),
                match int_range.end() {
                    std::ops::Bound::Included(i) => i,
                    std::ops::Bound::Excluded(i) => i - 1,
                    std::ops::Bound::Unbounded => {
                        return Err(ShellError::IncorrectValue {
                            msg: String::from("span needs a end"),
                            val_span: Span::unknown(),
                            call_span: call.head,
                        });
                    }
                },
            ),
            Range::FloatRange(_) => {
                return Err(ShellError::IncorrectValue {
                    msg: String::from("expected a integer range, got a float range"),
                    val_span: Span::unknown(),
                    call_span: call.head,
                });
            }
        };
        let span: Span = Span::new(range.0 as usize, range.1 as usize); // i don't use a 32bit system - not my problem
        let data: &[u8] = working_set.get_span_contents(span);

        Ok(PipelineData::Value(Value::binary(data, call.head), None))
    }
}
