use std::{
    fs,
    sync::{Arc, Mutex, MutexGuard},
};

use anyhow::{Context, Result};
use libp2p::{Multiaddr, PeerId, Swarm};
use rusqlite::Connection;
use tracing::debug;

use crate::network::Behaviour;

use self::peer::ScsPeer;

pub mod peer;

#[derive(Debug)]
pub struct Store {
    // db_path: PathBuf,
    conn: Arc<Mutex<Connection>>,
}

impl Store {
    pub fn initialize() -> Result<Store> {
        debug!("Initializing Database Connection");
        let dirs =
            directories_next::ProjectDirs::from("com", "onboardbase", "secureshare").unwrap();
        let path = dirs.data_local_dir();
        fs::create_dir_all(path).context("Failed to create default directory")?;
        let path = path.join("scs.db3");
        let conn = Connection::open(path)?;

        debug!("Preparing to execute schema");
        conn.execute(
            "CREATE TABLE IF NOT EXISTS peer (
            id    INTEGER PRIMARY KEY,
            name  TEXT NOT NULL UNIQUE,
            addrs  BLOB,
            peer_id TEXT NOT NULL UNIQUE,
            last_seen TEXT
        )",
            (), // empty list of parameters.
        )?;
        debug!("Executed schema creation for peer");

        let settings = Store {
            conn: Arc::new(Mutex::new(conn)),
        };
        Ok(settings)
    }

    pub fn get_conn_handle(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().unwrap()
    }

    pub fn store_peer(&self, swarm: &mut Swarm<Behaviour>, peer_id: PeerId) -> Result<()> {
        debug!("Initiating Peer Storage");
        let addrs = swarm.external_addresses().collect::<Vec<_>>();
        //FIXME chnage this to the connector address
        let addr = <&Multiaddr>::clone(addrs.first().unwrap());
        let name = "testing_t".to_string();
        let peer = ScsPeer::from((addr, name, peer_id));
        peer.save(self)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::Store;

    #[test]
    fn initialize_db() -> Result<()> {
        let settings = Store::initialize()?;
        let conn = settings.get_conn_handle();
        assert!(conn.is_autocommit());
        Ok(())
    }
}
