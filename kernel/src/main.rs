use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};

use actix::*;
use actix_files::{Files, NamedFile};
use actix_web::{
    middleware::Logger, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_actors::ws;

mod kernel;
mod chain;

/// Entry point for our websocket route
async fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
    ccl: web::Data<Addr<chain::ChainClient>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        kernel::Kernel {
            ipfs_url: String::new(),
            hb: Instant::now(),
            ccl_addr: ccl.get_ref().clone()
        },
        &req,
        stream,
    )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting HTTP server at http://localhost:1509");

    // start chain client actor
    let client = chain::ChainClient::new().start();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .route("/ws", web::get().to(chat_route))
            .wrap(Logger::default())
    })
    .workers(2)
    .bind(("127.0.0.1", 1509))?
    .run()
    .await
}