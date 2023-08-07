use std::process::exit;

use libp2p::{PeerId, Swarm};
use request_response::ResponseChannel;
use tracing::{error, info};

use crate::{
    config::Config,
    item::{Item, ItemResponse, ItemType, Status},
    Mode,
};

use super::Behaviour;

pub fn handle_request(
    request: Vec<Item>,
    config: &Config,
    swarm: &mut Swarm<Behaviour>,
    channel: ResponseChannel<ItemResponse>,
) {
    let mut items_saved_successfully: Vec<&Item> = vec![];
    let mut items_saved_fail: Vec<&Item> = vec![];

    request.iter().for_each(|item| match item.save(config) {
        Ok(_) => {
            info!("Saved {:?} successfully", item.item_type(),);
            items_saved_successfully.push(item)
        }
        Err(err) => {
            error!("Failed to send {:?}: {}", item.item_type(), err.to_string());
            items_saved_fail.push(item);
        }
    });

    let status = Status::Succes;

    let res = ItemResponse {
        status,
        no_of_fails: items_saved_fail.len(),
        no_of_success: items_saved_successfully.len(),
        err: None,
    };

    swarm
        .behaviour_mut()
        .request_response
        .send_response(channel, res)
        .unwrap();
}

pub fn make_request(mode: Mode, swarm: &mut Swarm<Behaviour>, peer_id: PeerId, config: &Config) {
    match mode {
        Mode::Send => {
            let items = get_items_to_be_sent(config);

            info!("Sending {} items", items.len());
            swarm
                .behaviour_mut()
                .request_response
                .send_request(&peer_id, items);
        }
        Mode::Receive | Mode::List => {
            // if !is_ip_whitelisted(event, config)
        }
    }
}

fn get_items_to_be_sent(opts: &Config) -> Vec<Item> {
    if opts.file().is_none() && opts.secret().is_none() && opts.message().is_none() {
        error!("Pass in a secret with the `-s` flag or a message with `-m` flag or a file path with the `f` flag");
        exit(1);
    }

    let mut items = match &opts.secret() {
        None => vec![],
        Some(secrets) => secrets.iter().map(Item::from).collect::<Vec<_>>(),
    };

    let mut messages = {
        match &opts.message() {
            None => vec![],
            Some(msgs) => msgs
                .iter()
                //I can unwrap safely because I don't expect any error
                .map(|msg| Item::new(msg.clone(), ItemType::Message).unwrap())
                .collect::<Vec<_>>(),
        }
    };
    let mut files = match &opts.file() {
        None => vec![],
        Some(paths) => paths
            .iter()
            .map(|path| match Item::new(path.to_string(), ItemType::File) {
                Err(err) => {
                    error!("{}", err.to_string());
                    exit(1);
                }
                Ok(res) => res,
            })
            .collect::<Vec<_>>(),
    };

    items.append(&mut messages);
    items.append(&mut files);
    items
}
