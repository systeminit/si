//! This module contains [`SubjectGenerator`] which is used to centralize subject naming for
//! "rebaser" client and server setup and communication.

use ulid::Ulid;

/// A generator that provides subject names in a centralized location.
#[allow(missing_debug_implementations)]
pub struct SubjectGenerator;

impl SubjectGenerator {
    /// Returns the root subject for all rebaser-related-messages.
    pub fn root(subject_prefix: Option<impl AsRef<str>>) -> String {
        Self::assemble_with_prefix("rebaser", subject_prefix)
    }

    /// Returns the subject covering all rebaser-related messages.
    pub fn all(subject_prefix: Option<impl AsRef<str>>) -> String {
        Self::assemble_with_prefix("rebaser.>", subject_prefix)
    }

    /// Returns the subject used for publishing a rebase request.
    pub fn request(
        workspace_id: Ulid,
        change_set_id: Ulid,
        subject_prefix: Option<impl AsRef<str>>,
    ) -> String {
        Self::assemble_with_prefix(
            format!("rebaser.ws.{workspace_id}.cs.{change_set_id}"),
            subject_prefix,
        )
    }

    fn assemble_with_prefix(
        base_subject_name: impl AsRef<str>,
        maybe_subject_prefix: Option<impl AsRef<str>>,
    ) -> String {
        let base_subject_name = base_subject_name.as_ref();
        if let Some(subject_prefix) = maybe_subject_prefix {
            format!("{}-{base_subject_name}", subject_prefix.as_ref())
        } else {
            base_subject_name.to_string()
        }
    }
}
