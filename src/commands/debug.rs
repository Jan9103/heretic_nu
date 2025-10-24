use std::time::{SystemTime, UNIX_EPOCH};

use nu_engine::command_prelude::*;
use nu_protocol::PipelineData;

use crate::{
    debug_x::{HereticDebuggerLogTarget, HereticDebuggerX},
    step_debug::HereticStepDebugger,
};

#[derive(Clone)]
pub struct HereticDebug;

impl Command for HereticDebug {
    fn name(&self) -> &str {
        "heretic_debug"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required("mode", SyntaxShape::String, "'x', 'xx', 'step', or 'off'")
            .named(
                "target-file",
                SyntaxShape::Filepath,
                "For 'x' and 'xx' with '--output=file'",
                None,
            )
            .named(
                "output",
                SyntaxShape::String,
                "For 'x' and 'xx': where to send the data? 'file', 'stdout', 'stderr'\n\
                 file-path: ~/.local/share/heretic_nu/debug_logs/<timestamp>.txt",
                None,
            )
            .input_output_type(Type::Nothing, Type::Nothing)
            .category(Category::Debug)
    }

    fn description(&self) -> &str {
        "enable or disable a debugger.\n\
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
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let val_r = call.req::<Spanned<String>>(engine_state, stack, 0)?;
        let val: String = val_r.item;

        match val.as_str() {
            "x" | "xx" => {
                let op = call.get_flag::<String>(engine_state, stack, "output")?;
                let log_target = match op.unwrap_or(String::from("stdout")).as_str() {
                    "stdout" => HereticDebuggerLogTarget::StdOut,
                    "stderr" => HereticDebuggerLogTarget::StdErr,
                    "file" => HereticDebuggerLogTarget::LogDir(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("Time-Travel is not supported")
                            .as_secs(),
                    ),
                    _ => {
                        return Err(ShellError::IncorrectValue {
                            msg: "Invalid log-target".into(),
                            val_span: call
                                .get_flag_span(stack, "output")
                                .unwrap_or(Span::unknown()),
                            call_span: call.span(),
                        });
                    }
                };

                engine_state
                    .activate_debugger(Box::new(HereticDebuggerX {
                        log_target,
                        very_verbose: &val == "xx",
                    }))
                    .expect("Failed to enable x-debugger");
            }
            "step" => {
                engine_state
                    .activate_debugger(Box::new(HereticStepDebugger::default()))
                    .expect("Failed to enable step-debugger");
            }
            "off" => {
                engine_state
                    .deactivate_debugger()
                    .expect("Failed to disable debugger");
            }
            _ => {
                return Err(ShellError::IncorrectValue {
                    msg: "Unsupported debug mode".into(),
                    val_span: val_r.span,
                    call_span: call.span(),
                });
            }
        }

        Ok(PipelineData::Empty)
    }
}
