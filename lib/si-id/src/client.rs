use std::{
    array::TryFromSliceError, net::{AddrParseError, SocketAddr}, sync::Arc
};

use thiserror::Error;
use tokio::net::UdpSocket;

use crate::SiId;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("io error: `{0}`")]
    Io(#[from] std::io::Error),
    #[error("invalid local binding address: {0}")]
    Address(#[from] AddrParseError),
    #[error("Cannot make a u64 from the data; bug!")]
    NoU64(#[from] TryFromSliceError),
    #[error("u64 is not a valid SiId: {0}")]
    Snowdon(#[from] snowdon::Error),
    #[error("did not read 8 bytes from the server; bug!")]
    ShortRead(usize),
}

pub type ClientResult<T> = Result<T, ClientError>;

#[derive(Debug)]
pub struct SiIdClient {
    remote_addr: Arc<SocketAddr>,
}

impl SiIdClient {
    pub async fn new(remote_addr: SocketAddr) -> ClientResult<SiIdClient> {
        Ok(SiIdClient {
            remote_addr: Arc::new(remote_addr),
        })
    }

    pub async fn get_id(&self) -> ClientResult<SiId> {
        // We use port 0 to let the operating system allocate an available port for us.
        let local_addr: SocketAddr = if self.remote_addr.is_ipv4() {
            "0.0.0.0:0"
        } else {
            "[::]:0"
        }
        .parse()?;
        let socket = UdpSocket::bind(local_addr).await?;
        socket.connect(*self.remote_addr).await?;
        socket.send(&[0]).await?;
        let mut data = vec![0u8; 8];
        let len = socket.recv(&mut data).await?;
        if len != 8 {
            return Err(ClientError::ShortRead(len));
        }
        let si_id_u64: u64 = u64::from_be_bytes(data[..].try_into()?);
        let si_id = SiId::from_raw(si_id_u64)?;
        Ok(si_id)
    }
}
