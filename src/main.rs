extern crate pulldown_cmark;
extern crate syntect;
#[macro_use]
extern crate tera;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate warp;
extern crate sass_rs;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;

use warp::Filter;

mod md2html;
mod scss2css;

fn main() {
    std::env::set_var("RUST_LOG", "wikimark=info");
    pretty_env_logger::init();

    let page = warp::get2()
        .and(path!("page" / String))
        .map(md2html::renderer("templates/**/*"));

    let css = warp::get2()
        .and(path!("static" / "wiki.css"))
        .map(scss2css::getter("sass/wiki.scss"));

    let statics = warp::get2()
        .and(path!("static"))
        .and(warp::fs::dir("static"));

    let api = page.or(css).or(statics);

    let routes = api.with(warp::log("wikimark"));
    warp::serve(routes).run(([127, 0, 0, 1], 8000));
}
