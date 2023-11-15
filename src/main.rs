use axum::{
    routing::{get, post},
    Router,
};
use clap::Parser;
use include_dir::{include_dir, Dir};
use std::sync::{Arc, Mutex};
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

mod errors;
mod git;
mod md2html;
mod page;
mod routes;
mod templates;

pub static STATIC_ASSETS: Dir = include_dir!("static");
pub static CSS: &str = grass::include!("sass/wiki.scss");

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, env = "WIKIMARK_PORT", default_value = "3000")]
    port: u16,
    #[arg(short, long, env = "WIKIMARK_ADDR", default_value = "127.0.0.1")]
    address: String,
    #[arg(short, long, env = "WIKIMARK_REPO", default_value = "repo")]
    repo: String,
}

pub struct WikiState {
    pub repo: Mutex<git::Repo>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let args = Args::parse();
    tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_env("WIKIMARK_LOG"))
        .init();

    let state = WikiState {
        repo: Mutex::new(git::Repo::open(&args.repo)?),
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

    #[cfg(debug_assertions)]
    let app = app.layer(tower_livereload::LiveReloadLayer::new());

    axum::Server::bind(&format!("{}:{}", args.address, args.port).parse()?)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
