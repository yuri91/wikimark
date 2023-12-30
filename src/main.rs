use axum::{
    routing::{get, post},
    Router,
};
use clap::Parser;
use include_dir::{include_dir, Dir};
use minijinja::Environment;
use std::sync::Arc;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

mod errors;
mod git;
mod md2html;
mod page;
mod routes;

pub static STATIC_ASSETS: Dir = include_dir!("static");
pub static TEMPLATES: Dir = include_dir!("templates");
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
    #[arg(short, long, env = "WIKIMARK_COMMIT_URL_PREFIX", default_value = "")]
    commit_url_prefix: String,
}

pub struct WikiState {
    pub repo: git::Repo,
    pub commit_url_prefix: String,
    pub env: Environment<'static>,
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

    let mut env = Environment::new();
    env.set_loader(|name| {
        Ok(TEMPLATES.get_file(name).map(|f| f.contents_utf8().unwrap().to_owned()))
    });
    let state = WikiState {
        repo: git::Repo::open(&args.repo)?,
        commit_url_prefix: args.commit_url_prefix,
        env,
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
        .route("/changelog", get(changelog))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(
                    trace::DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(tower_http::LatencyUnit::Micros),
                ),
        )
        .with_state(Arc::new(state));

    #[cfg(debug_assertions)]
    let app = app.layer(tower_livereload::LiveReloadLayer::new().request_predicate(
        |r: &axum::http::Request<axum::body::Body>| r.headers().get("HX-Request").is_none(),
    ));

    axum::Server::bind(&format!("{}:{}", args.address, args.port).parse()?)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
