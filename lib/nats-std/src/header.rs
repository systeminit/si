use si_data_nats::{
    HeaderMap,
    HeaderValue,
    Subject,
    async_nats,
    header::IntoHeaderValue,
};

pub mod value;

// X-CONTENT-ENCODING: gzip
pub const CONTENT_ENCODING: &str = "X-CONTENT-ENCODING";

// X-CONTENT-TYPE: application/json
pub const CONTENT_TYPE: &str = "X-CONTENT-TYPE";

// X-MESSAGE-TYPE: EnqueueRequest
pub const MESSAGE_TYPE: &str = "X-MESSAGE-TYPE";

// X-MESSAGE-VERSION: 1
pub const MESSAGE_VERSION: &str = "X-MESSAGE-VERSION";

// X-REPLY-INBOX: _INBOX.3wJ4MnwZ8xRSBAaTbwa2t6
pub const REPLY_INBOX: &str = "X-Reply-Inbox";

#[inline]
pub fn insert_content_encoding(headers: &mut HeaderMap, value: impl IntoHeaderValue) {
    headers.insert(CONTENT_ENCODING, value);
}

#[inline]
pub fn content_encoding(maybe_headers: Option<&HeaderMap>) -> Option<&HeaderValue> {
    maybe_headers.and_then(|headers| headers.get(CONTENT_ENCODING))
}

#[inline]
pub fn content_encoding_is(maybe_headers: Option<&HeaderMap>, check: &'static str) -> bool {
    maybe_headers
        .and_then(|headers| headers.get(CONTENT_ENCODING))
        .map(|val| val.as_str() == check)
        .unwrap_or(false)
}

#[inline]
pub fn insert_content_type(headers: &mut HeaderMap, value: impl IntoHeaderValue) {
    headers.insert(CONTENT_TYPE, value);
}

#[inline]
pub fn insert_nats_msg_id(headers: &mut HeaderMap, value: impl IntoHeaderValue) {
    headers.insert(async_nats::header::NATS_MESSAGE_ID, value);
}

#[inline]
pub fn insert_reply_inbox(headers: &mut HeaderMap, reply_inbox: &str) {
    headers.insert(REPLY_INBOX, reply_inbox);
}

#[inline]
pub fn insert_maybe_reply_inbox(headers: &mut HeaderMap, maybe_reply_inbox: Option<&Subject>) {
    if let Some(reply_inbox) = maybe_reply_inbox {
        headers.insert(REPLY_INBOX, reply_inbox.as_str());
    }
}
