//! This module contains the ability to generate NATS subjects for council and users of council.

use crate::Id;

pub(crate) type PubChannel = String;
pub(crate) type ReplyChannel = String;
pub(crate) type AllChannels = String;
pub(crate) type ManagementChannel = String;
pub(crate) type ManagementReplyChannel = String;

pub(crate) struct SubjectGenerator;

impl SubjectGenerator {
    pub fn for_client(subject_prefix: Option<String>, id: Id) -> (PubChannel, ReplyChannel) {
        let base_subject = Self::base_subject(subject_prefix.clone());

        let pub_channel = format!("{base_subject}.{id}");
        let reply_channel = format!("{pub_channel}.reply");

        (pub_channel, reply_channel)
    }

    pub fn for_management_client(subject_prefix: Option<String>) -> ManagementChannel {
        Self::management_subject(subject_prefix)
    }

    pub fn for_server(
        subject_prefix: Option<String>,
    ) -> (AllChannels, ManagementChannel, ManagementReplyChannel) {
        let all_channels = Self::all_subjects(subject_prefix.clone());
        let management_channel = Self::management_subject(subject_prefix);
        let management_reply_channel = format!("{management_channel}.reply");

        (all_channels, management_channel, management_reply_channel)
    }

    fn management_subject(subject_prefix: Option<String>) -> String {
        let base_subject = Self::base_subject(subject_prefix);
        format!("{base_subject}.management")
    }

    fn all_subjects(subject_prefix: Option<String>) -> String {
        let base_subject = Self::base_subject(subject_prefix);
        format!("{base_subject}.*")
    }

    fn base_subject(subject_prefix: Option<String>) -> String {
        match subject_prefix {
            Some(provided) => format!("{provided}.council"),
            None => "council".to_string(),
        }
    }
}
