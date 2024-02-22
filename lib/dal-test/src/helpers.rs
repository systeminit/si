use color_eyre::Result;
use dal::change_set_pointer::{ChangeSetPointer, ChangeSetPointerId};
use dal::{DalContext, UserClaim};
use jwt_simple::algorithms::RSAKeyPairLike;
use jwt_simple::{claims::Claims, reexports::coarsetime::Duration};
use names::{Generator, Name};

use crate::jwt_private_signing_key;
use crate::signup::WorkspaceSignup;

// pub mod component_bag;

pub fn generate_fake_name() -> String {
    Generator::with_naming(Name::Numbered).next().unwrap()
}

pub async fn create_auth_token(claim: UserClaim) -> String {
    let key_pair = jwt_private_signing_key()
        .await
        .expect("failed to load jwt private signing key");
    let claim = Claims::with_custom_claims(claim, Duration::from_days(1))
        .with_audience("https://app.systeminit.com")
        .with_issuer("https://app.systeminit.com")
        .with_subject(claim.user_pk);

    key_pair.sign(claim).expect("unable to sign jwt")
}

pub async fn workspace_signup(ctx: &DalContext) -> Result<(WorkspaceSignup, String)> {
    use color_eyre::eyre::WrapErr;

    let mut ctx = ctx.clone_with_head();

    let workspace_name = generate_fake_name();
    let user_name = format!("frank {workspace_name}");
    let user_email = format!("{workspace_name}@example.com");

    let nw = WorkspaceSignup::new(&mut ctx, &workspace_name, &user_name, &user_email)
        .await
        .wrap_err("cannot signup a new workspace")?;
    let auth_token = create_auth_token(UserClaim {
        user_pk: nw.user.pk(),
        workspace_pk: *nw.workspace.pk(),
    })
    .await;
    Ok((nw, auth_token))
}

// pub async fn create_user(ctx: &DalContext) -> User {
//     let name = generate_fake_name();
//     User::new(
//         ctx,
//         UserPk::generate(),
//         &name,
//         &format!("{name}@test.systeminit.com"),
//         None::<&str>,
//     )
//     .await
//     .expect("cannot create user")
// }
//

pub async fn create_change_set_and_update_ctx(
    ctx: &mut DalContext,
    base_change_set_id: ChangeSetPointerId,
) {
    let base_change_set = ChangeSetPointer::find(ctx, base_change_set_id)
        .await
        .expect("could not perform find change set")
        .expect("no change set found");
    let mut change_set = ChangeSetPointer::new(ctx, generate_fake_name(), Some(base_change_set_id))
        .await
        .expect("could not create change set pointer");
    change_set
        .update_pointer(
            ctx,
            base_change_set
                .workspace_snapshot_id
                .expect("no workspace snapshot set on base change set"),
        )
        .await
        .expect("could not update pointer");
    ctx.update_visibility_v2(&change_set);
    ctx.update_snapshot_to_visibility()
        .await
        .expect("could not update snapshot to visibility");
}

// /// Get the "si:identity" [`Func`] and execute (if necessary).
// pub async fn setup_identity_func(
//     ctx: &DalContext,
// ) -> (
//     FuncId,
//     FuncBindingId,
//     FuncBindingReturnValueId,
//     FuncArgumentId,
// ) {
//     let identity_func: Func = Func::find_by_attr(ctx, "name", &"si:identity".to_string())
//         .await
//         .expect("could not find identity func by name attr")
//         .pop()
//         .expect("identity func not found");
//
//     let identity_func_identity_arg = FuncArgument::list_for_func(ctx, *identity_func.id())
//         .await
//         .expect("cannot list identity func args")
//         .pop()
//         .expect("cannot find identity func identity arg");
//
//     let (identity_func_binding, identity_func_binding_return_value) =
//         FuncBinding::create_and_execute(
//             ctx,
//             serde_json::json![{ "identity": null }],
//             *identity_func.id(),
//         )
//         .await
//         .expect("could not find or create identity func binding");
//     (
//         *identity_func.id(),
//         *identity_func_binding.id(),
//         *identity_func_binding_return_value.id(),
//         *identity_func_identity_arg.id(),
//     )
// }
