use super::git;
use super::md2html;
use super::page;

use askama::Template;

type Result<T> = std::result::Result<T, anyhow::Error>;

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index<'a> {
    user: Option<&'a str>,
}
impl<'a> Index<'a> {
    pub fn new(user: Option<&'a str>) -> Index<'a> {
        Index { user }
    }
}

#[derive(Template)]
#[template(path = "page.html")]
pub struct Page<'a> {
    user: Option<&'a str>,
    page: page::Page,
}

impl<'a> Page<'a> {
    pub fn new(user: Option<&'a str>, fname: &str, repo: &git::Repo) -> Result<Page<'a>> {
        let md = repo.page_getter(fname)?;
        let page = if md.meta.private && user.is_none() {
            md2html::parse("Access denied", &md.meta)
        } else {
            md2html::parse(&md.content, &md.meta)
        };
        Ok(Page { user, page })
    }
}

#[derive(Template)]
#[template(path = "pages.html")]
pub struct Pages<'a> {
    user: Option<&'a str>,
    pages: Vec<page::Metadata>,
}

impl<'a> Pages<'a> {
    pub fn new(user: Option<&'a str>, repo: &git::Repo) -> Result<Pages<'a>> {
        let pages = repo.list_files("")?;
        Ok(Pages { user, pages })
    }
}

#[derive(Template)]
#[template(path = "edit.html")]
pub struct Edit<'a> {
    user: Option<&'a str>,
}

impl<'a> Edit<'a> {
    pub fn new(user: &'a str) -> Edit<'a> {
        Edit { user: Some(user) }
    }
}
