use warp::path;
use warp::Filter;

mod git;
mod md2html;
mod page;
mod scss2css;
mod state;
mod templates;

fn result_adapter<T: warp::Reply + 'static, E: std::fmt::Display>(
    r: Result<T, E>,
) -> Result<T, warp::Rejection> {
    match r {
        Ok(t) => Ok(t),
        Err(e) => {
            log::error!("{}", e);
            Err(warp::reject::custom(format!("{}", e)))
        }
    }
}
fn json<T: serde::Serialize, E>() -> impl Fn(Result<T, E>) -> Result<String, E> + Clone {
    move |r| r.map(|t| serde_json::to_string(&t).expect("cannot serialize"))
}

fn main() {
    std::env::set_var("RUST_LOG", "wikimark=info");
    pretty_env_logger::init();

    let state = state::State::create("templates/**/*");
    let inject_state = warp::any().map(move || state.clone());

    let index = warp::get2()
        .and(inject_state.clone())
        .and(warp::path::end())
        .map(templates::index())
        .and_then(result_adapter);

    let page = warp::get2()
        .and(inject_state.clone())
        .and(path!("page" / String))
        .map(templates::page())
        .and_then(result_adapter);

    let css = warp::get2()
        .and(path!("static" / "wiki.css"))
        .map(scss2css::getter("sass/wiki.scss"));

    let statics = warp::get2()
        .and(path!("static"))
        .and(warp::fs::dir("static"));

    let md = warp::get2()
        .and(path!("repo" / String))
        .map(git::page_getter("repo"))
        .map(json())
        .and_then(result_adapter);

    let all = warp::get2()
        .and(inject_state.clone())
        .and(path!("all"))
        .map(templates::all())
        .and_then(result_adapter);

    let edit = warp::get2()
        .and(inject_state)
        .and(path!("edit"))
        .map(templates::edit())
        .and_then(result_adapter);

    let commit = warp::post2()
        .and(path!("commit"))
        .and(warp::body::json())
        .map(git::page_committer("repo"))
        .and_then(result_adapter);

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
