use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use y_sync::awareness::Awareness;

pub mod errors;
pub mod signalling;
pub mod ws;

pub type BroadcastGroup = y_sync::net::BroadcastGroup;
pub type AwarenessRef = Arc<RwLock<Awareness>>;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo, State,
    },
    response::IntoResponse,
    routing::get,
    Router, TypedHeader,
};

use http::Method;
use signalling::{Signal, SignallingService};

use tokio::sync::broadcast;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

use crate::signalling::signalling_conn;

const USER_COOKIE_NAME: &str = "user_token";
const COOKIE_MAX_AGE: &str = "9999999";

#[derive(Clone)]
pub struct AppState {
    // Channel used to send messages to all connected clients.
    svc: SignallingService,
}

impl AppState {
    pub fn new(svc: SignallingService) -> Self {
        Self { svc }
    }

    pub async fn router(self) -> Result<axum::Router, errors::BlackbookServerError> {
        let trace_layer = TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
            .on_response(DefaultOnResponse::new().level(Level::INFO));

        let cors_layer = CorsLayer::new()
            // allow `GET` and `POST` when accessing the resource
            .allow_methods([Method::GET, Method::POST])
            // allow requests from any origin
            .allow_origin(Any);

        let router = Router::new()
            .layer(cors_layer)
            .layer(trace_layer)
            .route("/ws", get(ws_handler))
            .with_state(self);

        let api = Router::new().nest("/:version/api", router);

        Ok(api)
    }
}

#[axum::debug_handler]
async fn ws_handler(
    State(AppState { svc }): State<AppState>,
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    println!("`{user_agent}` at {addr} connected.");

    ws.on_upgrade(move |socket| peer(socket, svc))
}

async fn peer(ws: WebSocket, svc: SignallingService) {
    println!("new incoming signaling connection");
    match signalling_conn(ws, svc).await {
        Ok(_) => println!("signaling connection stopped"),
        Err(e) => eprintln!("signaling connection failed: {}", e),
    }
}
