use std::{fs::OpenOptions, io::Write, path::PathBuf};

use nu_protocol::debugger::Debugger;

type LogIdType = u64;

fn log_file(id: LogIdType) -> PathBuf {
    std::env::home_dir()
        .expect("No home dir")
        .join(".local")
        .join("share")
        .join("heretic_nu")
        .join("debug_logs")
        .join(format!("{id}.txt"))
}

#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub enum HereticDebuggerLogTarget {
    StdErr,
    StdOut,
    LogDir(LogIdType),
}
impl Default for HereticDebuggerLogTarget {
    fn default() -> Self {
        Self::StdOut
    }
}
impl HereticDebuggerLogTarget {
    pub fn log(&self, message: &str) {
        match self {
            HereticDebuggerLogTarget::StdErr => eprintln!("-x- {message}"),
            HereticDebuggerLogTarget::StdOut => println!("-x- {message}"),
            HereticDebuggerLogTarget::LogDir(id) => {
                let mut file = OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(log_file(*id))
                    .expect("Failed to open log-file");
                file.write_all(format!("{message}\n").as_bytes())
                    .expect("Failde to write to log-file");
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct HereticDebuggerX {
    pub log_target: HereticDebuggerLogTarget,
    pub very_verbose: bool,
}

impl HereticDebuggerX {
    fn log(&self, message: &str) {
        self.log_target.log(message);
    }
    fn for_block(
        &mut self,
        engine_state: &nu_protocol::engine::EngineState,
        block: &nu_protocol::ast::Block,
        message: &str,
    ) {
        if let Some(span) = block.span {
            let code = String::from_utf8_lossy(engine_state.get_span_contents(span))[..10]
                .replace("\n", " ");
            self.log(&format!(
                "{message}: span={}..{}, code={code}…",
                span.start, span.end
            ));
        } else {
            self.log(&format!("{message}: span=null"));
        }
    }
    fn for_element(
        &mut self,
        engine_state: &nu_protocol::engine::EngineState,
        pipeline_element: &nu_protocol::ast::PipelineElement,
        message: &str,
    ) {
        let span = pipeline_element.expr.span;
        let code =
            String::from_utf8_lossy(engine_state.get_span_contents(span))[..10].replace("\n", " ");
        self.log(&format!(
            "{message}: span={}..{}, code={code}…",
            span.start, span.end
        ));
    }
    fn for_instruction(
        &mut self,
        engine_state: &nu_protocol::engine::EngineState,
        ir_block: &nu_protocol::ir::IrBlock,
        instruction_index: usize,
        message: &str,
    ) {
        if !self.very_verbose {
            return;
        }
        let instruction: &nu_protocol::ir::Instruction = &ir_block.instructions[instruction_index];
        self.log(&format!(
            "{message}: instruction={}",
            instruction.display(engine_state, &ir_block.data)
        ));
    }
}

impl Debugger for HereticDebuggerX {
    fn activate(&mut self) {
        match self.log_target {
            HereticDebuggerLogTarget::StdErr => (),
            HereticDebuggerLogTarget::StdOut => (),
            HereticDebuggerLogTarget::LogDir(id) => {
                let lf = log_file(id);
                let lfp = lf
                    .parent()
                    .expect("failed to get parent-dir of log-file (impossible)");
                if !std::fs::exists(lfp).expect("Failed to check if log-dir exists") {
                    std::fs::create_dir_all(lfp).expect("Failed to create log-dir");
                }
                std::fs::File::create(lf).expect("Failed to create log-file");
            }
        }
        self.log("activated HereticDebuggerX")
    }
    fn deactivate(&mut self) {
        self.log("deactivated HereticDebuggerX");
    }

    fn enter_block(
        &mut self,
        engine_state: &nu_protocol::engine::EngineState,
        block: &nu_protocol::ast::Block,
    ) {
        self.for_block(engine_state, block, "enter block");
    }

    fn leave_block(
        &mut self,
        engine_state: &nu_protocol::engine::EngineState,
        block: &nu_protocol::ast::Block,
    ) {
        self.for_block(engine_state, block, "leave block");
    }

    fn enter_element(
        &mut self,
        engine_state: &nu_protocol::engine::EngineState,
        pipeline_element: &nu_protocol::ast::PipelineElement,
    ) {
        self.for_element(engine_state, pipeline_element, "enter element");
    }

    fn leave_element(
        &mut self,
        engine_state: &nu_protocol::engine::EngineState,
        element: &nu_protocol::ast::PipelineElement,
        result: &Result<nu_protocol::PipelineData, nu_protocol::ShellError>,
    ) {
        self.for_element(
            engine_state,
            element,
            if result.is_ok() {
                "leave element (ok)"
            } else {
                "leave element (err)"
            },
        );
    }

    fn enter_instruction(
        &mut self,
        engine_state: &nu_protocol::engine::EngineState,
        ir_block: &nu_protocol::ir::IrBlock,
        instruction_index: usize,
        _registers: &[nu_protocol::PipelineExecutionData],
    ) {
        self.for_instruction(
            engine_state,
            ir_block,
            instruction_index,
            "   enter instruction",
        );
    }

    fn leave_instruction(
        &mut self,
        engine_state: &nu_protocol::engine::EngineState,
        ir_block: &nu_protocol::ir::IrBlock,
        instruction_index: usize,
        _registers: &[nu_protocol::PipelineExecutionData],
        error: Option<&nu_protocol::ShellError>,
    ) {
        self.for_instruction(
            engine_state,
            ir_block,
            instruction_index,
            if error.is_some() {
                "   leave instruction (err)"
            } else {
                "   leave instruction (ok)"
            },
        );
    }

    fn report(
        &self,
        _engine_state: &nu_protocol::engine::EngineState,
        debugger_span: nu_protocol::Span,
    ) -> Result<nu_protocol::Value, nu_protocol::ShellError> {
        Ok(nu_protocol::Value::nothing(debugger_span))
    }
}
