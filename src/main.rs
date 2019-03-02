use ::actix::prelude::*;
use actix_web::*;
use dotenv::dotenv;
use futures::Future;

use std::sync::Arc;

mod actors;
mod extractors;
mod git;
mod graphql;

use extractors::Identity;

struct AppState {
    executor:
        Addr<actors::GraphQLExecutor<std::sync::Arc<git::Repo>, graphql::Query, graphql::Mutation>>,
}

fn graphiql(_req: &HttpRequest<AppState>) -> Result<HttpResponse, Error> {
    let html = juniper::http::graphiql::graphiql_source("http://127.0.0.1:8888/graphql");
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

fn graphql(
    st: State<AppState>,
    data: Json<actors::GraphQLData>,
    identity: Identity,
) -> FutureResponse<HttpResponse> {
    let msg = actors::GraphQLDataMessage {
        message_context: identity.name,
        data: data.0,
    };
    st.executor
        .send(msg)
        .from_err()
        .and_then(|res| match res {
            Ok(user) => Ok(HttpResponse::Ok()
                .content_type("application/json")
                .body(user)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

fn main() {
    std::env::set_var("RUST_LOG", "actix_web=info,feeder=info,server=info");
    dotenv().ok();
    env_logger::init();

    let sys = actix::System::new("wikimark");

    let graphql_addr = SyncArbiter::start(4, move || actors::GraphQLExecutor {
        schema: Arc::new(graphql::create_schema()),
        executor_context: std::sync::Arc::new(git::Repo::new("repo").unwrap()),
    });

    server::new(move || {
        App::with_state(AppState {
            executor: graphql_addr.clone(),
        })
        .middleware(middleware::Logger::default())
        .configure(|app| {
            middleware::cors::Cors::for_app(app)
                .allowed_origin("http://localhost:8888")
                .supports_credentials()
                .resource("/graphql", |r| r.method(http::Method::POST).with(graphql))
                .resource("/graphiql", |r| r.method(http::Method::GET).h(graphiql))
                .register()
        })
    })
    .bind("127.0.0.1:8888")
    .unwrap()
    .start();

    println!("Started http server: 127.0.0.1:8888");
    let _ = sys.run();
}
