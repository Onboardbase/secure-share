use core::ascii;
use std::{
    fs, io,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str,
    sync::Arc,
};

//Acts as a receiver
use crate::common::ALPN_QUIC_HTTP;
use anyhow::{anyhow, bail, Context, Result};
use quinn::{Connecting, ConnectionError, Endpoint, ServerConfig};
use tracing::{error, info, info_span};
use tracing_futures::Instrument as _;

pub async fn run() -> Result<()> {
    //TODO remove this and make it an argument
    let listen: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4433);

    let dirs = directories_next::ProjectDirs::from("com", "onboardbase", "sharebase").unwrap();
    let path = dirs.data_local_dir();
    let certificate_path = path.join("cert.der");
    let key_path = path.join("key.der");

    let (certificate, key) =
        match fs::read(&certificate_path).and_then(|path| Ok((path, fs::read(&key_path)?))) {
            Ok(z) => z,
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                info!("Generating self signed certificate");
                let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
                let key = cert.serialize_private_key_der();
                let cert = cert.serialize_der()?;
                fs::create_dir_all(path).context("failed to create certificate directory")?;
                fs::write(&certificate_path, &cert).context("failed to write certificate")?;
                fs::write(&key_path, &key).context("failed to write private key")?;
                (cert, key)
            }
            Err(e) => {
                bail!("failed to read certificate: {}", e);
            }
        };
    let key = rustls::PrivateKey(key);
    let certificates = vec![rustls::Certificate(certificate)];

    let mut server_crypto = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certificates, key)?;
    server_crypto.alpn_protocols = ALPN_QUIC_HTTP.iter().map(|&x| x.into()).collect();

    let mut server_config = ServerConfig::with_crypto(Arc::new(server_crypto));
    let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    transport_config.max_concurrent_uni_streams(0_u8.into());

    let endpoint = Endpoint::server(server_config, listen)?;
    eprintln!("Listening on {}", endpoint.local_addr()?);

    while let Some(conn) = endpoint.accept().await {
        info!("connection incoming");
        let fut = handle_connection(conn);
        tokio::spawn(async move {
            if let Err(e) = fut.await {
                error!("connection failed: {reason}", reason = e.to_string())
            }
        });
    }
    Ok(())
}

async fn handle_connection(connection: Connecting) -> Result<()> {
    let conn = connection.await?;
    let span = info_span!(
        "connection",
        remote = %conn.remote_address(),
        protocol = %conn
            .handshake_data()
            .unwrap()
            .downcast::<quinn::crypto::rustls::HandshakeData>().unwrap()
            .protocol
            .map_or_else(|| "<none>".into(), |x| String::from_utf8_lossy(&x).into_owned())
    );

    async {
        info!("Connection Established");

        // Each stream from the client means a new request.
        loop {
            let stream = conn.accept_bi().await;
            let stream = match stream {
                Err(ConnectionError::ApplicationClosed { .. }) => {
                    info!("connection closed");
                    return Ok(());
                }
                Err(e) => {
                    return Err(e);
                }
                Ok(s) => s,
            };
            let fut = handle_request(stream);
            tokio::spawn(
                async move {
                    if let Err(e) = fut.await {
                        error!("Failed: {reason}", reason = e.to_string());
                    }
                }
                .instrument(info_span!("request")),
            );
        }
    }
    .instrument(span)
    .await?;

    Ok(())
}

//send and recv are destructured from `stream`
async fn handle_request(
    (mut send, mut recv): (quinn::SendStream, quinn::RecvStream),
) -> Result<()> {
    let req = recv
        .read_to_end(64 * 1024)
        .await
        .map_err(|e| anyhow!("failed reading request: {}", e))?;
    let mut escaped = String::new();
    for &x in &req[..] {
        let part = ascii::escape_default(x).collect::<Vec<_>>();
        escaped.push_str(str::from_utf8(&part).unwrap());
    }
    info!(content = %escaped);

    // Execute the request
    let resp = "hi there".as_bytes();
    // Write the response
    send.write_all(resp)
        .await
        .map_err(|e| anyhow!("failed to send response: {}", e))?;
    // Gracefully terminate the stream
    send.finish()
        .await
        .map_err(|e| anyhow!("failed to shutdown stream: {}", e))?;
    info!("complete");

    Ok(())
}
