use std::convert::From;
use std::sync::Arc;
use serde::Serialize;
use serde_derive::Serialize;

use warp::Filter;

use once_cell::sync::Lazy;

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
impl warp::reject::Reject for Error {}

static TERA: Lazy<tera::Tera> = Lazy::new(|| {
    tera::Tera::new("templates/**/*").expect("failed startup template parsing")
});

pub async fn render<T: Serialize>(params: Result<T, Error>, template: &str) -> Result<String, warp::Rejection> {
    let result = || -> Result<String, Error> {
        let params = params?;
        let ctx = tera::Context::from_serialize(params)?;
        Ok(TERA.render(template, &ctx)?)
    };
    result().map_err(|e| warp::reject::custom(Error::from(e)))
}

#[derive(Serialize)]
pub struct Index {
    user: Option<String>,
}
impl Index {
    pub fn new(user: Option<String>) -> Result<Index, Error> {
        Ok(Index {
            user,
        })
    }
}

#[derive(Serialize)]
pub struct Page {
    page: super::page::Page,
    user: Option<String>,
}
impl Page {
    pub fn new(mut fname: String, user: Option<String>) -> Result<Page, Error> {
        let metaname = format!("meta/{}.json", fname);
        fname.push_str(".md");
        let repo = git::get_repo("repo")?;
        let md = git::get_file(&fname, &repo)?;
        let meta = serde_json::from_str(&git::get_file(&metaname, &repo)?)?;
        let page = md2html::parse(&md, &meta);
        Ok(Page {
            page,
            user,
        })
    }
}

#[derive(Serialize)]
pub struct Pages {
    pages: Vec<super::page::Metadata>,
    user: Option<String>,
}
impl Pages {
    pub fn new(user: Option<String>) -> Result<Pages, Error> {
        let repo = git::get_repo("repo")?;
        let pages = git::list_files("", &repo)?;
        Ok(Pages {
            pages,
            user,
        })
    }
}

#[derive(Serialize)]
pub struct Edit {
    user: String,
}
impl Edit {
    pub fn new(user: String) -> Result<Edit, Error> {
        Ok(Edit {
            user,
        })
    }
}
