use tracing::{debug, debug_span, warn};
use tracing_futures::Instrument;

pub use crate::error::{AccountError, Result};
pub use crate::protobuf::{BillingAccount, User, UserLoginInternalReply, UserLoginInternalRequest};
pub use si_data::{password::verify_password, DataError, Storable};

impl User {
    pub async fn login_internal(
        db: &si_data::Db,
        request: UserLoginInternalRequest,
    ) -> Result<UserLoginInternalReply> {
        async {
            debug!("get billing account");
            let billing_account = BillingAccount::get_by_natural_key(
                db,
                format!(
                    "global:{}:{}",
                    BillingAccount::type_name(),
                    request
                        .billing_account_name
                        .as_ref()
                        .ok_or_else(|| DataError::ValidationError("billingAccountName".into()))?
                )
                .as_ref(),
            )
            .await?;
            debug!(?billing_account);

            debug!("get user");
            let user = User::get_by_natural_key(
                db,
                format!(
                    "{}:{}:{}",
                    billing_account
                        .id
                        .as_ref()
                        .ok_or_else(|| DataError::ValidationError("billingAccountId".into()))?,
                    User::type_name(),
                    request
                        .email
                        .as_ref()
                        .ok_or_else(|| DataError::ValidationError("email".into()))?
                )
                .as_ref(),
            )
            .await?;
            debug!(?user);

            debug!("verify password");
            let authenticated = verify_password(
                request
                    .password
                    .as_ref()
                    .ok_or_else(|| DataError::ValidationError("password".into()))?,
                user.password
                    .ok_or_else(|| DataError::ValidationError("passwordHash".into()))?,
            );
            debug!(?authenticated);

            if authenticated {
                debug!("login succeded");
                Ok(UserLoginInternalReply {
                    authenticated: Some(authenticated),
                    billing_account_id: Some(
                        billing_account
                            .id
                            .expect("missing billing acount id during login"),
                    ),
                    user_id: Some(user.id.expect("missing user id during login")),
                })
            } else {
                warn!("login failed");
                Ok(UserLoginInternalReply {
                    authenticated: Some(authenticated),
                    ..Default::default()
                })
            }
        }
        .instrument(debug_span!("login_internal"))
        .await
    }
}
