//This "Direct Connection Upgrade Through Relay Server" (DCUTR) allows peers to establish direct connections with each other.
//i.e hole punching

use std::process::exit;

use super::request_response_handler;
use crate::handlers::security::{is_ip_blacklisted, is_ip_whitelisted};
use crate::network::request::make_request;
use crate::network::{get_behaviour, ConnectionDetails, Event};
use crate::{config::Config, Mode};
use anyhow::Result;
use futures::{
    executor::{block_on, ThreadPool},
    stream::StreamExt,
    FutureExt,
};
use libp2p::{
    core::{muxing::StreamMuxerBox, upgrade},
    dns::DnsConfig,
    identify, identity,
    multiaddr::Protocol,
    noise, relay,
    swarm::{SwarmBuilder, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Transport,
};
use rand::Rng;
use tracing::{debug, error, info, instrument};

#[instrument(level = "trace")]
pub fn punch(mode: Mode, remote_peer_id: Option<PeerId>, config: Config) -> Result<()> {
    let relay_address: Multiaddr =
        "/ip4/157.245.40.97/tcp/4001/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN"
            .to_string()
            .parse()
            .unwrap();
    let secret_key_seed = rand::thread_rng().gen_range(0..100);
    let port = config.port();

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

    let behaviour = get_behaviour(client, local_key, local_peer_id);
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
    match mode {
        Mode::Send => {
            swarm
                .dial(
                    relay_address
                        .with(Protocol::P2pCircuit)
                        .with(Protocol::P2p(remote_peer_id.unwrap())),
                )
                .unwrap();
        }
        Mode::Receive => {
            swarm
                .listen_on(relay_address.with(Protocol::P2pCircuit))
                .unwrap();
        }
    }
    let mut connection_deets = ConnectionDetails::new();
    block_on(async {
        loop {
            match swarm.next().await.unwrap() {
                SwarmEvent::NewListenAddr { address, .. } => {
                    info!("Listening on {:?}", address);
                }
                SwarmEvent::Behaviour(Event::Relay(
                    relay::client::Event::ReservationReqAccepted { .. },
                )) => {
                    assert!(mode == Mode::Receive);
                    debug!("Relay accepted our reservation request.");
                }
                SwarmEvent::Behaviour(Event::Relay(event)) => {
                    debug!("RELAY: {:?}", event)
                }
                SwarmEvent::Behaviour(Event::Dcutr(event)) => {
                    debug!("DCUTR: {:?}", event)
                }
                SwarmEvent::Behaviour(Event::Identify(event)) => {
                    debug!("IDENTIFY: {:?}", event);
                    let connection_id = connection_deets.id().unwrap();

                    if is_ip_blacklisted(&event, &config) {
                        // println!("deetttss {:#?}", connection_id);
                        error!("This IP address is present in your blacklist.");
                        swarm.close_connection(connection_id);
                    }

                    if !is_ip_whitelisted(&event, &config) {
                        swarm.close_connection(connection_id);
                    }
                }
                SwarmEvent::Behaviour(Event::Ping(_)) => {}
                SwarmEvent::IncomingConnection { connection_id, .. } => {
                    connection_deets.save_id(connection_id);
                }
                SwarmEvent::ConnectionEstablished {
                    peer_id, endpoint, ..
                } => {
                    let addr = endpoint.get_remote_address();
                    info!("Established connection to {peer_id} via {addr}");

                    //Send secrets to the receiver
                    make_request(mode, &mut swarm, peer_id, &config);
                }
                SwarmEvent::OutgoingConnectionError {
                    peer_id: _, error, ..
                } => {
                    error!("{:#?}", error.to_string());
                }
                SwarmEvent::Behaviour(Event::RequestResonse(
                    request_response::Event::Message { peer, message },
                )) => {
                    request_response_handler(&mut swarm, message, peer, &config);
                }
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