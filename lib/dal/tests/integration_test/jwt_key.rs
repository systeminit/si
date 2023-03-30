use dal::{jwt_key, DalContext, JwtSecretKey};
use dal_test::test;

// {get_jwt_signing_key, get_jwt_validation_key};

#[test]
async fn get_jwt_signing_key(ctx: &DalContext, jwt_secret_key: &JwtSecretKey) {
    let _signing_key = jwt_key::get_jwt_signing_key(ctx, jwt_secret_key)
        .await
        .expect("cannot get jwt signing key");
}
