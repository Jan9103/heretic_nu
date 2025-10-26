pub mod ansi;
pub mod commands;
pub mod debug_x;
#[cfg(feature = "heretic_step_debug")]
pub mod step_debug;

use nu_engine::eval_block_with_early_return;
use nu_protocol::engine::{EngineState, Stack, StateWorkingSet};
use nu_protocol::{PipelineData, ShellError, Span, Value};
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;

pub struct NuInstance {
    pub engine_state: EngineState,
    pub stack: Stack,
}

impl NuInstance {
    pub fn add_var(
        &mut self,
        value: nu_protocol::Value,
    ) -> nu_protocol::Id<nu_protocol::marker::Var> {
        let current_max_id = self
            .stack
            .vars
            .iter()
            .map(|i| i.0.get())
            .max()
            .unwrap_or(100);
        let vid = nu_protocol::Id::new(current_max_id + 1);
        self.stack.add_var(vid, value);
        vid
    }

    #[cfg(feature = "nu_std")]
    #[allow(clippy::result_large_err)]
    pub fn add_stdlib(&mut self) -> Result<(), ShellError> {
        match nu_std::load_standard_library(&mut self.engine_state) {
            Ok(_) => Ok(()),
            Err(e) => Err(ShellError::NushellFailed {
                msg: format!("Failed to load nu-std: {e}"),
            }),
        }
    }

    #[allow(clippy::result_large_err)]
    pub fn new() -> Result<Self, ShellError> {
        let mut engine_state = nu_cmd_lang::create_default_context();
        engine_state = nu_command::add_shell_command_context(engine_state);
        if cfg!(feature = "nu_cmd_extra") {
            engine_state = nu_cmd_extra::add_extra_command_context(engine_state);
        }
        if cfg!(feature = "nu_explore") {
            engine_state = nu_explore::add_explore_context(engine_state);
        }
        let init_cwd = std::env::current_dir().expect("Failed to get CWD");
        nu_cli::gather_parent_env_vars(&mut engine_state, init_cwd.as_ref());

        engine_state.generate_nu_constant();

        engine_state.add_env_var(
            "CMD_DURATION_MS".into(),
            Value::string("0823", Span::unknown()), // compatibility for prompts
        );

        let mut res = Self {
            engine_state,
            stack: Stack::new(),
        };

        res.stack.set_last_exit_code(0, Span::unknown());

        res.append_commands(vec![
            // things not in the standard scope for some reason
            Box::new(nu_cli::Print),
            Box::new(nu_cli::NuHighlight),
            // custom commands
            Box::new(commands::evil::Evil),
            #[cfg(feature = "heretic_const_evil")]
            Box::new(commands::evil::ConstEvil),
            Box::new(commands::debug::HereticDebug),
            #[cfg(feature = "heretic_test")]
            Box::new(commands::run_tests::HereticTestsRun),
            // overrides
            Box::new(commands::version::HereticVersion),
        ])?;

        Ok(res)
    }

    #[allow(clippy::result_large_err)]
    pub fn append_commands(
        &mut self,
        commands: Vec<Box<dyn nu_protocol::engine::Command>>,
    ) -> Result<(), ShellError> {
        self.engine_state.merge_delta({
            let mut working_set = StateWorkingSet::new(&self.engine_state);
            for command in commands.into_iter() {
                working_set.add_decl(command);
            }
            working_set.render()
        })?;
        Ok(())
    }

