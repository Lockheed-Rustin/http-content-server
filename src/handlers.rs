use crate::{AppState, POLL_TIME};
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use drone_network::{
    controller::Error,
    message::{ClientBody, ClientContentBody, ServerContentBody},
};
use mime_guess::mime::{TEXT_PLAIN, TEXT_PLAIN_UTF_8};
use serde::Deserialize;
use std::thread;

pub fn get_path(path: String) -> String {
    if !path.starts_with("text") && !path.starts_with("site") {
        format!("site/{}", path)
    } else {
        path
    }
}

pub async fn req_file(
    State(state): State<AppState>,
    Path(file_path): Path<String>,
) -> impl IntoResponse {
    let file_path = get_path(file_path);
    let mut header = HeaderMap::new();
    header.insert(
        header::CONTENT_TYPE,
        TEXT_PLAIN_UTF_8.to_string().parse().unwrap(),
    );

    let guess = mime_guess::from_path(&file_path);
    if let Some(mime) = guess.first() {
        if mime != TEXT_PLAIN {
            header.insert(header::CONTENT_TYPE, mime.essence_str().parse().unwrap());
        }
    }
    if let Some(file) = get_file(state.clone(), file_path.clone()) {
        if let Some(file_type) = infer::get(&file) {
            header.insert(header::CONTENT_TYPE, file_type.mime_type().parse().unwrap());
        }
        (StatusCode::OK, header, file)
    } else {
        (StatusCode::NOT_FOUND, header, b"Not Found".to_vec())
    }
}

fn get_file(state: AppState, file_path: String) -> Option<Vec<u8>> {
    {
        let controller = state.controller.lock().unwrap();
        controller
            .client_send_message(
                state.client_id,
                state.server_id,
                ClientBody::ClientContent(ClientContentBody::ReqFile(file_path.clone())),
            )
            .unwrap();
    }
    loop {
        thread::sleep(POLL_TIME);
        let mut cache = state.cache.lock().unwrap();
        let i = cache.iter().position(|body| {
            matches!(body, ServerContentBody::RespFile(_, path) if *path == file_path)
                || matches!(body, ServerContentBody::ErrFileNotFound)
        });
        if let Some(i) = i {
            return match cache.remove(i) {
                ServerContentBody::RespFile(vec, _) => Some(vec),
                ServerContentBody::ErrFileNotFound => None,
                _ => unreachable!(),
            };
        }
    }
}

pub async fn file_list(State(state): State<AppState>) -> impl IntoResponse {
    {
        let controller = state.controller.lock().unwrap();
        controller
            .client_send_message(
                state.client_id,
                state.server_id,
                ClientBody::ClientContent(ClientContentBody::ReqFilesList),
            )
            .unwrap();
    }
    loop {
        thread::sleep(POLL_TIME);
        let mut cache = state.cache.lock().unwrap();
        let i = cache
            .iter()
            .position(|body| matches!(body, ServerContentBody::RespFilesList(_)));
        if let Some(i) = i {
            if let ServerContentBody::RespFilesList(file_list) = cache.remove(i) {
                return Json(file_list);
            } else {
                unreachable!();
            }
        }
    }
}

pub async fn crash_drone(State(state): State<AppState>, Path(id): Path<u8>) -> impl IntoResponse {
    let mut controller = state.controller.lock().unwrap();
    match controller.crash_drone(id) {
        Ok(_) => (StatusCode::OK, format!("crashed drone {}", id)),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            match e {
                Error::Missing | Error::SendError => format!("{} is not a valid id", id),
                Error::InvalidNode => format!("node {} is not a drone", id),
                Error::InvalidTopology => format!("cannot crash drone {}: invalid topology", id),
                Error::EdgeExists => unreachable!(),
            },
        ),
    }
}

#[derive(Deserialize)]
pub struct Pdr {
    pdr: f32,
}

pub async fn set_pdr(State(state): State<AppState>, Json(pdr): Json<Pdr>) -> impl IntoResponse {
    let mut controller = state.controller.lock().unwrap();
    let drone_ids = controller.get_drone_ids();
    for id in drone_ids {
        controller.set_pdr(id, pdr.pdr).unwrap();
    }
}
