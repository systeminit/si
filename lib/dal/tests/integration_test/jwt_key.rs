use dal::{jwt_key, DalContext, JwtSecretKey};
use dal_test::test;
use jwt_simple::algorithms::RSAKeyPairLike;

// {get_jwt_signing_key, get_jwt_validation_key};

#[test]
async fn get_jwt_signing_key(ctx: &DalContext, jwt_secret_key: &JwtSecretKey) {
    let _signing_key = jwt_key::get_jwt_signing_key(ctx, jwt_secret_key)
        .await
        .expect("cannot get jwt signing key");
}

#[test]
async fn get_jwt_validation_key(ctx: &DalContext, jwt_secret_key: &JwtSecretKey) {
    let signing_key = jwt_key::get_jwt_signing_key(ctx, jwt_secret_key)
        .await
        .expect("cannot get jwt signing key");

    let _validation_key = jwt_key::get_jwt_validation_key(
        ctx,
        signing_key
            .key_id()
            .as_ref()
            .expect("this key should have an id, that it doesn't is a problem"),
    )
    .await
    .expect("cannot get jwt validation key");
}
