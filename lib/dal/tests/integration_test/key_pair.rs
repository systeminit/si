use dal::{
    key_pair::PublicKey, BillingAccount, BillingAccountPk, DalContext, KeyPair, StandardModel,
};
use dal_test::{
    test,
    test_harness::{create_billing_account, create_key_pair},
};

#[test]
async fn new(ctx: &DalContext, bid: BillingAccountPk) {
    let _key_pair = KeyPair::new(ctx, "funky", bid)
        .await
        .expect("cannot create key_pair");
}

#[test]
async fn belongs_to(ctx: &DalContext) {
    let billing_account = create_billing_account(ctx).await;
    let mut key_pair = create_key_pair(ctx, *billing_account.pk()).await;

    let belongs_to_ba: BillingAccount = key_pair
        .billing_account(ctx)
        .await
        .expect("cannot get belongs to billing account");
    assert_eq!(&billing_account, &belongs_to_ba);

    let billing_account2 = create_billing_account(ctx).await;
    key_pair
        .set_billing_account_pk(ctx, billing_account2.pk())
        .await
        .expect("cannot set billing account 2");

    let belongs_to_ba = key_pair
        .billing_account(ctx)
        .await
        .expect("cannot get belongs to billing account");
    assert_eq!(&billing_account2, &belongs_to_ba);
}

#[test]
async fn public_key_get_current(ctx: &DalContext, bid: BillingAccountPk) {
    let first_key_pair = create_key_pair(ctx, bid).await;
    let pk = PublicKey::get_current(ctx, &bid)
        .await
        .expect("cannot get public key");
    assert_eq!(first_key_pair.pk(), pk.pk());
    assert_eq!(first_key_pair.public_key(), pk.public_key());

    let second_key_pair = create_key_pair(ctx, bid).await;

    let pk = PublicKey::get_current(ctx, &bid)
        .await
        .expect("cannot get public key");

    assert_eq!(second_key_pair.pk(), pk.pk());
    assert_eq!(second_key_pair.public_key(), pk.public_key());
}
