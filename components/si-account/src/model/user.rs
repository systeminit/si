use sodiumoxide::crypto::pwhash::argon2id13;
use tracing::trace;

pub use crate::error::{AccountError, Result};
pub use crate::protobuf::{User, UserInitialCreateReply, UserInitialCreateRequest};

impl User {
    pub async fn user_initial_create(
        db: &si_data::Db,
        request: UserInitialCreateRequest,
    ) -> Result<UserInitialCreateReply> {
        Ok(UserInitialCreateReply {
            ..Default::default()
        })
    }

    #[tracing::instrument]
    pub fn verify_password(&self, password: &str) -> bool {
        //let password_bytes = password.as_bytes();
        //if let Some(argon_password) =
        //    argon2id13::HashedPassword::from_slice(self.password_hash.as_bytes())
        //{
        //    if argon2id13::pwhash_verify(&argon_password, password_bytes) {
        //        trace!("correct password");
        //        true
        //    } else {
        //        trace!("incorrect password");
        //        false
        //    }
        //} else {
        //    trace!("corrupt password hash in database");
        //    false
        //}
        //
        true
    }
}
