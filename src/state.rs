use tera::compile_templates;
use tera::Tera;

use std::sync::Arc;

use super::md2html::ParseContext;

pub struct State {
    pub parse_context: ParseContext,
    pub tera: Tera,
}
impl State {
    pub fn create(templates_path: &str) -> Arc<State> {
        let tera = compile_templates!(templates_path);
        Arc::new(State {
            parse_context: ParseContext::new(),
            tera,
        })
    }
}
