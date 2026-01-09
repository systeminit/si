//! This module provides [`SdfChangeSetTestHelpers`].

use dal::DalContext;
use dal_test::prelude::*;
use sdf_core::dal_wrapper;

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

        ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;
        Ok(())
    }
}
