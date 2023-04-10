use dal::{DalContext, JwtSecretKey, Tenancy, WorkspacePk};
use dal_test::{helpers::workspace_signup, test};

#[test]
async fn check_workspace_pk_identical(ctx: &mut DalContext, jwt_secret_key: &JwtSecretKey) {
    let (nw, _) = workspace_signup(ctx, jwt_secret_key)
        .await
        .expect("cannot signup new workspace");
    let tenancy = Tenancy::new(*nw.workspace.pk());

    let check = tenancy
        .check(ctx.txns().await.expect("failed to get txns").pg(), &tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_workspace_pk_mismatched(ctx: &mut DalContext, jwt_secret_key: &JwtSecretKey) {
    let (nw, _) = workspace_signup(ctx, jwt_secret_key)
        .await
        .expect("cannot signup new workspace");
    let tenancy = Tenancy::new(*nw.workspace.pk());
    let other_tenancy = Tenancy::new(WorkspacePk::NONE);

    let check = tenancy
        .check(
            ctx.txns().await.expect("failed to get txns").pg(),
            &other_tenancy,
        )
        .await
        .expect("cannot check tenancy");
    assert!(!check);

    let check = other_tenancy
        .check(ctx.txns().await.expect("failed to get txns").pg(), &tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}
