use super::md2html::ParseContext;
use anyhow::{Context, Result};
use std::sync::Arc;
use tera::Tera;

pub struct State {
    pub parse_context: ParseContext,
    pub tera: Tera,
}
impl State {
    pub fn create(templates_path: &str) -> Result<Arc<State>> {
        let tera = Tera::new(templates_path).context("failed startup template parsing")?;
        Ok(Arc::new(State {
            parse_context: ParseContext::new(),
            tera,
        }))
    }
}
