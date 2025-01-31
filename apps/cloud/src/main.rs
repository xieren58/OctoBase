use axum::{Extension, Router, Server};
use http::Method;
use jwst_logger::{error, info, init_logger};
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};

mod api;
mod context;
mod error_status;
mod files;
mod layer;
mod utils;

#[tokio::main]
async fn main() {
    init_logger();

    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::DELETE,
            Method::OPTIONS,
        ])
        // allow requests from any origin
        .allow_origin(["https://affine-next.vercel.app".parse().unwrap()])
        .allow_headers(Any);

    let context = Arc::new(context::Context::new().await);

    let app = files::static_files(
        Router::new()
            .nest(
                "/api",
                api::make_rest_route(context.clone()).nest("/sync", api::make_ws_route()),
            )
            .layer(Extension(context.clone()))
            .layer(cors),
    );

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("listening on {}", addr);

    if let Err(e) = Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(utils::shutdown_signal())
        .await
    {
        error!("Server shutdown due to error: {}", e);
    }

    info!("Server shutdown complete");
}
