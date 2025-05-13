use std::str::FromStr;

use si_events::{
    ChangeSetId,
    WorkspacePk,
};
use thiserror::Error;

pub(crate) struct ParsedWorkspaceId<'a> {
    id: WorkspacePk,
    str: &'a str,
}

impl<'a> ParsedWorkspaceId<'a> {
    pub(crate) fn id(&self) -> WorkspacePk {
        self.id
    }

    pub(crate) fn str(&self) -> &'a str {
        self.str
    }
}

pub(crate) struct ParsedChangeSetId<'a> {
    id: ChangeSetId,
    str: &'a str,
}

impl<'a> ParsedChangeSetId<'a> {
    pub(crate) fn id(&self) -> ChangeSetId {
        self.id
    }

    pub(crate) fn str(&self) -> &'a str {
        self.str
    }
}

#[derive(Debug, Error)]
#[error("failed to parse subject: subject={0}, reason={1}")]
pub(crate) struct SubjectParseError(String, String);

pub(crate) fn parse_subject<'a>(
    subject_prefix: Option<&str>,
    subject_str: &'a str,
) -> Result<(ParsedWorkspaceId<'a>, ParsedChangeSetId<'a>), SubjectParseError> {
    let mut parts = subject_str.split('.');

    if let Some(prefix) = subject_prefix {
        match parts.next() {
            // Prefix part matches expected/configured prefix
            Some(parsed_prefix) if parsed_prefix == prefix => {}
            // Prefix part does not match expected/configured prefix
            Some(unexpected) => {
                return Err(SubjectParseError(
                    subject_str.to_string(),
                    format!(
                        "found unexpected subject prefix; expected={prefix}, parsed={unexpected}"
                    ),
                ));
            }
            // Prefix part not found but expected
            None => {
                return Err(SubjectParseError(
                    subject_str.to_string(),
                    format!("expected subject prefix not found; expected={prefix}"),
                ));
            }
        };
    }

    match (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(), // assert last part is `None` to ensure there are no additional parts
    ) {
        (
            Some(_),
            Some(_),
            Some(workspace_id_str),
            Some(change_set_id_str),
            Some("process"),
            None,
        ) => {
            let workspace_id = WorkspacePk::from_str(workspace_id_str).map_err(|err| {
                SubjectParseError(
                    subject_str.to_string(),
                    format!("workspace id parse error: {err}"),
                )
            })?;
            let change_set_id = ChangeSetId::from_str(change_set_id_str).map_err(|err| {
                SubjectParseError(
                    subject_str.to_string(),
                    format!("change set id parse error: {err}"),
                )
            })?;

            Ok((
                ParsedWorkspaceId {
                    id: workspace_id,
                    str: workspace_id_str,
                },
                ParsedChangeSetId {
                    id: change_set_id,
                    str: change_set_id_str,
                },
            ))
        }
        _ => Err(SubjectParseError(
            subject_str.to_string(),
            "subject failed to parse with unexpected parts".to_string(),
        )),
    }
}
