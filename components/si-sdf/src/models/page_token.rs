use base64;
use serde::{Deserialize, Serialize};
use serde_json;
use sodiumoxide::crypto::secretbox;
use thiserror::Error;

// TODO: your mission, should you choose to accept it, is to re-wire page tokens in to a new,
//       generic, raw JSON list handler for the API. Then wire it to the front-end model in
//       a nice way, and then get the workspace/organization/billing account information
//       back into the web ui.

#[derive(Error, Debug)]
pub enum PageTokenError {
    #[error("failed to decrypt a page token")]
    Decrypt,
    #[error("failed to serialize/deserialize the token: {0}")]
    Json(#[from] serde_json::Error),
    #[error("failed to decode base64 string: {0}")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("failed to create a string from utf-8: {0}")]
    StringUtf8(#[from] std::string::FromUtf8Error),
}

pub type PageTokenResult<T> = Result<T, PageTokenError>;

use crate::models::{OrderByDirection, Query};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageToken {
    pub query: Option<Query>,
    pub page_size: u32,
    pub order_by: String,
    pub order_by_direction: OrderByDirection,
    pub item_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct InternalToken {
    nonce: secretbox::Nonce,
    ciphertext: Vec<u8>,
}

impl PageToken {
    pub fn seal(&self, key: &secretbox::Key) -> PageTokenResult<String> {
        let nonce = secretbox::gen_nonce();
        let page_token_json = serde_json::to_string(self)?;
        let ciphertext = secretbox::seal(&page_token_json.as_bytes(), &nonce, &key);
        let internal_token = InternalToken { nonce, ciphertext };
        let internal_token_json = serde_json::to_string(&internal_token)?;
        let base64text = base64::encode_config(&internal_token_json, base64::URL_SAFE_NO_PAD);
        Ok(base64text)
    }

    pub fn unseal(token: &str, key: &secretbox::Key) -> PageTokenResult<PageToken> {
        let internal_token_json_vec = base64::decode_config(token, base64::URL_SAFE_NO_PAD)?;
        let internal_token_json_string = String::from_utf8(internal_token_json_vec)?;
        let internal_token: InternalToken = serde_json::from_str(&internal_token_json_string)?;
        let page_token_json_vec =
            secretbox::open(&internal_token.ciphertext, &internal_token.nonce, key)
                .map_err(|_| PageTokenError::Decrypt)?;
        let page_token_json_string = String::from_utf8(page_token_json_vec)?;
        let page_token: PageToken = serde_json::from_str(&page_token_json_string)?;
        Ok(page_token)
    }
}
