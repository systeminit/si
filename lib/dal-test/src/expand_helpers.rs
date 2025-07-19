//! This module contains helpers for dal test macro expansion.
//!
//! _Caution:_ functions in this module may appear to be unused, but they are likely used during
//! macro expansion.

#![allow(clippy::expect_used, clippy::panic)]

use dal::{
    ChangeSet,
    ChangeSetId,
    DalContext,
    WorkspacePk,
};
use jwt_simple::{
    algorithms::RSAKeyPairLike,
    claims::Claims,
    prelude::Duration,
};
use si_db::Tenancy;
use si_jwt_public_key::SiJwtClaims;
use tracing_subscriber::{
    EnvFilter,
    Registry,
    fmt,
    util::SubscriberInitExt,
};

use crate::{
    WorkspaceSignup,
    helpers::{
        create_user,
        generate_fake_name,
    },
    jwt_private_signing_key,
};

/// Creates a user for each test to run as
pub async fn setup_history_actor_ctx(ctx: &mut DalContext) {
    let user = create_user(ctx).await.expect("unable to create user");
    user.associate_workspace(
        ctx,
        ctx.tenancy()
            .workspace_pk_opt()
            .expect("no workspace pk set on context"),
    )
    .await
    .expect("unable to associate user with workspace");
    ctx.update_history_actor(si_db::HistoryActor::User(user.pk()));
}

/// This function is used during macro expansion for setting up a [`ChangeSet`] in an integration test.
pub async fn create_change_set_and_update_ctx(
    ctx: &mut DalContext,
    base_change_set_id: ChangeSetId,
) {
    let base_change_set = ChangeSet::get_by_id(ctx, base_change_set_id)
        .await
        .expect("could not find change set");
    let workspace = ctx
        .get_workspace_or_builtin()
        .await
        .expect("could not get workspace");
    let workspace_snapshot_address = base_change_set.workspace_snapshot_address;
    let change_set = ChangeSet::new(
        ctx,
        generate_fake_name().expect("could not generate fake name"),
        Some(base_change_set_id),
        workspace_snapshot_address,
        workspace.snapshot_kind(),
    )
    .await
    .expect("could not create change set");
    ctx.update_visibility_and_snapshot_to_visibility(change_set.id)
        .await
        .expect("could not update visibility and snapshot");
}

/// This function is used during macro expansion for setting up tracing in an integration test.
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
    use tracing_subscriber::layer::SubscriberExt;

    telemetry::opentelemetry::global::set_text_map_propagator(
        opentelemetry_sdk::propagation::TraceContextPropagator::new(),
    );

    let span_events_fmt = {
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
                        "{span_events_env_var} must contain filters separated by `,`.\n\t\
                            For example: `active` or `new,close`\n\t
                            Got: {value}",
                    ),
                })
                .fold(FmtSpan::NONE, |acc, filter| filter | acc),
            Err(::std::env::VarError::NotUnicode(_)) => {
                panic!("{span_events_env_var} must contain a valid UTF-8 string",)
            }
            Err(::std::env::VarError::NotPresent) => FmtSpan::NONE,
        }
    };

    let env_filter = EnvFilter::from_env(log_env_var);

    let format_layer = fmt::layer()
        .with_thread_ids(true)
        .with_span_events(span_events_fmt)
        .with_test_writer()
        .pretty();

    // let otel_layer = {
    //     use std::time::Duration;
    //
    //     let resource = opentelemetry_sdk::Resource::from_detectors(
    //         Duration::from_secs(3),
    //         vec![
    //             Box::new(opentelemetry_sdk::resource::EnvResourceDetector::new()),
    //             Box::new(opentelemetry_sdk::resource::OsResourceDetector),
    //             Box::new(opentelemetry_sdk::resource::ProcessResourceDetector),
    //         ],
    //     )
    //     .merge(&opentelemetry_sdk::Resource::new(vec![
    //         // TODO(fnichol): make name configurable
    //         telemetry::opentelemetry::KeyValue::new("service.name", "test"),
    //         telemetry::opentelemetry::KeyValue::new("service.namespace", "si"),
    //     ]));
    //
    //     let otel_tracer = opentelemetry_otlp::new_pipeline()
    //         .tracing()
    //         .with_exporter(opentelemetry_otlp::new_exporter().tonic())
    //         .with_trace_config(opentelemetry_sdk::trace::config().with_resource(resource))
    //         .install_batch(opentelemetry_sdk::runtime::Tokio)
    //         .expect("Creating otel_tracer failed");
    //
    //     tracing_opentelemetry::layer().with_tracer(otel_tracer)
    // };

    let registry = Registry::default();
    let registry = registry.with(env_filter);
    let registry = registry.with(format_layer);
    // let registry = registry.with(otel_layer);

    registry
        .try_init()
        .expect("failed to initialize subscriber");
}

/// This function is used during macro expansion for setting up the workspace for integration tests.
pub async fn workspace_signup(ctx: &mut DalContext) -> crate::Result<(WorkspaceSignup, String)> {
    use color_eyre::eyre::WrapErr;
    // Tenancy must be set here or the update_visibility call will fail, since it now needs to check
    // the workspace snapshot_kind
    ctx.update_tenancy(Tenancy::new(WorkspacePk::NONE));

    let mut ctx = ctx.clone_with_head().await?;

    let workspace_name = generate_fake_name().expect("could not generate fake name");
    let user_name = format!("frank {workspace_name}");
    let user_email = format!("{workspace_name}@example.com");
    let token = "workspace_name".to_string();

    let nw = WorkspaceSignup::new(&mut ctx, &workspace_name, &user_name, &user_email, &token)
        .await
        .wrap_err("cannot signup a new workspace")?;
    let auth_token = create_auth_token(SiJwtClaims::for_web(nw.user.pk(), *nw.workspace.pk()))
        .await
        .expect("could not create auth token");
    Ok((nw, auth_token))
}

async fn create_auth_token(claim: SiJwtClaims) -> crate::Result<String> {
    let key_pair = jwt_private_signing_key().await?;
    let claim = Claims::with_custom_claims(claim.clone(), Duration::from_days(1))
        .with_audience("https://app.systeminit.com")
        .with_issuer("https://app.systeminit.com")
        .with_subject(claim.user_id());

    Ok(key_pair.sign(claim).expect("could not sign"))
}
