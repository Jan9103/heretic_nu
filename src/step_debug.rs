use nu_protocol::{debugger::Debugger, Span, Value};

use crate::NuInstance;

const MAX_SOCKET_DIR_PATH_LENGTH: usize = 512;

const RESET: &str = "\x1b[0m";
const HEADER: &str = "\x1b[1;37;45m";

// TODO: send over actual structured data in addition to render
// TODO: recieve commands back (update variables, etc)

#[derive(Copy, Clone, Debug, Default)]
pub struct HereticStepDebugger {
    socket_dir: Option<[char; MAX_SOCKET_DIR_PATH_LENGTH]>,
}

impl HereticStepDebugger {
    fn send_to_server(&self, text: String) {
        let mut ni = NuInstance::new().expect("Failed to create new NU instance");
        ni.engine_state.add_env_var(
            "sock_dir".into(),
            Value::string(
                self.socket_dir
                    .expect("called debugger step before activate (HereticStepDebugger)")
                    .iter()
                    .filter(|i| **i != '\0')
                    .collect::<String>(),
                Span::unknown(),
            ),
        );
        ni.exec(
            include_str!("step_debug_client.nu"),
            Some(nu_protocol::PipelineData::Value(
                Value::string(text, Span::unknown()),
                None,
            )),
        )
        .expect("Failed to run step_debug_client.nu");
    }
}

impl Debugger for HereticStepDebugger {
    fn activate(&mut self) {
        let socket_dir: String = if let Some(res) = match NuInstance::new()
            .expect("Failed to create new NU instance")
            .exec(
                r#"
                    let sock_dir = (mktemp --directory)
                    touch ($sock_dir | path join 'no_data_lock.bin')
                    def is_installed [app: string]: nothing -> bool {
                        (which $app).0?.path? != null
                    }
                    if (is_installed 'wezterm') {
                        job spawn { ^wezterm start --always-new-process --no-auto-connect heretic_nu --step-debug-ui $sock_dir }
                    } else {
                        print --stderr 'Failed to find a terminal-emulator'
                        exit 1
                    }
                    return $sock_dir
                "#,
                None,
            )
            .expect(
                "Failed to launch the HereticStepDebugger ui app in a new terminal (nu-code-error)",
            ) {
            nu_protocol::PipelineData::Value(value, ..) => match value {
                nu_protocol::Value::String { val, .. } => Some(val),
                _ => None,
            },
            nu_protocol::PipelineData::Empty
            | nu_protocol::PipelineData::ListStream(..)
            | nu_protocol::PipelineData::ByteStream(..) => None,
        } {
            res
        } else {
            panic!("Failed to launch the HereticStepDebugger ui app in a new terminal (did not return a socket_dir)");
        };
        if socket_dir.chars().count() >= MAX_SOCKET_DIR_PATH_LENGTH {
            panic!("Socket dir path is to long (>= {MAX_SOCKET_DIR_PATH_LENGTH})");
        }
        let mut sda: [char; MAX_SOCKET_DIR_PATH_LENGTH] = ['\0'; MAX_SOCKET_DIR_PATH_LENGTH];
        let mut cs = socket_dir.chars();
        #[allow(clippy::needless_range_loop)]
        for i in 0..MAX_SOCKET_DIR_PATH_LENGTH {
            sda[i] = cs.next().unwrap_or('\0');
        }
        self.socket_dir = Some(sda);
    }

    fn deactivate(&mut self) {}

    #[allow(unused_variables)]
    fn enter_block(
        &mut self,
        engine_state: &nu_protocol::engine::EngineState,
        block: &nu_protocol::ast::Block,
    ) {
    }

    #[allow(unused_variables)]
    fn leave_block(
        &mut self,
        engine_state: &nu_protocol::engine::EngineState,
        block: &nu_protocol::ast::Block,
    ) {
    }

    #[allow(unused_variables)]
    fn enter_element(
        &mut self,
        engine_state: &nu_protocol::engine::EngineState,
        pipeline_element: &nu_protocol::ast::PipelineElement,
    ) {
    }

    #[allow(unused_variables)]
    fn leave_element(
        &mut self,
        engine_state: &nu_protocol::engine::EngineState,
        element: &nu_protocol::ast::PipelineElement,
        result: &Result<nu_protocol::PipelineData, nu_protocol::ShellError>,
    ) {
    }

    #[allow(unused_variables)]
    fn enter_instruction(
        &mut self,
        engine_state: &nu_protocol::engine::EngineState,
        ir_block: &nu_protocol::ir::IrBlock,
        instruction_index: usize,
        registers: &[nu_protocol::PipelineExecutionData],
    ) {
        self.send_to_server(format!(
            "\
                {HEADER}  <=== ENV ===>  {RESET}\n\
                {env_vars}\n\
                \n\
                {HEADER}  <=== REGISTERS ===>  {RESET}\n\
                {registers}\n\
                \n\
                {HEADER}  <=== IR ===>  {RESET}\n\
                {ir}\n\
                \n\
                \x1b[1;36mstep: ENTER INSTRUCTION{RESET}\n\
                press <return> to continue execution\
            ",
            ir = render_ir(engine_state, ir_block, instruction_index),
            registers = render_registers(registers),
            env_vars = render_env_vars(engine_state),
        ));
    }

    #[allow(unused_variables)]
    fn leave_instruction(
        &mut self,
        engine_state: &nu_protocol::engine::EngineState,
        ir_block: &nu_protocol::ir::IrBlock,
        instruction_index: usize,
        registers: &[nu_protocol::PipelineExecutionData],
        error: Option<&nu_protocol::ShellError>,
    ) {
        let nu_inst = NuInstance::new();
    }

    #[allow(unused_variables)]
    fn report(
        &self,
        engine_state: &nu_protocol::engine::EngineState,
        debugger_span: nu_protocol::Span,
    ) -> Result<nu_protocol::Value, nu_protocol::ShellError> {
        Ok(nu_protocol::Value::nothing(debugger_span))
    }
}

fn render_ir(
    engine_state: &nu_protocol::engine::EngineState,
    ir_block: &nu_protocol::ir::IrBlock,
    instruction_index: usize,
) -> String {
    ir_block
        .instructions
        .iter()
        .enumerate()
        .map(|i| {
            format!(
                "{point}{c}{idx}: {v}",
                c = if i.0 % 2 == 0 { "\x1b[32m" } else { "\x1b[33m" },
                point = if i.0 == instruction_index {
                    "\x1b[1;31m>"
                } else {
                    " "
                },
                idx = i.0,
                v = i.1.display(engine_state, &ir_block.data)
            )
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn render_registers(registers: &[nu_protocol::PipelineExecutionData]) -> String {
    registers
        .iter()
        .enumerate()
        .map(|register| -> String {
            let v = register.1.body.get_type();
            format!(
                "{c}{idx}: {v}",
                c = if register.0 % 2 == 0 {
                    "\x1b[32m"
                } else {
                    "\x1b[33m"
                },
                idx = register.0,
            )
        })
        .collect::<Vec<String>>()
        .join("\n")
}
fn render_env_vars(engine_state: &nu_protocol::engine::EngineState) -> String {
    engine_state
        .render_env_vars()
        .iter()
        .map(|(k, v)| -> String {
            let t = v.get_type();
            format!("{k}: {t}")
        })
        .collect::<Vec<String>>()
        .join("\n")
}
