use libp2p::{
    identify::{Event, Info},
    multiaddr::Protocol,
};

use crate::config::Config;

pub fn is_ip_whitelisted(event: &Event, config: &Config) -> bool {
    if let Some(whitelists) = config.whitelists() {
        match event {
            Event::Received {
                info: Info { listen_addrs, .. },
                ..
            } => {
                let addresses = listen_addrs
                    .iter()
                    .map(|addr| {
                        let components = addr.iter().collect::<Vec<_>>();
                        components[0].clone()
                    })
                    .collect::<Vec<_>>();

                addresses.iter().all(|addr| match addr {
                    Protocol::Ip4(ip_addr) => whitelists.contains(ip_addr),
                    _ => false,
                })
            }
            _ => true,
        }
    } else {
        true
    }
}

pub fn is_ip_blacklisted(event: &Event, config: &Config) -> bool {
    if let Some(blacklists) = config.blacklists() {
        match event {
            Event::Received {
                info: Info { listen_addrs, .. },
                ..
            } => {
                let addresses = listen_addrs
                    .iter()
                    .map(|addr| {
                        let components = addr.iter().collect::<Vec<_>>();
                        components[0].clone()
                    })
                    .collect::<Vec<_>>();

                addresses.iter().any(|addr| match addr {
                    Protocol::Ip4(ip_addr) => blacklists.contains(ip_addr),
                    _ => false,
                })
            }
            _ => false,
        }
    } else {
        false
    }
}
