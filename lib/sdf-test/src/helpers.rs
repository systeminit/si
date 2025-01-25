//! This module provides [`SdfChangeSetTestHelpers`].

use dal::{ChangeSet, DalContext};
use dal_test::prelude::*;
use sdf_server::dal_wrapper;

/// This unit struct providers helper functions when [dal_test::helpers] cannot perform helper
/// tasks alone (e.g. need to talk to SpiceDB).
#[derive(Debug)]
pub struct SdfTestHelpers;

impl SdfTestHelpers {
    /// Perform a change set apply, but with protections to ensure that all approval requirements
    /// have been met.
    pub async fn protected_apply_change_set_to_base(
        ctx: &mut DalContext,
        spicedb_client: &mut si_data_spicedb::Client,
    ) -> Result<()> {
        // First, check if all requirements have been satisfied.
        dal_wrapper::change_set::approval_requirements_are_satisfied_or_error(ctx, spicedb_client)
            .await?;

        // We need to prepare for apply, but use the new version that skips the status check. Why?
        // Our new approval requirements check above replaces the old status check.
        ChangeSet::prepare_for_apply_without_status_check(ctx).await?;

        // We can mostly follow the DAL test flow for applying a change set while factoring in
        // approvals. The "mostly" part comes from the fact that we need to perform our own
        // "prepare" step above rather than using the one from the DAL test helpers.
        ChangeSetTestHelpers::apply_change_set_to_base_approvals_without_prepare_step(ctx).await?;
        Ok(())
    }
}
