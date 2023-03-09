use dal::{key_pair::PublicKey, DalContext, KeyPair, Tenancy};
use dal_test::{
    test,
    test_harness::{create_key_pair, create_workspace},
};

#[test]
async fn new(ctx: &DalContext) {
    let _key_pair = KeyPair::new(ctx, "funky")
        .await
        .expect("cannot create key_pair");
}

#[test]
async fn belongs_to(ctx: &mut DalContext) {
    let workspace = create_workspace(ctx).await;
    ctx.update_tenancy(Tenancy::new(*workspace.pk()));

    let key_pair = create_key_pair(ctx).await;
    let belongs_to_wo = key_pair
        .workspace(ctx)
        .await
        .expect("cannot get belongs to workspace");
    assert_eq!(&workspace, &belongs_to_wo);
}

#[test]
async fn public_key_get_current(ctx: &mut DalContext) {
    let workspace = create_workspace(ctx).await;
    ctx.update_tenancy(Tenancy::new(*workspace.pk()));

    let first_key_pair = create_key_pair(ctx).await;
    let pk = PublicKey::get_current(ctx)
        .await
        .expect("cannot get public key");
    assert_eq!(first_key_pair.pk(), *pk.pk());
    assert_eq!(first_key_pair.public_key(), pk.public_key());

    let second_key_pair = create_key_pair(ctx).await;
    let pk = PublicKey::get_current(ctx)
        .await
        .expect("cannot get public key");

    assert_eq!(second_key_pair.pk(), *pk.pk());
    assert_eq!(second_key_pair.public_key(), pk.public_key());
}
