use base64;
use prost::Message;
use serde::{Deserialize, Serialize};
use serde_cbor;
use sodiumoxide::crypto::secretbox;

use crate::data::DataPageToken;
use crate::error::{DataError, Result};

#[derive(Debug, Serialize, Deserialize)]
struct Token {
    nonce: secretbox::Nonce,
    ciphertext: Vec<u8>,
}

impl DataPageToken {
    pub fn seal(&self, key: &secretbox::Key) -> Result<String> {
        let nonce = secretbox::gen_nonce();
        let mut buffer = Vec::new();

        self.encode(&mut buffer)?;
        let ciphertext = secretbox::seal(&buffer, &nonce, &key);
        let token = Token {
            nonce: nonce,
            ciphertext: ciphertext,
        };
        let cbordata = serde_cbor::to_vec(&token)?;
        let base64text = base64::encode(&cbordata);
        Ok(base64text)
    }

    pub fn unseal(token: &str, key: &secretbox::Key) -> Result<DataPageToken> {
        let cbor_token = base64::decode(token)?;
        let token: Token = serde_cbor::from_slice(&cbor_token)?;
        let protobuf_page_token = secretbox::open(&token.ciphertext, &token.nonce, key)
            .map_err(|_| DataError::SodiumOxideOpen)?;
        let page_token: DataPageToken = prost::Message::decode(&protobuf_page_token[..])?;
        Ok(page_token)
    }
}
