use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use blackbook::{
    errors,
    signalling::{signalling_conn, SignallingService},
    AppState,
};
use dotenv::dotenv;
use std::convert::From;
use std::net::Ipv4Addr;
use std::{
    env,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), errors::BlackbookServerError> {
    dotenv().ok();

    let svc = SignallingService::new();

    let mut env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // axum logs rejections from built-in extractors with the `axum::rejection`
        // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
        "example_tracing_aka_logging=debug,tower_http=debug,axum::rejection=trace,parelthon_server=debug,error,info".into()
    });

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    let port: u16 = 1690;

    // 0.0.0.0: This IP address is a way to specify that the socket should bind to all available network interfaces on
    // the host machine. It's a common choice when you want your service to be reachable from outside networks.
    // let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port));

    tracing::debug!("listening on {}", addr);

    let router = AppState::new(svc).router().await?;

    let server = axum::Server::bind(&addr)
        .serve(router.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();

    Ok(())
}