    #[allow(clippy::result_large_err)]
    pub fn compile(
        &mut self,
        code: &str,
    ) -> Result<std::sync::Arc<nu_protocol::ast::Block>, ShellError> {
        let mut working_set = StateWorkingSet::new(&self.engine_state);
        let mut block: std::sync::Arc<nu_protocol::ast::Block> =
            nu_parser::parse(&mut working_set, None, code.as_bytes(), false);
        if block.ir_block.is_none() {
            let block_mut = Arc::make_mut(&mut block);
            match nu_engine::compile(&working_set, block_mut) {
                Ok(ir_block) => {
                    block_mut.ir_block = Some(ir_block);
                }
                Err(err) => {
                    let msg = format!("Compiling IR failed: {err:?}");
                    let span: Option<Span> = match err {
                        nu_protocol::CompileError::DataOverflow { block_span } |
                        nu_protocol::CompileError::FileOverflow { block_span } |
                        nu_protocol::CompileError::IncoherentLoopState { block_span } |
                        nu_protocol::CompileError::RegisterOverflow { block_span } => block_span,

                        nu_protocol::CompileError::RegisterUninitialized { .. } => None,

                        nu_protocol::CompileError::SetBranchTargetOfNonBranchInstruction { span, .. } |
                        nu_protocol::CompileError::RunExternalNotFound { span } |
                        nu_protocol::CompileError::AssignmentRequiresVar { span } |
                        nu_protocol::CompileError::AssignmentRequiresMutableVar { span } |
                        nu_protocol::CompileError::AutomaticEnvVarSetManually { span, .. } |
                        nu_protocol::CompileError::CannotReplaceEnv { span } |
                        nu_protocol::CompileError::UnexpectedExpression { span, .. } |
                        nu_protocol::CompileError::MissingRequiredDeclaration { span, .. } |
                        nu_protocol::CompileError::InvalidLiteral { span, .. } |
                        nu_protocol::CompileError::Garbage { span } |
                        nu_protocol::CompileError::UnsupportedOperatorExpression { span } |
                        nu_protocol::CompileError::AccessEnvByInt { span } |
                        nu_protocol::CompileError::InvalidKeywordCall { span, .. } |
                        nu_protocol::CompileError::InvalidRedirectMode { span } |
                        nu_protocol::CompileError::RegisterUninitializedWhilePushingInstruction { span, .. } => Some(span),

                        nu_protocol::CompileError::NotInATry { span, .. } |
                        nu_protocol::CompileError::NotInALoop { span, .. } |
                        nu_protocol::CompileError::UndefinedLabel { span, .. } => span,
                    };
                    working_set.compile_errors.push(err);
                    return Err(if let Some(span) = span {
                        let txt = String::from_utf8_lossy(working_set.get_span_contents(span));
                        println!("{txt}");

                        ShellError::NushellFailedSpanned {
                            msg,
                            label: format!("here: {txt}"),
                            span,
                        }
                    } else {
                        ShellError::NushellFailed { msg }
                    });
                }
            };
        }
        self.engine_state.merge_delta(working_set.render())?;
        Ok(block)
    }

    #[allow(clippy::result_large_err)]
    pub fn exec(
        &mut self,
        line: &str,
        pipeline_data: Option<PipelineData>,
    ) -> Result<PipelineData, ShellError> {
        let block = self.compile(line)?;

        match eval_block_with_early_return::<nu_protocol::debugger::WithDebug>(
            &self.engine_state,
            &mut self.stack,
            &block,
            pipeline_data.unwrap_or(PipelineData::Empty),
        ) {
            Ok(res) => {
                match nu_protocol::process::check_exit_status_future(res.exit) {
                    Ok(_) => (),
                    Err(err) => {
                        return Err(err);
                    }
                }
                Ok(res.body)
            }
            Err(err) => Err(err),
        }
    }

    #[allow(clippy::result_large_err)]
    pub fn run_file(
        &mut self,
        filepath: String,
        args: &[String],
        input: Option<PipelineData>,
    ) -> Result<(), ShellError> {
        nu_cli::evaluate_file(
            filepath,
            args,
            &mut self.engine_state,
            &mut self.stack,
            input.unwrap_or(PipelineData::Empty),
        )
    }

