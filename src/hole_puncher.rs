//This "Direct Connection Upgrade Through Relay Server" (DCUTR) allows peers to establish direct connections with each other.
//i.e hole punching

use std::process::exit;

use crate::{
    item::{Item, ItemResponse, ItemType, Status},
    Mode,
};
use anyhow::Result;
use futures::{
    executor::{block_on, ThreadPool},
    stream::StreamExt,
    FutureExt,
};
use libp2p::{
    core::{muxing::StreamMuxerBox, upgrade},
    dcutr,
    dns::DnsConfig,
    identify, identity,
    multiaddr::Protocol,
    noise, ping, relay,
    swarm::{NetworkBehaviour, SwarmBuilder, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, StreamProtocol, Transport,
};
use rand::Rng;
use request_response::{self, json, ProtocolSupport};
use tracing::{debug, error, info, instrument};

use super::Cli;

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "Event")]
struct Behaviour {
    relay_client: relay::client::Behaviour,
    ping: ping::Behaviour,
    identify: identify::Behaviour,
    dcutr: dcutr::Behaviour,
    request_response: json::Behaviour<Vec<Item>, ItemResponse>,
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
enum Event {
    Ping(ping::Event),
    Identify(identify::Event),
    Relay(relay::client::Event),
    Dcutr(dcutr::Event),
    RequestResonse(request_response::Event<Vec<Item>, ItemResponse>),
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

#[instrument(level = "trace")]
pub fn punch(opts: Cli) -> Result<()> {
    let relay_address: Multiaddr =
        "/ip4/157.245.40.97/tcp/4001/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN"
            .to_string()
            .parse()
            .unwrap();
    let secret_key_seed = rand::thread_rng().gen_range(0..100);
    let port = opts.port.unwrap_or(0).to_string();

    let local_key = generate_ed25519(secret_key_seed);
    let local_peer_id = PeerId::from(local_key.public());
    info!("Your PeerId is: {}", local_peer_id);

    //intitate relay client connection
    let (relay_transport, client) = relay::client::new(local_peer_id);
    let transport = {
        let relay_tcp_quic_transport = relay_transport
            .or_transport(tcp::async_io::Transport::new(
                tcp::Config::default().port_reuse(true),
            ))
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::Config::new(&local_key).unwrap())
            .multiplex(yamux::Config::default());

        block_on(DnsConfig::system(relay_tcp_quic_transport))
            .unwrap()
            .map(|either_output, _| (either_output.0, StreamMuxerBox::new(either_output.1)))
            .boxed()
    };

    let behaviour = Behaviour {
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
    };
    let mut swarm = match ThreadPool::new() {
        Ok(tp) => SwarmBuilder::with_executor(transport, behaviour, local_peer_id, tp),
        Err(_) => SwarmBuilder::without_executor(transport, behaviour, local_peer_id),
    }
    .build();

    swarm
        .listen_on(format!("/ip4/0.0.0.0/tcp/{port}").parse().unwrap())
        .unwrap();

    // Wait to listen on all interfaces.
    block_on(async {
        let mut delay = futures_timer::Delay::new(std::time::Duration::from_secs(1)).fuse();
        loop {
            futures::select! {
                event = swarm.next() => {
                    match event.unwrap() {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            info!("Listening on {:?}", address);
                        }
                        event =>  { error!("{event:?}"); exit(1); }
                    }
                }
                _ = delay => {
                    // Likely listening on all interfaces now, thus continuing by breaking the loop.
                    break;
                }
            }
        }
    });

    // Connect to the relay server. Not for the reservation or relayed connection, but to (a) learn
    // our local public address and (b) enable a freshly started relay to learn its public address.
    swarm.dial(relay_address.clone()).unwrap();
    block_on(async {
        let mut learned_observed_addr = false;
        let mut told_relay_observed_addr = false;

        loop {
            match swarm.next().await.unwrap() {
                SwarmEvent::NewListenAddr { .. } => {}
                SwarmEvent::Dialing { .. } => {}
                SwarmEvent::ConnectionEstablished { .. } => {}
                SwarmEvent::Behaviour(Event::Ping(_)) => {}
                SwarmEvent::Behaviour(Event::Identify(identify::Event::Sent { .. })) => {
                    debug!("Told relay its public address.");
                    told_relay_observed_addr = true;
                }
                SwarmEvent::Behaviour(Event::Identify(identify::Event::Received {
                    info: identify::Info { observed_addr, .. },
                    ..
                })) => {
                    debug!("Relay told us our public address: {:?}", observed_addr);
                    swarm.add_external_address(observed_addr);
                    learned_observed_addr = true;
                }
                event => error!("{event:?}"),
            }

            if learned_observed_addr && told_relay_observed_addr {
                break;
            }
        }
    });

    //denotes whether to send or receive secrets
    match opts.mode {
        Mode::Send => {
            swarm
                .dial(
                    relay_address
                        .with(Protocol::P2pCircuit)
                        .with(Protocol::P2p(opts.remote_peer_id.unwrap())),
                )
                .unwrap();
        }
        Mode::Receive => {
            swarm
                .listen_on(relay_address.with(Protocol::P2pCircuit))
                .unwrap();
        }
    }

