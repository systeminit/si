use std::collections::HashSet;

use si_frontend_types::reference::ReferenceKind;
use strum::IntoEnumIterator;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReferenceError {}

type Result<T> = std::result::Result<T, ReferenceError>;

pub fn implemented_kinds() -> HashSet<ReferenceKind> {
    let mut implemented_kinds = HashSet::new();
    for reference in ReferenceKind::iter() {
        match reference {
            ReferenceKind::ChangeSetList
            | ReferenceKind::ChangeSetRecord
            | ReferenceKind::MvIndex => {}
            implemented_kind => {
                implemented_kinds.insert(implemented_kind);
            }
        }
    }
    implemented_kinds
}
