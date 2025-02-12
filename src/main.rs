mod handlers;
mod helper;

use axum::{
    response::Redirect,
    routing::{get, post},
    Router,
};
use drone_network::{controller::SimulationController, message::ServerContentBody, network};
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use tokio::net::TcpListener;
use wg_2024::network::NodeId;

const POLL_TIME: Duration = Duration::from_millis(100);

#[derive(Clone)]
struct AppState {
    controller: Arc<Mutex<SimulationController>>,
    client_id: NodeId,
    server_id: NodeId,
    cache: Arc<Mutex<Vec<ServerContentBody>>>,
}

#[tokio::main]
async fn main() {
    let config = helper::parse_config("config.toml");
    let controller = network::init_network(&config).unwrap();
    let recv = controller.get_client_recv();
    let (client_id, server_id) = helper::get_ids(&controller);

    let state = AppState {
        controller: Arc::new(Mutex::new(controller)),
        client_id,
        server_id,
        cache: Arc::new(Mutex::new(Vec::new())),
    };

    let app = Router::new()
        .route("/", get(Redirect::to("/index.html")))
        .route("/file_list", get(handlers::file_list))
        .route("/{*path}", get(handlers::req_file))
        .route("/crash_drone/{id}", post(handlers::crash_drone))
        .route("/set_pdr", post(handlers::set_pdr))
        .with_state(state.clone());

    thread::spawn(move || {
        helper::event_loop(state, recv);
    });

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
