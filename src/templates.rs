use std::sync::Arc;

use super::git;
use super::md2html;
use super::state;

use super::page::PageInfo;

pub fn index() -> impl Fn(Arc<state::State>) -> String + Clone {
    move |state| {
        let state = state.clone();
        let ctx = tera::Context::new();
        state
            .tera
            .render("index.html", &ctx)
            .expect("template rendering failed")
    }
}

pub fn page() -> impl Fn(Arc<state::State>, String) -> String + Clone {
    move |state, mut fname| {
        fname.push_str(".md");
        let state = state.clone();
        let repo = git::get_repo("repo");
        let md = git::get_file(&fname, &repo);
        let page = md2html::parse(&md, &state.parse_context);
        let mut ctx = tera::Context::new();
        ctx.insert("page", &page);
        state
            .tera
            .render("page.html", &ctx)
            .expect("template rendering failed")
    }
}

pub fn all() -> impl Fn(Arc<state::State>) -> String + Clone {
    move |state| {
        let repo = git::get_repo("repo");
        let list = git::list_files("", &repo)
            .into_iter()
            .filter_map(|f| {
                if f.ends_with(".md") {
                    let permalink = format!("/page/{}", &f[0..f.len() - 3]);
                    let content = git::get_file(&f, &repo);
                    let title = content.lines().next().expect("empty file").to_owned();
                    Some(PageInfo { title, permalink })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let mut ctx = tera::Context::new();
        ctx.insert("pages", &list);
        state
            .tera
            .render("pages.html", &ctx)
            .expect("template error")
    }
}

pub fn edit() -> impl Fn(Arc<state::State>) -> String + Clone {
    move |state| {
        let ctx = tera::Context::new();
        state
            .tera
            .render("edit.html", &ctx)
            .expect("template error")
    }
}
