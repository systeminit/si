use std::sync::Arc;
use std::net::SocketAddr;

use thiserror::Error;
use tokio::{net::UdpSocket, sync::mpsc};
use crate::SiId;

use super::SI_ID_GENERATOR;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("io error: `{0}`")]
    Io(#[from] std::io::Error),
    #[error("error generating snowflake: {0}")]
    Snowdon(#[from] snowdon::Error),
}

pub type ServerResult<T> = Result<T, ServerError>;

pub async fn run_server() -> ServerResult<()> {
    let sock = UdpSocket::bind("0.0.0.0:7765").await?;
    let recv = Arc::new(sock);
    let send = recv.clone();
    let (tx, mut rx) = mpsc::channel::<(Vec<u8>, SocketAddr)>(1_000);

    tokio::spawn(async move {
        while let Some((_bytes, addr)) = rx.recv().await {
            match send_new_id(send.clone(), addr).await {
                Err(ServerError::Snowdon(e)) => {
                    panic!("Error creating snowflake; this is unrecoverable. {:?}", e);
                }
                Ok(_) => {},
                Err(e) => {
                    println!("Error sending snowflake response: {:?}", e);
                }
            }
        }
    });

    let mut buf = [0; 1];
    loop {
        let (len, addr) = recv.recv_from(&mut buf).await?;
        tx.send((buf[..len].to_vec(), addr)).await.unwrap();
    }
}

async fn send_new_id(send: Arc<UdpSocket>, addr: SocketAddr) -> ServerResult<SiId> {
    let id = SI_ID_GENERATOR.generate()?;
    send.send_to(&id.into_inner().to_be_bytes(), &addr).await?;
    Ok(id)
}
