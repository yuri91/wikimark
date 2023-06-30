use axum::{
    routing::{get, post},
    Router,
};
use std::sync::{Arc, Mutex};
use tower_http::trace::{self, TraceLayer};
use tracing::Level;
use include_dir::{Dir, include_dir};

mod errors;
mod git;
mod md2html;
mod page;
mod routes;
mod templates;

pub static STATIC_ASSETS: Dir = include_dir!("static");
pub static CSS: &str = grass::include!("sass/wiki.scss");

pub struct WikiState {
    pub repo: Mutex<git::Repo>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_env("WIKIMARK_LOG"))
        .init();

    let repo = std::env::var("WIKIMARK_REPO")
        .ok()
        .unwrap_or("repo".to_owned());
    let state = WikiState {
        repo: Mutex::new(git::Repo::open(&repo)?),
    };
    use routes::*;
    let app = Router::new()
        .route("/", get(index))
        .route("/static/wiki.css", get(css))
        .route("/static/*path", get(assets))
        .route("/page/:page", get(page))
        .route("/all", get(pages))
        .route("/repo/:page", get(md))
        .route("/edit", get(edit))
        .route("/commit", post(commit))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(Arc::new(state));

    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
