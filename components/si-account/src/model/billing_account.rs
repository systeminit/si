use si_data::{DataError, Db};

use crate::error::Result;
pub use crate::protobuf::{
    BillingAccount, BillingAccountSignupReply, BillingAccountSignupRequest, User, UserSiProperties,
};
use tracing::debug;

impl BillingAccount {
    pub async fn billing_account_signup(
        db: &Db,
        request: BillingAccountSignupRequest,
    ) -> Result<BillingAccountSignupReply> {
        debug!("billing account req");
        let billing_account_req = request
            .billing_account
            .ok_or(DataError::RequiredField("billingAccount".into()))?;

        debug!("billing account create");
        let billing_account = BillingAccount::create(
            db,
            billing_account_req.name,
            billing_account_req.display_name,
        )
        .await?;

        debug!("user");
        let user_req = request
            .user
            .ok_or(DataError::RequiredField("user".into()))?;

        debug!("user_si_properties");
        let user_si_properties = UserSiProperties {
            billing_account_id: billing_account.id.clone(),
        };

        debug!("user_create");
        let user = User::create(
            db,
            user_req.name,
            user_req.display_name,
            user_req.email,
            user_req.password,
            Some(user_si_properties),
        )
        .await?;

        // TODO: Finish stealing all the rest of the createAccount

        debug!("replied");
        Ok(BillingAccountSignupReply {
            billing_account: Some(billing_account),
            user: Some(user),
        })
    }
}
