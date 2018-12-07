use warp::Filter;
use warp::path;

mod git;
mod md2html;
mod page;
mod scss2css;
mod state;
mod templates;

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
        .and(inject_state.clone())
        .and(path!("all"))
        .map(templates::all());

    let edit = warp::get2()
        .and(inject_state)
        .and(path!("edit"))
        .map(templates::edit());

    let commit = warp::post2()
        .and(path!("commit"))
        .and(warp::body::json())
        .map(git::file_committer("repo"))
        .map(|r| {
            if r {
                warp::http::StatusCode::BAD_REQUEST
            } else {
                warp::http::StatusCode::CREATED
            }
        });

    let api = index
        .or(page)
        .or(css)
        .or(statics)
        .or(md)
        .or(all)
        .or(edit)
        .or(commit);

    let routes = api.with(warp::log("wikimark"));
    warp::serve(routes).run(([127, 0, 0, 1], 8000));
}
