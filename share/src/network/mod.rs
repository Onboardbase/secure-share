use std::time::Duration;

use libp2p::{
    autonat, dcutr, identify, ping, relay,
    swarm::{ConnectionId, NetworkBehaviour},
    PeerId, StreamProtocol, Swarm,
};
use request_response::{json, Message, ProtocolSupport};
use tracing::{error, info};

use crate::{
    config::Config,
    item::{Item, ItemResponse},
    Mode,
};
pub use hole_puncher::punch;
use request::handle_request;

mod hole_puncher;
mod request;

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "Event")]
pub struct Behaviour {
    relay_client: relay::client::Behaviour,
    ping: ping::Behaviour,
    identify: identify::Behaviour,
    dcutr: dcutr::Behaviour,
    request_response: json::Behaviour<Vec<Item>, ItemResponse>,
    auto_nat: autonat::Behaviour,
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Event {
    Ping(ping::Event),
    Identify(identify::Event),
    Relay(relay::client::Event),
    Dcutr(dcutr::Event),
    RequestResonse(request_response::Event<Vec<Item>, ItemResponse>),
    AutoNat(autonat::Event),
}

impl From<ping::Event> for Event {
    fn from(e: ping::Event) -> Self {
        Event::Ping(e)
    }
}

impl From<identify::Event> for Event {
    fn from(e: identify::Event) -> Self {
        Event::Identify(e)
    }
}

impl From<relay::client::Event> for Event {
    fn from(e: relay::client::Event) -> Self {
        Event::Relay(e)
    }
}

impl From<dcutr::Event> for Event {
    fn from(e: dcutr::Event) -> Self {
        Event::Dcutr(e)
    }
}

impl From<request_response::Event<Vec<Item>, ItemResponse>> for Event {
    fn from(e: request_response::Event<Vec<Item>, ItemResponse>) -> Self {
        Event::RequestResonse(e)
    }
}

impl From<autonat::Event> for Event {
    fn from(v: autonat::Event) -> Self {
        Self::AutoNat(v)
    }
}

//I know libp2p stores the info, but I need them all in one place
#[derive(Debug, Clone)]
pub struct ConnectionDetails {
    connection_id: Option<ConnectionId>,
    // listen_addrs: Vec<Protocol<'a>>,
}

impl ConnectionDetails {
    pub fn new() -> ConnectionDetails {
        ConnectionDetails {
            connection_id: None,
            // listen_addrs: vec![],
        }
    }

    pub fn save_id(&mut self, id: ConnectionId) -> &ConnectionDetails {
        self.connection_id = Some(id);
        self
    }

    pub fn id(&self) -> Option<ConnectionId> {
        self.connection_id
    }

    // pub fn save_addresses<'b>(
    //     &'b mut self,
    //     addrs: Vec<Protocol<'static>>,
    // ) -> &ConnectionDetails<'b> {
    //     self.listen_addrs = addrs.clone();
    //     self
    // }
}

pub fn get_behaviour(
    payload: (
        libp2p::relay::client::Behaviour,
        libp2p::identity::Keypair,
        PeerId,
        Mode,
    ),
) -> Behaviour {
    let (client, local_key, local_peer_id, mode) = payload;
    let autonat_config = match mode {
        Mode::Receive => autonat::Config {
            only_global_ips: false,
            ..Default::default()
        },
        Mode::Send => autonat::Config {
            retry_interval: Duration::from_secs(10),
            refresh_interval: Duration::from_secs(30),
            boot_delay: Duration::from_secs(5),
            throttle_server_period: Duration::ZERO,
            only_global_ips: false,
            ..Default::default()
        },
        Mode::List => autonat::Config {
            ..Default::default()
        },
    };

    Behaviour {
        relay_client: client,
        ping: ping::Behaviour::new(ping::Config::new()),
        identify: identify::Behaviour::new(identify::Config::new(
            "/SHARE/0.0.1".to_string(),
            local_key.public(),
        )),
        dcutr: dcutr::Behaviour::new(local_peer_id),
        request_response: json::Behaviour::<Vec<Item>, ItemResponse>::new(
            [(
                StreamProtocol::new("/share-json-protocol"),
                ProtocolSupport::Full,
            )],
            request_response::Config::default(),
        ),
        auto_nat: autonat::Behaviour::new(local_peer_id, autonat_config),
    }
}

fn request_response_handler(
    swarm: &mut Swarm<Behaviour>,
    message: Message<Vec<Item>, ItemResponse>,
    peer: PeerId,
    config: &Config,
) {
    match message {
        request_response::Message::Request {
            request_id: _,
            request,
            channel,
        } => {
            info!("Received {} items from {peer}", request.len());
            handle_request(request, config, swarm, channel);
        }
        request_response::Message::Response {
            request_id: _,
            response,
        } => {
            info!("Sent {} items successfully", response.no_of_success);
            if response.no_of_fails > 0 {
                error!("Failed to save {} items", response.no_of_fails);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use libp2p::{identity, relay, swarm::ConnectionId, PeerId};

    use crate::Mode;

    use super::{get_behaviour, ConnectionDetails};

    #[test]
    fn new_connection_details() {
        let details = ConnectionDetails::new();
        assert_eq!(details.connection_id, None)
    }

    #[test]
    fn connection_id() {
        let mut details = ConnectionDetails::new();
        let id = ConnectionId::new_unchecked(0);
        details.save_id(id);
        assert_eq!(details.id(), Some(id));
    }

    fn generate_ed25519() -> identity::Keypair {
        let mut bytes = [0u8; 32];
        bytes[0] = 2;

        identity::Keypair::ed25519_from_bytes(bytes).expect("only errors on wrong length")
    }

    #[test]
    fn hole_puncher_behaviour() {
        let local_key = generate_ed25519();
        let peer_id = PeerId::random();
        let (_, relay_client) = relay::client::new(peer_id);
        let behaviour = get_behaviour((relay_client, local_key, peer_id, Mode::Send));
        assert!(!behaviour.request_response.is_connected(&peer_id));
    }

    #[test]
    fn public_nat() {
        let local_key = generate_ed25519();
        let peer_id = PeerId::random();
        let (_, relay_client) = relay::client::new(peer_id);
        let behaviour = get_behaviour((relay_client, local_key, peer_id, Mode::Send));
        let nat_status = behaviour.auto_nat.nat_status();
        assert!(!nat_status.is_public());
    }
}
