use color_eyre::Result;
use dal::change_set::{ChangeSet, ChangeSetId};
use dal::secret::DecryptedSecret;
use dal::{DalContext, UserClaim};
use jwt_simple::algorithms::RSAKeyPairLike;
use jwt_simple::{claims::Claims, reexports::coarsetime::Duration};
use names::{Generator, Name};
use serde_json::Value;

use crate::jwt_private_signing_key;
use crate::signup::WorkspaceSignup;

use crate::tracing::subscriber;
use crate::tracing_subscriber::fmt;
use crate::tracing_subscriber::EnvFilter;
use crate::tracing_subscriber::Registry;

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

    let mut ctx = ctx.clone_with_head().await?;

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

pub fn tracing_init(span_events_env_var: &'static str, log_env_var: &'static str) {
    use std::thread;

    thread::spawn(move || {
        let tokio = ::tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime for Tracing for Testing");

        tokio.block_on(async move {
            tracing_init_inner(span_events_env_var, log_env_var);
            tokio::time::sleep(std::time::Duration::from_secs(10000000)).await
        });
    });
}

fn tracing_init_inner(span_events_env_var: &str, log_env_var: &str) {
    use opentelemetry_sdk::runtime;
    use tracing_subscriber::layer::SubscriberExt;

    let event_filter = {
        use fmt::format::FmtSpan;

        // Used exclusively in tests & prefixed with `SI_TEST_`
        #[allow(clippy::disallowed_methods)]
        match ::std::env::var(span_events_env_var) {
            Ok(value) => value
                .to_ascii_lowercase()
                .split(',')
                .map(|filter| match filter.trim() {
                    "new" => FmtSpan::NEW,
                    "enter" => FmtSpan::ENTER,
                    "exit" => FmtSpan::EXIT,
                    "close" => FmtSpan::CLOSE,
                    "active" => FmtSpan::ACTIVE,
                    "full" => FmtSpan::FULL,
                    _ => panic!(
                        "{} must contain filters separated by `,`.\n\t\
                            For example: `active` or `new,close`\n\t
                            Got: {}",
                        span_events_env_var, value,
                    ),
                })
                .fold(FmtSpan::NONE, |acc, filter| filter | acc),
            Err(::std::env::VarError::NotUnicode(_)) => {
                panic!("{} must contain a valid UTF-8 string", span_events_env_var,)
            }
            Err(::std::env::VarError::NotPresent) => FmtSpan::NONE,
        }
    };

    let env_filter = EnvFilter::from_env(log_env_var);
    let format_layer = fmt::layer()
        .with_thread_ids(true)
        .with_span_events(event_filter)
        .with_test_writer()
        .pretty();

    let otel_tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .install_batch(runtime::Tokio)
        .expect("Creating otel_tracer failed");

    let otel_layer = tracing_opentelemetry::layer().with_tracer(otel_tracer);

    let registry = Registry::default();

    let registry = registry
        .with(env_filter)
        .with(format_layer)
        .with(otel_layer);

    subscriber::set_global_default(registry).expect("failed to register global default");
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
    base_change_set_id: ChangeSetId,
) {
    let base_change_set = ChangeSet::find(ctx, base_change_set_id)
        .await
        .expect("could not perform find change set")
        .expect("no change set found");
    let mut change_set = ChangeSet::new(ctx, generate_fake_name(), Some(base_change_set_id))
        .await
        .expect("could not create change set");
    change_set
        .update_pointer(
            ctx,
            base_change_set
                .workspace_snapshot_address
                .expect("no workspace snapshot set on base change set"),
        )
        .await
        .expect("could not update pointer");
    ctx.update_visibility_and_snapshot_to_visibility(change_set.id)
        .await
        .expect("could not update visibility and snapshot");
}

pub fn prepare_decrypted_secret_for_assertions(decrypted_secret: &DecryptedSecret) -> Value {
    // We don't provide a direct getter for the raw decrypted message (higher effort should mean
    // less chance of developer error when handling `DecryptedSecret` types), so we'll serialize to
    // a `Value` to compare messages
    let decrypted_value =
        serde_json::to_value(decrypted_secret).expect("failed to serialize decrypted contents");
    decrypted_value["message"].to_owned()
}
