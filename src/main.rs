extern crate pulldown_cmark;
extern crate syntect;
#[macro_use]
extern crate tera;
#[macro_use]
extern crate serde_derive;
extern crate serde;
#[macro_use]
extern crate warp;
extern crate sass_rs;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;
extern crate git2;
extern crate slab_tree;

use warp::Filter;

mod md2html;
mod scss2css;
mod git;
mod state;
mod templates;
mod page;

fn main() {
    std::env::set_var("RUST_LOG", "wikimark=info");
    pretty_env_logger::init();

    let state = state::State::create("templates/**/*");
    let inject_state = warp::any().map(move || state.clone());

    let index = warp::get2()
        .and(inject_state.clone())
        .and(warp::path::end())
        .map(templates::index());

    let page = warp::get2()
        .and(inject_state.clone())
        .and(path!("page" / String))
        .map(templates::page());

    let css = warp::get2()
        .and(path!("static" / "wiki.css"))
        .map(scss2css::getter("sass/wiki.scss"));

    let statics = warp::get2()
        .and(path!("static"))
        .and(warp::fs::dir("static"));

    let md = warp::get2()
        .and(path!("repo" / String))
        .map(git::file_getter("repo"));

    let all = warp::get2()
        .and(inject_state)
        .and(path!("all"))
        .map(templates::all());

    let api = index
        .or(page)
        .or(css)
        .or(statics)
        .or(md)
        .or(all);

    let routes = api.with(warp::log("wikimark"));
    warp::serve(routes).run(([127, 0, 0, 1], 8000));
}
