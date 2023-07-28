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

#[cfg(test)]
mod tests {
    use super::{is_ip_blacklisted, is_ip_whitelisted};
    use anyhow::Result;
    use std::str::FromStr;

    use libp2p::{
        identify::{Event, Info},
        identity, Multiaddr, PeerId, StreamProtocol,
    };

    use crate::config::Config;

    fn generate_ed25519() -> identity::PublicKey {
        let mut bytes = [0u8; 32];
        bytes[0] = 2;

        let key_pair =
            identity::Keypair::ed25519_from_bytes(bytes).expect("only errors on wrong length");
        key_pair.public()
    }

    fn generate_event(addrs: &Vec<&str>, event_type: &str) -> Result<Event> {
        let multi_addrs = addrs
            .iter()
            .map(|addr| Multiaddr::from_str(addr).unwrap())
            .collect::<Vec<_>>();
        let protocols = vec![StreamProtocol::new("/foo/bar/1.0.0`")];

        let info = Info {
            public_key: generate_ed25519(),
            protocol_version: "test/0.0.1".to_string(),
            agent_version: "foo/bar".to_string(),
            listen_addrs: multi_addrs,
            protocols: protocols,
            observed_addr: Multiaddr::from_str("/ip4/186.0.0.2/tcp/43675").unwrap(),
        };

        let event = match event_type {
            "received" => Event::Received {
                peer_id: PeerId::random(),
                info,
            },
            _ => Event::Sent {
                peer_id: PeerId::random(),
            },
        };
        Ok(event)
    }

    fn generate_config(addr: &str) -> Result<Config> {
        let yaml_config = format!(
            "
            port: 5555 
            save_path: 'default'
            secret:
            - key: foo
              value: bar
            - key: baz
              value: woo
            message: 
            - new message from me
            - test message
            debug: 1
            blacklists:
            - {addr}
            whitelists:
            - {addr}
        "
        );
        let config: Config = serde_yaml::from_str(&yaml_config)?;

        Ok(config)
    }

    #[test]
    fn blacklisted_ip() -> Result<()> {
        let addrs = vec![
            "/ip4/127.0.0.1/tcp/43675",
            "/ip4/142.132.198.26/tcp/43675",
            "/ip4/189.173.43.88/tcp/4001",
        ];

        let event = generate_event(&addrs, "received")?;
        let config = generate_config("142.132.198.26")?;
        assert!(is_ip_blacklisted(&event, &config));
        Ok(())
    }

    #[test]
    fn passthrough_ip() -> Result<()> {
        let addrs = vec![
            "/ip4/127.0.0.1/tcp/43675",
            "/ip4/142.132.198.26/tcp/43675",
            "/ip4/189.173.43.88/tcp/4001",
        ];
        let event = generate_event(&addrs, "received")?;
        let config = generate_config("142.132.198.10")?;
        assert!(!is_ip_blacklisted(&event, &config));
        Ok(())
    }

    #[test]
    fn sent_event() -> Result<()> {
        let addrs = vec![
            "/ip4/127.0.0.1/tcp/43675",
            "/ip4/142.132.198.26/tcp/43675",
            "/ip4/189.173.43.88/tcp/4001",
        ];
        let config = generate_config("142.132.198.26")?;
        let event = generate_event(&addrs, "sent")?;

        assert!(!is_ip_blacklisted(&event, &config));
        Ok(())
    }

    #[test]
    fn whitelisted_ip() -> Result<()> {
        let addrs = vec![
            "/ip4/127.0.0.1/tcp/43675",
            "/ip4/142.132.198.26/tcp/43675",
            "/ip4/189.173.43.88/tcp/4001",
        ];
        let config = generate_config("142.132.198.26")?;
        let event = generate_event(&addrs, "received")?;

        assert!(!is_ip_whitelisted(&event, &config));
        Ok(())
    }

    #[test]
    fn passthrough_whitelist_ip() -> Result<()> {
        let addrs = vec![
            "/ip4/127.0.0.1/tcp/43675",
            "/ip4/142.132.198.26/tcp/43675",
            "/ip4/189.173.43.88/tcp/4001",
        ];
        let config = generate_config("142.132.198.26")?;
        let event = generate_event(&addrs, "sent")?;

        assert!(is_ip_whitelisted(&event, &config));
        Ok(())
    }
}