    block_on(async {
        loop {
            match swarm.next().await.unwrap() {
                SwarmEvent::NewListenAddr { address, .. } => {
                    info!("Listening on {:?}", address);
                }
                SwarmEvent::Behaviour(Event::Relay(
                    relay::client::Event::ReservationReqAccepted { .. },
                )) => {
                    assert!(opts.mode == Mode::Receive);
                    debug!("Relay accepted our reservation request.");
                }
                SwarmEvent::Behaviour(Event::Relay(event)) => {
                    debug!("RELAY: {:?}", event)
                }
                SwarmEvent::Behaviour(Event::Dcutr(event)) => {
                    debug!("DCUTR: {:?}", event)
                }
                SwarmEvent::Behaviour(Event::Identify(event)) => {
                    debug!("IDENTIFY: {:?}", event)
                }
                SwarmEvent::Behaviour(Event::Ping(_)) => {}
                SwarmEvent::ConnectionEstablished {
                    peer_id, endpoint, ..
                } => {
                    let addr = endpoint.get_remote_address();
                    info!("Established connection to {peer_id} via {addr}");

                    //Send secrets to the receiver
                    match opts.mode {
                        Mode::Send => {
                            let items = get_items_to_be_sent(&opts);

                            info!("Sending items: {:#?}", items);
                            swarm
                                .behaviour_mut()
                                .request_response
                                .send_request(&peer_id, items);
                        }
                        Mode::Receive => {}
                    }
                }
                SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                    error!(
                        "Outgoing connection error to {:?}: {:?}",
                        peer_id,
                        error.to_string()
                    );
                }
                SwarmEvent::Behaviour(Event::RequestResonse(
                    request_response::Event::Message { peer, message },
                )) => match message {
                    request_response::Message::Request {
                        request_id: _,
                        request,
                        channel,
                    } => {
                        info!("Received {} items from {peer}", request.len());
                        let mut items_saved_successfully: Vec<&Item> = vec![];
                        let mut items_saved_fail: Vec<&Item> = vec![];

                        request.iter().for_each(|item| match item.save() {
                            Ok(_) => {
                                info!("Saved {:?} successfully", item.item_type(),);
                                items_saved_successfully.push(item)
                            }
                            Err(err) => {
                                error!(
                                    "Failed to save {:?}: {}",
                                    item.item_type(),
                                    err.to_string()
                                );
                                items_saved_fail.push(item);
                            }
                        });

                        let status = Status::Succes;

                        //TODO handle error
                        swarm
                            .behaviour_mut()
                            .request_response
                            .send_response(
                                channel,
                                ItemResponse {
                                    status,
                                    no_of_fails: items_saved_fail.len(),
                                    no_of_success: items_saved_successfully.len(),
                                },
                            )
                            .unwrap();
                    }
                    request_response::Message::Response {
                        request_id: _,
                        response,
                    } => {
                        info!("Saved {} items successfully", response.no_of_success);
                        if response.no_of_fails > 0 {
                            error!("Failed to save {} items", response.no_of_fails);
                        }
                    }
                },
                _ => {}
            }
        }
    })
}

fn generate_ed25519(secret_key_seed: u8) -> identity::Keypair {
    let mut bytes = [0u8; 32];
    bytes[0] = secret_key_seed;

    identity::Keypair::ed25519_from_bytes(bytes).expect("only errors on wrong length")
}

fn get_items_to_be_sent(opts: &Cli) -> Vec<Item> {
    if opts.file.is_none() && opts.secret.is_none() && opts.message.is_none() {
        error!("Pass in a secret with the `-s` flag or a message with `-m` flag or a file path with the `f` flag");
        exit(1);
    }

    let mut items = match &opts.secret {
        None => vec![],
        Some(secrets) => secrets
            .iter()
            .map(
                |secret| match Item::new(secret.to_string(), ItemType::Secret) {
                    Err(err) => {
                        error!("{}", err.to_string());
                        exit(1);
                    }
                    Ok(res) => res,
                },
            )
            .collect::<Vec<_>>(),
    };

    let mut messages = {
        match &opts.message {
            None => vec![],
            Some(msgs) => msgs
                .iter()
                //I can unwrap safely because I don't expect any error
                .map(|msg| Item::new(msg.clone(), ItemType::Message).unwrap())
                .collect::<Vec<_>>(),
        }
    };

    let mut files = match &opts.file {
        None => vec![],
        Some(secrets) => secrets
            .iter()
            .map(
                |secret| match Item::new(secret.to_string(), ItemType::File) {
                    Err(err) => {
                        error!("{}", err.to_string());
                        exit(1);
                    }
                    Ok(res) => res,
                },
            )
            .collect::<Vec<_>>(),
    };

    items.append(&mut messages);
    items.append(&mut files);
    items
}