    pub fn render(&mut self, result: Result<PipelineData, ShellError>) {
        match result {
            Ok(pipeline_data) => match pipeline_data.into_value(Span::unknown()) {
                Ok(value) => match value {
                    Value::Nothing { .. } => println!(),
                    _ => match self.exec("print", Some(PipelineData::Value(value, None))) {
                        Ok(_) => (),
                        Err(e) => {
                            eprintln!("RENDER FAILED:");
                            self.render(Err(e));
                        }
                    },
                },
                Err(e) => eprintln!("Conversion-Error (into_value): {:?}", e),
            },
            Err(render_error) => {
                eprintln!("Nu-Error: {:?}", render_error);
                #[allow(clippy::single_match)]
                match render_error {
                    ShellError::VariableNotFoundAtRuntime { span } => {
                        let span_contents = self.engine_state.get_span_contents(span);
                        if let Ok(a) = std::str::from_utf8(span_contents) {
                            eprintln!("Span contents: {a}");
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    pub fn load_default_config(&mut self) {
        self.exec(nu_utils::utils::ConfigFileKind::Env.default(), None)
            .expect("failed to run upstream default_env.nu");
        self.exec(nu_utils::utils::ConfigFileKind::Config.default(), None)
            .expect("failed to run upstream default_config.nu");

        self.exec(include_str!("default_config.nu"), None)
            .expect("Default config is invalid");
    }

    //#[allow(clippy::result_large_err)]
    //pub fn load_base_settings(&mut self) -> Result<(), ShellError> {
    //    if let Some(home_dir) = std::env::home_dir() {
    //        let settings_file = home_dir
    //            .join(".config")
    //            .join("heretic_nu")
    //            .join("settings.nuon");
    //        if settings_file.is_file() {
    //            let mut unparsed_nuon = String::new();
    //            std::fs::File::open(settings_file)
    //                .expect("File not found.")
    //                .read_to_string(&mut unparsed_nuon)
    //                .expect("Failed to read config.nu file (filesystem IO)");
    //            if let Value::Record {
    //                val: main_record, ..
    //            } = nuon::from_nuon(&unparsed_nuon, None)?
    //            {
    //                // main_record.get("lib_dirs")
    //            } else {
    //                return Err(ShellError::IncorrectValue {
    //                    msg: "settings.nuon is not a record".into(),
    //                    val_span: Span::unknown(),
    //                    call_span: Span::unknown(),
    //                });
    //            }
    //            // self.exec(&script, None)?;
    //        };
    //    }
    //    Ok(())
    //}

    #[allow(clippy::result_large_err)]
    pub fn load_all_configs(&mut self) -> Result<(), ShellError> {
        self.load_default_config();

        if let Some(home_dir) = std::env::home_dir() {
            let config_file = home_dir
                .join(".config")
                .join("heretic_nu")
                .join("config.nu");
            if config_file.is_file() {
                let mut script = String::new();
                std::fs::File::open(config_file)
                    .expect("File not found.")
                    .read_to_string(&mut script)
                    .expect("Failed to read config.nu file (filesystem IO)");
                self.exec(&script, None)?;
            }
            let ev = self
                .engine_state
                .get_env_var("heretic_nu_autoload_dirs")
                .cloned();
            match ev {
                Some(Value::List { vals, .. }) => {
                    for val in vals {
                        match val {
                            Value::String { val, .. } => {
                                let fp = PathBuf::from(val);
                                if fp.is_dir() {
                                    for f in std::fs::read_dir(fp)
                                        .expect("Failed to read autoload-dir contents")
                                    {
                                        let f: std::fs::DirEntry =
                                            f.expect("Failed to read autoload-dir contents");
                                        let p = f.path();
                                        if p.is_file()
                                            && p.extension() == Some(std::ffi::OsStr::new("nu"))
                                        {
                                            let mut script = String::new();
                                            std::fs::File::open(p)
                                                .expect("File not found.")
                                                .read_to_string(&mut script).expect("Failed to read autoload-dir file (filesystem IO)");
                                            self.exec(&script, None)?;
                                        }
                                    }
                                }
                            }
                            _ => {
                                return Err(ShellError::TypeMismatch {
                                    err_message:
                                        "$env.heretic_nu_autoload_dirs has to be a list<path>"
                                            .into(),
                                    span: Span::unknown(),
                                });
                            }
                        }
                    }
                }
                Some(_) => {
                    return Err(ShellError::TypeMismatch {
                        err_message: "$env.heretic_nu_autoload_dirs has to be a list<path>".into(),
                        span: Span::unknown(),
                    });
                }
                None => {}
            }
        }
        Ok(())
    }

    pub fn set_exitcode(&mut self, code: i32, span: Span) {
        self.stack.set_last_exit_code(code, span);
    }
}
