use dal::{key_pair::PublicKey, BillingAccount, DalContext, KeyPair, StandardModel};
use dal_test::{
    test,
    test_harness::{create_billing_account, create_key_pair},
};

#[test]
async fn new(ctx: &DalContext) {
    let _key_pair = KeyPair::new(ctx, "funky")
        .await
        .expect("cannot create key_pair");
}

#[test]
async fn belongs_to(ctx: &DalContext) {
    let billing_account = create_billing_account(ctx).await;
    let key_pair = create_key_pair(ctx).await;

    key_pair
        .set_billing_account(ctx, billing_account.id())
        .await
        .expect("cannot set billing account");

    let belongs_to_ba: BillingAccount = key_pair
        .billing_account(ctx)
        .await
        .expect("cannot get belongs to billing account")
        .expect("billing account should exist");
    assert_eq!(&billing_account, &belongs_to_ba);

    key_pair
        .unset_billing_account(ctx)
        .await
        .expect("cannot set billing account");

    let belongs_to_ba: Option<BillingAccount> = key_pair
        .billing_account(ctx)
        .await
        .expect("cannot get belongs to billing account");
    assert!(
        belongs_to_ba.is_none(),
        "billing account is not associated anymore"
    );
}

#[test]
async fn public_key_get_current(ctx: &DalContext) {
    let billing_account = create_billing_account(ctx).await;
    let first_key_pair = create_key_pair(ctx).await;
    first_key_pair
        .set_billing_account(ctx, billing_account.id())
        .await
        .expect("cannot set billing account");
    let second_key_pair = create_key_pair(ctx).await;
    second_key_pair
        .set_billing_account(ctx, billing_account.id())
        .await
        .expect("cannot set billing account");

    let pk = PublicKey::get_current(ctx, billing_account.id())
        .await
        .expect("cannot get public key");

    assert_eq!(second_key_pair.pk(), pk.pk());
    assert_eq!(second_key_pair.public_key(), pk.public_key());
}
