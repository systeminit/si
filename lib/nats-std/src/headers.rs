use si_data_nats::{
    HeaderMap,
    Subject,
};

pub const REPLY_INBOX: &str = "X-Reply-Inbox";

pub fn insert_reply_inbox(headers: &mut HeaderMap, reply_inbox: &str) {
    headers.insert(REPLY_INBOX, reply_inbox);
}

pub fn insert_maybe_reply_inbox(headers: &mut HeaderMap, maybe_reply_inbox: Option<&Subject>) {
    if let Some(reply_inbox) = maybe_reply_inbox {
        headers.insert(REPLY_INBOX, reply_inbox.as_str());
    }
}
