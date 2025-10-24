use std::collections::HashMap;

use nu_engine::command_prelude::*;
use nu_protocol::{ast, debugger::WithoutDebug, PipelineData};

use crate::NuInstance;

#[derive(Clone)]
pub struct HereticTestsRun;

impl Command for HereticTestsRun {
    fn name(&self) -> &str {
        "heretic tests run"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_type(Type::Nothing, Type::Nothing)
            .category(Category::Debug)
    }

    fn description(&self) -> &str {
        "run tests\n\
         \n\
         PART OF HERETIC-NU"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        for (_decl_name, decl_id) in engine_state.get_decls_sorted(false) {
            for_decl(decl_id, engine_state, stack, call.span())?;
        }
        Ok(PipelineData::Empty)
    }
}

#[allow(clippy::result_large_err)]
fn for_decl(
    decl_id: nu_protocol::Id<nu_protocol::marker::Decl>,
    engine_state: &EngineState,
    stack: &Stack,
    span: Span,
) -> Result<(), ShellError> {
    let decl = engine_state.get_decl(decl_id);
    if !decl.is_custom() {
        return Ok(());
    }

    let mut is_test: bool = false;
    let mut parameterizibla_values: Vec<Vec<Value>> = Vec::new();
    let mut parameterizibla_targets: Vec<String> = Vec::new();

    for line in decl
        .description()
        .lines()
        .map(|i| i.trim())
        .filter(|i| i.starts_with('['))
    {
        if let Some(s) = line.split_once(']') {
            #[allow(clippy::collapsible_match, clippy::single_match)]
            match s {
                ("[test", "") => {
                    is_test = true;
                }
                ("[test_param", v) => {
                    let (param, value) = match v.split_once('=') {
                        Some(v) => v,
                        None => {
                            return Err(ShellError::IncorrectValue { msg: format!("[test_param] of a test does not have a equal-sign. function name: {}", decl.name()), val_span: Span::unknown(), call_span: span });
                        }
                    };
                    let param = param.trim().to_string();
                    let value = match nuon::from_nuon(value, None) {
                        Ok(v) => v,
                        Err(_) => {
                            return Err(ShellError::IncorrectValue {
                                msg: format!(
                                    "[test_param] is invalid nuon. function name: {}",
                                    decl.name()
                                ),
                                val_span: Span::unknown(),
                                call_span: span,
                            });
                        }
                    };
                    match value {
                        Value::List { vals, .. } => {
                            parameterizibla_targets.push(param);
                            parameterizibla_values.push(vals);
                        }
                        _ => {
                            return Err(ShellError::IncorrectValue {
                                msg: format!(
                                    "[test_param] is not a list. function name: {}",
                                    decl.name()
                                ),
                                val_span: Span::unknown(),
                                call_span: span,
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if is_test {
        if parameterizibla_values.is_empty() {
            let mut tni = NuInstance {
                engine_state: engine_state.clone(),
                stack: stack.clone(),
            };
            nu_engine::eval_call::<WithoutDebug>(
                &tni.engine_state,
                &mut tni.stack,
                &ast::Call {
                    decl_id,
                    head: span,
                    arguments: vec![],
                    parser_info: HashMap::new(),
                },
                PipelineData::Empty,
            )?;
        } else {
            for param_combination in CartesianProduct::new(parameterizibla_values) {
                let mut tni = NuInstance {
                    engine_state: engine_state.clone(),
                    stack: stack.clone(),
                };
                let mut arguments = Vec::new();
                for (param_idx, param_value) in param_combination.into_iter().enumerate() {
                    // dbg!((&param_idx, &param_value));
                    let vt = param_value.get_type();
                    let vid = tni.add_var(param_value);
                    // dbg!(&vid);
                    let target: &str = &parameterizibla_targets[param_idx];
                    arguments.push(ast::Argument::Named((
                        Spanned {
                            item: String::from(target),
                            span: Span::unknown(),
                        },
                        None,
                        Some(ast::Expression::new_unknown(ast::Expr::Var(vid), span, vt)),
                    )));
                }
                nu_engine::eval_call::<WithoutDebug>(
                    &tni.engine_state,
                    &mut tni.stack,
                    &ast::Call {
                        decl_id,
                        head: span,
                        arguments,
                        parser_info: HashMap::new(),
                    },
                    PipelineData::Empty,
                )?;
            }
        }
    }

    Ok(())
}

struct CartesianProduct<I>
where
    I: Clone,
{
    vecs: Vec<Vec<I>>,
    idxs: Vec<usize>,
}
impl<I> CartesianProduct<I>
where
    I: Clone,
{
    pub fn new(vecs: Vec<Vec<I>>) -> Self {
        let mut idxs = Vec::new();
        while idxs.len() < vecs.len() {
            idxs.push(0);
        }
        Self { vecs, idxs }
    }
}

impl<I> Iterator for CartesianProduct<I>
where
    I: Clone,
{
    type Item = Vec<I>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idxs[self.idxs.len() - 1] >= self.vecs[self.idxs.len() - 1].len() {
            return None;
        }
        for vec_no in 0..=(self.idxs.len() - 1) {
            if self.idxs[vec_no] < (self.vecs[vec_no].len()) {
                let res = (0..=(self.idxs.len() - 1))
                    .map(|vec_idx| -> I { self.vecs[vec_idx][self.idxs[vec_idx]].clone() })
                    .collect::<Vec<I>>();
                self.idxs[0] += 1;
                return Some(res);
            } else {
                self.idxs[vec_no] = 0;
                if vec_no != self.idxs.len() - 1 {
                    self.idxs[vec_no + 1] += 1;
                }
            }
        }
        None
    }
}
