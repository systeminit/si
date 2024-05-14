//! This module provides utilities for parsing [NATS](https://nats.io) subjects.

use si_data_nats::subject::ToSubject;
use std::collections::HashSet;

/// This function combs through a given [`subject`](si_data_nats::subject::Subject) via
/// [dot notation](https://docs.nats.io/nats-concepts/subjects) in order to find all potential
/// [`MultiplexerKeys`](nats_multiplexer_core::MultiplexerKey) that may exist within the
/// [`Multiplexer`](crate::Multiplexer).
///
/// **Warning: currently, this only supports using wildcards in the final token and only if there are at least two
/// tokens found. Technically, with [NATS](https://nats.io), you can mix and match dot operators and use them at
/// different token locations. This also isn't going to cover every single use case and may be flimsy. Please be kind.**
pub(crate) fn keys_for_potential_receivers(subject: impl ToSubject) -> HashSet<String> {
    // Convert to a subject and then into a string to ensure we are dealing a valid subject.
    let raw_subject = subject.to_subject().to_string();

    // This does not cover all cases, but avoids collisions with the reserved "$" symbol as well as starting off with a
    // wildcard ("*" or ">") or token separator (".") by accident.
    if let Some(first_character) = raw_subject.chars().next() {
        if !first_character.is_alphanumeric() {
            return HashSet::new();
        }
    } else {
        return HashSet::new();
    }

    // Collect a list of keys for potential receivers. The first one we populate is if the key is identifcal to the
    // provided subject. After this, we need to find those using wildcards ("*" and ">") and insert them into this set.
    let mut keys_for_potential_receivers = HashSet::from([raw_subject.clone()]);

    // Split the raw subject using dot notation. Very academic.
    let raw_subject_split = raw_subject.split('.');

    // Cache the current and previous token. We'll need the "previous" one for the final "*" wildcard case.
    let mut current = "".to_string();
    let mut previous = "".to_string();

    for token in raw_subject_split {
        if current.is_empty() {
            current = token.to_string();
            continue;
        }

        // Once we have a populated "current" token, this wildcard is valid.
        keys_for_potential_receivers.insert(format!("{current}.>"));

        // Before entering the next iteration, set the "previous" and "current'.
        previous.clone_from(&current);
        current = format!("{current}.{token}");
    }

    // We need to handle the "*" wildcard case once the split is empty.
    if !previous.is_empty() {
        keys_for_potential_receivers.insert(format!("{previous}.*"));
    }

    keys_for_potential_receivers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_for_potential_receivers_common() {
        let expected = HashSet::from(
            [
                "si.>",
                "si.workspace_pk.>",
                "si.workspace_pk.8675309.>",
                "si.workspace_pk.8675309.*",
                "si.workspace_pk.8675309.event",
            ]
            .map(|s| s.to_string()),
        );
        let actual = keys_for_potential_receivers("si.workspace_pk.8675309.event");
        assert_eq!(expected, actual);

        let expected = HashSet::from(
            [
                "si.>",
                "si.workspace_pk.>",
                "si.workspace_pk.*",
                "si.workspace_pk.8675309",
            ]
            .map(|s| s.to_string()),
        );
        let actual = keys_for_potential_receivers("si.workspace_pk.8675309");
        assert_eq!(expected, actual);

        let expected = HashSet::from(["si.>", "si.*", "si.workspace_pk"].map(|s| s.to_string()));
        let actual = keys_for_potential_receivers("si.workspace_pk");
        assert_eq!(expected, actual);
    }

    #[test]
    fn keys_for_potential_receivers_no_dot_notation() {
        let expected = HashSet::from(["si"].map(|s| s.to_string()));
        let actual = keys_for_potential_receivers("si");
        assert_eq!(expected, actual);

        let expected =
            HashSet::from(["crdt-00000000000000000000000000-8675309"].map(|s| s.to_string()));
        let actual = keys_for_potential_receivers("crdt-00000000000000000000000000-8675309");
        assert_eq!(expected, actual);
    }

    #[test]
    fn keys_for_potential_receivers_bad_input() {
        let expected = HashSet::new();
        let actual = keys_for_potential_receivers("");
        assert_eq!(expected, actual);

        let expected = HashSet::new();
        let actual = keys_for_potential_receivers("*");
        assert_eq!(expected, actual);

        let expected = HashSet::new();
        let actual = keys_for_potential_receivers(">");
        assert_eq!(expected, actual);

        let expected = HashSet::new();
        let actual = keys_for_potential_receivers("?");
        assert_eq!(expected, actual);

        let expected = HashSet::new();
        let actual = keys_for_potential_receivers(".");
        assert_eq!(expected, actual);
    }
}
