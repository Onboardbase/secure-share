///Acts as a receiver
use core::ascii;
use std::{
    fs::{self, File},
    io::{self, BufWriter, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str,
    sync::Arc,
};

use crate::{common::ALPN_QUIC_HTTP, secret::Secret};
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
    let resp = process_request(&req).unwrap_or_else(|e| {
        error!("failed: {}", e);
        format!("failed to process request: {e}\n").into_bytes()
    });

    // Write the response
    send.write_all(&resp)
        .await
        .map_err(|e| anyhow!("failed to send response: {}", e))?;
    // Gracefully terminate the stream
    send.finish()
        .await
        .map_err(|e| anyhow!("failed to shutdown stream: {}", e))?;
    info!("complete");

    Ok(())
}

fn process_request(x: &[u8]) -> Result<Vec<u8>> {
    if x.len() < 4 || &x[0..4] != b"GET " {
        bail!("missing GET");
    }

    if x[4..].len() < 2 || &x[x.len() - 2..] != b"\r\n" {
        bail!("missing \\r\\n");
    }
    let x = &x[4..x.len() - 2];
    //The logic here is that since I know that `x.len() - 2` gives me the end of bytes, just before the line break and
    //return characters(\r\n). That means this position is also the end of the secerts in request body
    //That being said, the request body is delimited by "\r\n". And the secrets start on a new line right after the
    //GET and url. Getting the position of the first occurence of "\r\n" and then slicing x up to `x.len() -2" should
    //give me the secrets.

    let secret_start_position = x.iter().position(|&c| c == b'\n').unwrap_or(x.len());
    let secrets = &x[secret_start_position + 1..x.len()];
    let secrets = str::from_utf8(secrets).context("Secrets is malformed UTF-8")?;
    println!("secrets=={secrets}");

    let secrets_json: Vec<String> =
        serde_json::from_str(secrets).context("Failed to format secrets to JSON")?;
    let secrets_json = Secret::secrets_from_string(secrets_json);

    let dirs = directories_next::ProjectDirs::from("com", "onboardbase", "sharebase").unwrap();
    let path = dirs.data_local_dir();
    let secret_default_path = path.join("secrets.json");
    let secrets_file =
        File::create(secret_default_path).context("Failed to open secrets file storage")?;
    let mut writer = BufWriter::new(secrets_file);
    serde_json::to_writer(&mut writer, &secrets_json)?;
    writer.flush().context("Failed to save secrets")?;

    Ok("SUCCESS".as_bytes().to_vec())
}
