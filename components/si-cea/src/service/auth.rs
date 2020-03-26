use std::convert::TryFrom;

use si_account::{authorize::authorize, BillingAccount};
use si_data::Db;

use crate::error::{CeaError, Result};

#[derive(Debug)]
pub struct Authentication {
    user_id: String,
    billing_account_id: String,
}

impl Authentication {
    pub fn new(user_id: impl Into<String>, billing_account_id: impl Into<String>) -> Self {
        Authentication {
            user_id: user_id.into(),
            billing_account_id: billing_account_id.into(),
        }
    }

    pub fn billing_account_id(&self) -> &str {
        self.billing_account_id.as_ref()
    }

    pub fn user_id(&self) -> &str {
        self.user_id.as_ref()
    }

    pub async fn billing_account(&self, db: &Db) -> Result<BillingAccount> {
        let billing_account: BillingAccount = db.get(self.billing_account_id()).await?;
        Ok(billing_account)
    }

    pub async fn authorize_on_billing_account(
        &self,
        db: &Db,
        action: impl Into<&str>,
    ) -> Result<()> {
        let ba = self.billing_account(db).await?;
        authorize(
            db,
            self.user_id(),
            self.billing_account_id(),
            action.into(),
            &ba,
        )
        .await?;
        Ok(())
    }
}

impl<T> TryFrom<&tonic::Request<T>> for Authentication {
    type Error = CeaError;

    fn try_from(request: &tonic::Request<T>) -> std::result::Result<Self, Self::Error> {
        let metadata = request.metadata();
        let user_id = metadata
            .get("userId")
            .ok_or(CeaError::InvalidAuthenticationMissingUserId)?
            .to_str()?;
        let billing_account_id = metadata
            .get("billingAccountId")
            .ok_or(CeaError::InvalidAuthenticationMissingBillingAccountId)?
            .to_str()?;
        Ok(Authentication::new(user_id, billing_account_id))
    }
}
