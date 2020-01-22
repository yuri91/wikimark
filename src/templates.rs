use once_cell::sync::Lazy;

use super::errors::Error;
use super::git;
use super::md2html;

static TERA: Lazy<tera::Tera> =
    Lazy::new(|| tera::Tera::new("templates/**/*").expect("failed startup template parsing"));

pub fn index(user: Option<String>) -> Result<String, Error> {
    let mut ctx = tera::Context::new();

    ctx.insert("user", &user);

    Ok(TERA.render("index.html", &ctx)?)
}

pub fn page(user: Option<String>, mut fname: String) -> Result<String, Error> {
    let metaname = format!("meta/{}.json", fname);
    fname.push_str(".md");
    let repo = git::get_repo("repo")?;
    let md = git::get_file(&fname, &repo)?;
    let meta = serde_json::from_str(&git::get_file(&metaname, &repo)?)?;
    let page = md2html::parse(&md, &meta);

    let mut ctx = tera::Context::new();

    ctx.insert("user", &user);
    ctx.insert("page", &page);

    Ok(TERA.render("page.html", &ctx)?)
}

pub fn pages(user: Option<String>) -> Result<String, Error> {
    let repo = git::get_repo("repo")?;
    let pages = git::list_files("", &repo)?;

    let mut ctx = tera::Context::new();

    ctx.insert("user", &user);
    ctx.insert("pages", &pages);

    Ok(TERA.render("pages.html", &ctx)?)
}

pub fn edit(user: String) -> Result<String, Error> {
    let mut ctx = tera::Context::new();

    ctx.insert("user", &user);

    Ok(TERA.render("edit.html", &ctx)?)
}
