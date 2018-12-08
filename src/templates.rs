use std::sync::Arc;
use std::convert::From;

use super::git;
use super::md2html;
use super::state;

use super::page::PageInfo;

#[derive(Debug)]
pub enum Error {
    Git(git::Error),
    Tera(tera::Error),
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
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Git(g) => write!(f, "{:?}", g),
            Error::Tera(t) => write!(f, "{:?}", t),
        }
    }
}
impl std::error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Error::Git(_) => "git error",
            Error::Tera(_) => "tera error",
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
        Ok(state
            .tera
            .render("index.html", &ctx)?)
    }
}

pub fn page() -> impl Fn(Arc<state::State>, String) -> Result<String> + Clone {
    move |state, mut fname| {
        fname.push_str(".md");
        let state = state.clone();
        let repo = git::get_repo("repo")?;
        let md = git::get_file(&fname, &repo)?;
        let page = md2html::parse(&md, &state.parse_context);
        let mut ctx = tera::Context::new();
        ctx.insert("page", &page);
        Ok(state
            .tera
            .render("page.html", &ctx)?)
    }
}

pub fn all() -> impl Fn(Arc<state::State>) -> Result<String> + Clone {
    move |state| {
        let repo = git::get_repo("repo")?;
        let list = git::list_files("", &repo)?
            .into_iter()
            .filter_map(|f| {
                if f.ends_with(".md") {
                    let permalink = format!("/page/{}", &f[0..f.len() - 3]);
                    let content = git::get_file(&f, &repo).expect("file is in tree but cannot be found");
                    let title = content.lines().next().expect("empty file").to_owned();
                    Some(PageInfo { title, permalink })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let mut ctx = tera::Context::new();
        ctx.insert("pages", &list);
        Ok(state
            .tera
            .render("pages.html", &ctx)?)
    }
}

pub fn edit() -> impl Fn(Arc<state::State>) -> Result<String> + Clone {
    move |state| {
        let ctx = tera::Context::new();
        Ok(state
            .tera
            .render("edit.html", &ctx)?)
    }
}
