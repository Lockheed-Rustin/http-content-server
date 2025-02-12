use crate::{AppState, POLL_TIME};
use crossbeam_channel::Receiver;
use drone_network::{
    controller::{ClientEvent, SimulationController},
    message::{ClientBody, ServerBody, ServerType},
};
use std::fs;
use wg_2024::{config::Config, network::NodeId};

pub fn parse_config(path: &str) -> Config {
    let config = fs::read_to_string(path).unwrap();
    toml::from_str(&config).unwrap()
}

pub fn get_ids(controller: &SimulationController) -> (NodeId, NodeId) {
    let client_id = controller.get_client_ids()[0];
    let server_ids = controller.get_server_ids();

    for server_id in server_ids {
        controller
            .client_send_message(client_id, server_id, ClientBody::ReqServerType)
            .unwrap();
    }
    let recv = controller.get_client_recv();
    while let Ok(event) = recv.recv_timeout(POLL_TIME) {
        if let ClientEvent::MessageAssembled {
            body: ServerBody::RespServerType(server_type),
            from: server_id,
            ..
        } = event
        {
            if server_type == ServerType::Content {
                return (client_id, server_id);
            }
        }
    }
    panic!("no content server found");
}

pub fn event_loop(state: AppState, recv: Receiver<ClientEvent>) {
    loop {
        let event = recv.recv().unwrap();
        if let ClientEvent::MessageAssembled { body, .. } = event {
            if let ServerBody::ServerContent(body) = body {
                let mut cache = state.cache.lock().unwrap();
                cache.push(body);
            }
        }
    }
}
