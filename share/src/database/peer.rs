#![allow(dead_code)]

use anyhow::{anyhow, Result};
use libp2p::{Multiaddr, PeerId};
use rusqlite::{named_params, Row};
use tracing::debug;

use time::OffsetDateTime;

use super::Store;

#[derive(Debug, Clone)]
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
    pub fn fetch_all_peers(store: &Store) -> Result<Vec<ScsPeer>> {
        let conn = store.get_conn_handle();
        let mut stmt = conn.prepare("SELECT id, name, addrs, last_seen FROM peer")?;
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

    pub fn get_peer(name: String, store: &Store) -> Result<ScsPeer> {
        let conn = store.get_conn_handle();

        let mut statement =
            conn.prepare("SELECT id, name, addrs, last_seen FROM peer WHERE name = :name")?;
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
}
