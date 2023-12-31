#![allow(dead_code)]

use std::str::FromStr;

use anyhow::{anyhow, Result};
use libp2p::{Multiaddr, PeerId};
use rusqlite::{named_params, Row};
use tracing::debug;

use time::OffsetDateTime;

use super::Store;

#[derive(Debug, Clone, PartialEq)]
pub struct ScsPeer {
    addrs: String,
    name: String,
    last_seen: String,
    peer_id: String,
    id: Option<i32>,
}

impl TryFrom<&Row<'_>> for ScsPeer {
    fn try_from(row: &Row<'_>) -> Result<Self> {
        debug!("Creating Peer from Row");

        let peer = ScsPeer {
            id: row.get(0)?,
            name: row.get(1)?,
            addrs: row.get(2)?,
            peer_id: row.get(3)?,
            last_seen: row.get(4)?,
        };
        Ok(peer)
    }

    type Error = anyhow::Error;
}

impl From<(&Multiaddr, String, PeerId)> for ScsPeer {
    fn from(value: (&Multiaddr, String, PeerId)) -> Self {
        debug!("Creating Peer from tuple");
        let (addr, name, peer_id) = value;
        let local = OffsetDateTime::now_utc();
        ScsPeer {
            addrs: addr.to_string(),
            name,
            last_seen: local.to_string(),
            peer_id: peer_id.to_string(),
            id: None,
        }
    }
}

impl ScsPeer {
    pub fn peer_id(&self) -> Result<PeerId> {
        match PeerId::from_str(self.peer_id.as_str()) {
            Ok(id) => Ok(id),
            Err(err) => Err(anyhow!("{}", err.to_string())),
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn fetch_all_peers(store: &Store) -> Result<Vec<ScsPeer>> {
        let conn = store.get_conn_handle();
        let mut stmt = conn.prepare("SELECT id, name, addrs, peer_id, last_seen FROM peer")?;
        let peer_iter = stmt.query_map([], |row| Ok(ScsPeer::try_from(row).unwrap()))?;
        let peers = peer_iter.filter_map(|peer| peer.ok()).collect::<Vec<_>>();
        Ok(peers)
    }

    pub fn save(&self, store: &Store) -> Result<()> {
        debug!("Saving Peer");
        let conn = store.get_conn_handle();
        conn.execute(
            "INSERT INTO peer (name, addrs, last_seen, peer_id) VALUES (?1, ?2, ?3, ?4)",
            (&self.name, &self.addrs, &self.last_seen, &self.peer_id),
        )?;
        Ok(())
    }

    pub fn get_by_name(name: String, store: &Store) -> Result<ScsPeer> {
        let conn = store.get_conn_handle();

        let mut statement = conn
            .prepare("SELECT id, name, addrs, peer_id, last_seen FROM peer WHERE name = :name")?;
        let peer_iter = statement.query_map(named_params! { ":name": name }, |row| {
            Ok(ScsPeer::try_from(row).unwrap())
        })?;
        let peers = peer_iter.filter_map(|peer| peer.ok()).collect::<Vec<_>>();
        if peers.is_empty() {
            Err(anyhow!("Cannot find peer with name: {name}"))
        } else {
            Ok(peers.first().unwrap().clone())
        }
    }

    pub fn get_by_peer_id(peer_id: String, store: &Store) -> Result<Option<ScsPeer>> {
        let conn = store.get_conn_handle();

        let mut statement = conn.prepare(
            "SELECT id, name, addrs, peer_id, last_seen FROM peer WHERE peer_id = :peer_id",
        )?;
        let peer_iter = statement.query_map(named_params! { ":peer_id": peer_id }, |row| {
            Ok(ScsPeer::try_from(row).unwrap())
        })?;
        let peers = peer_iter.filter_map(|peer| peer.ok()).collect::<Vec<_>>();
        if peers.is_empty() {
            Ok(None)
        } else {
            Ok(Some(peers.first().unwrap().clone()))
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use libp2p::{Multiaddr, PeerId};

    use crate::database::Store;

    use super::ScsPeer;

    #[test]
    fn peer_from_tuple() -> Result<()> {
        let peer_id = PeerId::random();
        let addr: Multiaddr = "/ip4/127.0.0.0/tcp/5555".parse().unwrap();
        let peer = ScsPeer::from((&addr, "foo".to_string(), peer_id));
        assert_eq!(peer.peer_id()?, peer_id);
        assert_eq!(peer.name().as_str(), "foo");
        Ok(())
    }

    #[test]
    fn wrong_peer_id() {
        let peer = ScsPeer {
            addrs: "/ip4/jki/oo/tcp/990".to_string(),
            name: "foo".to_string(),
            last_seen: "now".to_string(),
            peer_id: "hi".to_string(),
            id: Some(1),
        };

        let peer_id = peer.peer_id();
        assert!(peer_id.is_err());
    }

    #[test]
    fn create_peer() -> Result<()> {
        let db_path = assert_fs::NamedTempFile::new("scs_peer.db3")?;
        let store = Store::initialize(Some(db_path.path().to_path_buf()))?;

        let addr: Multiaddr = "/ip4/127.0.0.1/udp/38404/quic-v1".parse().unwrap();
        let peer_id = PeerId::random();
        let name = "kasali".to_string();

        let peer = ScsPeer::from((&addr, name, peer_id));
        peer.save(&store)?;

        let found_peer = ScsPeer::get_by_peer_id(PeerId::random().to_string(), &store)?;
        assert_eq!(found_peer, None);
        Ok(())
    }
}
