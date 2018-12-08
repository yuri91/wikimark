use std::convert::From;
use std::sync::Arc;

use super::git;
use super::md2html;
use super::state;

#[derive(Debug)]
pub enum Error {
    Git(git::Error),
    Tera(tera::Error),
    Json(serde_json::Error),
}
impl From<git::Error> for Error {
    fn from(err: git::Error) -> Self {
        Error::Git(err)
    }
}
impl From<tera::Error> for Error {
    fn from(err: tera::Error) -> Self {
        Error::Tera(err)
    }
}
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err)
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Git(g) => write!(f, "{:?}", g),
            Error::Tera(t) => write!(f, "{:?}", t),
            Error::Json(j) => write!(f, "{:?}", j),
        }
    }
}
impl std::error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Error::Git(_) => "git error",
            Error::Tera(_) => "tera error",
            Error::Json(_) => "json error",
        }
    }
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn index() -> impl Fn(Arc<state::State>) -> Result<String> + Clone {
    move |state| {
        let state = state.clone();
        let ctx = tera::Context::new();
        Ok(state.tera.render("index.html", &ctx)?)
    }
}

pub fn page() -> impl Fn(Arc<state::State>, String) -> Result<String> + Clone {
    move |state, mut fname| {
        let metaname = format!("meta/{}.json", fname);
        fname.push_str(".md");
        let state = state.clone();
        let repo = git::get_repo("repo")?;
        let md = git::get_file(&fname, &repo)?;
        let meta = serde_json::from_str(&git::get_file(&metaname, &repo)?)?;
        let page = md2html::parse(&md, &meta, &state.parse_context);
        let mut ctx = tera::Context::new();
        ctx.insert("page", &page);
        Ok(state.tera.render("page.html", &ctx)?)
    }
}

pub fn all() -> impl Fn(Arc<state::State>) -> Result<String> + Clone {
    move |state| {
        let repo = git::get_repo("repo")?;
        let list = git::list_files("", &repo)?;
        let mut ctx = tera::Context::new();
        ctx.insert("pages", &list);
        Ok(state.tera.render("pages.html", &ctx)?)
    }
}

pub fn edit() -> impl Fn(Arc<state::State>) -> Result<String> + Clone {
    move |state| {
        let ctx = tera::Context::new();
        Ok(state.tera.render("edit.html", &ctx)?)
    }
}
