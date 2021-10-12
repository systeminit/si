use std::{cmp, fmt, io};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug)]
pub struct BytesLinesCodec {
    next_index: usize,
    max_length: usize,
    is_discarding: bool,
}

impl Default for BytesLinesCodec {
    fn default() -> Self {
        Self {
            next_index: 0,
            // Default max line length to attempt of 8MB
            max_length: 8 * 1_024 * 1_024,
            is_discarding: false,
        }
    }
}

impl BytesLinesCodec {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_max_length(max_length: usize) -> Self {
        Self {
            max_length,
            ..Self::default()
        }
    }
}

impl Decoder for BytesLinesCodec {
    type Item = BytesMut;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            let read_to = cmp::min(self.max_length.saturating_add(1), src.len());

            let newline_offset = src[self.next_index..read_to]
                .iter()
                .position(|b| *b == b'\n');

            match (self.is_discarding, newline_offset) {
                (true, Some(offset)) => {
                    // If we found a newline, discard up to that offset and then stop discarding.
                    // On the next iteration, we'll try to read a line normally.
                    src.advance(offset + self.next_index + 1);
                    self.is_discarding = false;
                    self.next_index = 0;
                }
                (true, None) => {
                    // Otherwise, we didn't find a newline, so we'll discard everything we read. On
                    // the next iteration, we'll continue discarding up to max_len bytes unless we
                    // find a newline.
                    src.advance(read_to);
                    self.next_index = 0;
                    if src.is_empty() {
                        return Ok(None);
                    }
                }
                (false, Some(offset)) => {
                    // Found a line
                    let newline_index = offset + self.next_index;
                    self.next_index = 0;
                    let mut line = src.split_to(newline_index + 1);
                    let line = line.split_to(line.len() - 1);
                    let line = without_carriage_return(line);
                    return Ok(Some(line));
                }
                (false, None) if src.len() > self.max_length => {
                    // Reached the maximum length without finding a newline, return an error and
                    // start discarding on the next call.
                    self.is_discarding = true;
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        BytesLinesCodecError { _priv: () },
                    ));
                }
                (false, None) => {
                    // We didn't find a line or reach the length limit, so the next call will
                    // resume searching at the current offset.
                    self.next_index = read_to;
                    return Ok(None);
                }
            }
        }
    }

    fn decode_eof(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        Ok(match self.decode(src)? {
            Some(frame) => Some(frame),
            None => {
                // No terminating newline - return remaining data, if any
                if src.is_empty() || src == &b"\r"[..] {
                    None
                } else {
                    let line = src.split_to(src.len());
                    let line = without_carriage_return(line);
                    self.next_index = 0;
                    Some(line)
                }
            }
        })
    }
}

impl Encoder<Bytes> for BytesLinesCodec {
    type Error = io::Error;

    fn encode(&mut self, data: Bytes, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let n = data.len();

        if n > self.max_length {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                BytesLinesCodecError { _priv: () },
            ));
        }

        // Reserve capacity in the destination buffer to fit the frame and newline
        dst.reserve(n + 1);
        // Write the frame to the buffer
        dst.extend_from_slice(&data[..]);
        dst.put_u8(b'\n');

        Ok(())
    }
}

pub struct BytesLinesCodecError {
    _priv: (),
}

impl fmt::Debug for BytesLinesCodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BytesLinesCodecError").finish()
    }
}

impl fmt::Display for BytesLinesCodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("frame size too big")
    }
}

impl std::error::Error for BytesLinesCodecError {}

fn without_carriage_return(mut src: BytesMut) -> BytesMut {
    if let Some(&b'\r') = src.last() {
        src.split_to(src.len() - 1)
    } else {
        src
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use futures::{SinkExt, TryStreamExt};
    use serde::{Deserialize, Serialize};
    use tokio::net::{TcpListener, TcpStream};
    use tokio_serde::formats::Json;
    use tokio_util::codec::Framed;

    use super::*;

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct Request {
        body: String,
        cool: bool,
    }

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct Response {
        body: String,
        cool: bool,
    }

    struct Server {
        listener: TcpListener,
    }

    impl Server {
        async fn new() -> Self {
            Self {
                listener: TcpListener::bind("127.0.0.1:0").await.unwrap(),
            }
        }

        fn local_addr(&self) -> SocketAddr {
            self.listener
                .local_addr()
                .expect("failed to get local addr")
        }

        async fn run(self) {
            loop {
                let (socket, _) = self
                    .listener
                    .accept()
                    .await
                    .expect("server failed to accept connection");

                let codec = Framed::new(socket, BytesLinesCodec::new());
                let mut stream: tokio_serde::Framed<_, Request, Response, _> =
                    tokio_serde::Framed::new(codec, Json::<Request, Response>::default());

                let req = match stream.try_next().await.expect("server failed next msg") {
                    Some(req) => req,
                    None => panic!("server tried to get message and there was none"),
                };

                let res = Response {
                    body: req.body,
                    cool: !req.cool,
                };
                stream.send(res).await.expect("failed send msg");
            }
        }
    }

    async fn client(local_addr: SocketAddr, req: Request) -> Response {
        let socket = TcpStream::connect(local_addr)
            .await
            .expect("client failed to connect");

        let codec = Framed::new(socket, BytesLinesCodec::new());
        let mut stream = tokio_serde::Framed::new(codec, Json::<Response, Request>::default());

        stream.send(req).await.expect("client failed to send msg");

        let res = stream
            .try_next()
            .await
            .expect("client failed to get msg")
            .expect("there was no msg");
        res
    }

    #[tokio::test]
    async fn it_works() {
        let server = Server::new().await;
        let local_addr = server.local_addr();
        tokio::spawn(server.run());

        let req = Request {
            body: "Mondays".to_string(),
            cool: true,
        };

        let expected = Response {
            body: "Mondays".to_string(),
            cool: false,
        };

        let res = client(local_addr, req).await;

        assert_eq!(res, expected);
    }
}
