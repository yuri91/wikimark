use std::convert::Infallible;
use warp::path;
use warp::Filter;

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

async fn json<T: serde::Serialize>(r: T) -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::json(&r))
}

fn inject<T: Clone + Send>(obj: T) -> impl Filter<Extract = (T,), Error = Infallible> + Clone {
    warp::any().map(move || obj.clone())
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "wikimark=info");
    pretty_env_logger::init();

    let index = warp::get()
        .and(warp::path::end())
        .and(user_optional())
        .map(templates::Index::new)
        .and(inject("index.html"))
        .and_then(templates::render);

    let page = warp::get()
        .and(path!("page" / String))
        .and(user_optional())
        .map(templates::Page::new)
        .and(inject("page.html"))
        .and_then(templates::render);

    let css = warp::get()
        .and(path!("static" / "wiki.css"))
        .map(scss2css::getter("sass/wiki.scss"));

    let statics = warp::get()
        .and(path!("static"))
        .and(warp::fs::dir("static"));

    let md = warp::get()
        .and(path!("repo" / String))
        .and(inject("repo"))
        .and_then(|p, rp| {
            async move {
                git::page_getter(p, rp).map_err(|e| warp::reject::custom(templates::Error::from(e)))
            }
        })
        .and_then(json);

    let all = warp::get()
        .and(path!("all"))
        .and(user_optional())
        .map(templates::Pages::new)
        .and(inject("pages.html"))
        .and_then(templates::render);

    let edit = warp::get()
        .and(path!("edit"))
        .and(user_required())
        .map(templates::Edit::new)
        .and(inject("edit.html"))
        .and_then(templates::render);

    let commit = warp::post()
        .and(path!("commit"))
        .and(user_required())
        .and(warp::body::json::<git::CommitInfo>())
        .and(inject("repo"))
        .and_then(|a, ci, rp| {
            async move {
                git::page_committer(a, ci, rp)
                    .map_err(|e| warp::reject::custom(templates::Error::from(e)))
            }
        })
        .and_then(json);

    let api = index
        .or(page)
        .or(css)
        .or(statics)
        .or(md)
        .or(all)
        .or(edit)
        .or(commit)
        .recover(handle_rejection);

    let routes = api.with(warp::log("wikimark"));
    warp::serve(routes).run(([127, 0, 0, 1], 4391)).await;

    Ok(())
}
