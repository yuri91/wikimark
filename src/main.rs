use std::convert::Infallible;
use warp::path;
use warp::Filter;

mod errors;
mod git;
mod md2html;
mod page;
mod scss2css;
mod templates;

#[derive(Debug)]
struct Unauthorized;
impl warp::reject::Reject for Unauthorized {}

async fn handle_rejection(_err: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::with_status(
        "ops",
        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
    ))
}

fn user_optional() -> impl Filter<Extract = (Option<String>,), Error = Infallible> + Clone {
    let default_user = std::env::var("WIKIMARK_DEFAULT_USER").ok();

    warp::header::<String>("X-Forwarded-User")
        .map(Some)
        .or(warp::any().map(move || default_user.clone()))
        .unify()
}

fn user_required() -> impl Filter<Extract = (String,), Error = warp::Rejection> + Clone {
    user_optional()
        .and_then(|u: Option<String>| async { u.ok_or_else(|| warp::reject::custom(Unauthorized)) })
}

async fn result_adapter<R: warp::Reply, E: warp::reject::Reject>(
    res: Result<R, E>,
) -> Result<warp::reply::Response, warp::Rejection> {
    res.map(warp::Reply::into_response)
        .map_err(warp::reject::custom)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "wikimark=info");
    pretty_env_logger::init();

    let index = warp::get()
        .and(user_optional())
        .and(warp::path::end())
        .map(templates::index)
        .and_then(result_adapter);

    let page = warp::get()
        .and(user_optional())
        .and(path!("page" / String))
        .map(templates::page)
        .and_then(result_adapter);

    let css = warp::get()
        .and(path!("static" / "wiki.css"))
        .map(scss2css::getter("sass/wiki.scss"));

    let statics = warp::get()
        .and(path!("static"))
        .and(warp::fs::dir("static"));

    let md = warp::get()
        .and(path!("repo" / String))
        .map(|p| {
            git::page_getter(p, "repo")
                .map(|p| warp::reply::json(&p))
                .map_err(errors::Error::from)
        })
        .and_then(result_adapter);

    let pages = warp::get()
        .and(user_optional())
        .and(path!("all"))
        .map(templates::pages)
        .and_then(result_adapter);

    let edit = warp::get()
        .and(user_required())
        .and(path!("edit"))
        .map(templates::edit)
        .and_then(result_adapter);

    let commit = warp::post()
        .and(user_required())
        .and(path!("commit"))
        .and(warp::body::json::<git::CommitInfo>())
        .map(|user, commit| git::page_committer(user, commit, "repo").map_err(errors::Error::from))
        .and_then(result_adapter);

    let api = index
        .or(page)
        .or(css)
        .or(statics)
        .or(md)
        .or(pages)
        .or(edit)
        .or(commit)
        .recover(handle_rejection);

    let routes = api.with(warp::log("wikimark"));
    warp::serve(routes).run(([127, 0, 0, 1], 4391)).await;

    Ok(())
}
