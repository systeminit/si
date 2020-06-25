use futures;
use lazy_static::lazy_static;
use opentelemetry::{api::Provider, sdk};
use tokio;
use tokio::sync::Mutex;
use tracing;
use tracing_opentelemetry::layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{self, fmt, EnvFilter, Registry};

use std::env;

use si_account::{gen::service::Service, protobuf, protobuf::account_server::Account};
use si_data::Db;
use si_settings::Settings;

use std::process::Command;
use std::sync::Arc;

pub struct TestAccount {
    user: protobuf::User,
    billing_account: protobuf::BillingAccount,
}

lazy_static! {
    pub static ref SETTINGS: Settings = {
        let exporter = opentelemetry_jaeger::Exporter::builder()
            .with_process(opentelemetry_jaeger::Process {
                service_name: "si-test".into(),
                tags: Vec::new(),
            })
            .init()
            .expect("opentelemetry exporter should initialize");
        let provider = sdk::Provider::builder()
            .with_simple_exporter(exporter)
            .with_config(sdk::Config {
                default_sampler: Box::new(sdk::Sampler::Always),
                ..Default::default()
            })
            .build();

        let tracer = provider.get_tracer("si-test");

        let fmt_layer = fmt::Layer::default();
        let opentelemetry_layer = layer().with_tracer(tracer);
        let env_filter_layer = EnvFilter::from_default_env();

        let subscriber = Registry::default()
            .with(env_filter_layer)
            .with(fmt_layer)
            .with(opentelemetry_layer);

        tracing::subscriber::set_global_default(subscriber)
            .expect("tracing global default should be set");

        env::set_var("RUN_ENV", "testing");

        Settings::new().expect("settings should load")
    };
    pub static ref SERVICE: Service = delete_and_create_buckets();
    pub static ref TEST_ACCOUNT: Arc<Mutex<Option<TestAccount>>> = Arc::new(Mutex::new(None));
}

pub fn delete_and_create_buckets() -> Service {
    // OMG, blocking http clients just.. not longer exist, that
    // can also send form encoded data. I'm.. hurt.
    let _delete_result = Command::new("curl")
        .arg("-u")
        .arg("si:bugbear")
        .arg("-X")
        .arg("DELETE")
        .arg("http://localhost:8091/pools/default/buckets/si_integration")
        .output();

    let _create_result = Command::new("curl")
        .arg("-u")
        .arg("si:bugbear")
        .arg("-X")
        .arg("POST")
        .arg("http://127.0.0.1:8091/pools/default/buckets")
        .arg("-d")
        .arg("name=si_integration")
        .arg("-d")
        .arg("ramQuotaMB=512")
        .arg("-d")
        .arg("bucketType=couchbase")
        .arg("-d")
        .arg("authType=sasl")
        .arg("-d")
        .arg("flushEnabled=1")
        .output();

    std::thread::sleep(std::time::Duration::from_secs(10));

    let _create_result = Command::new("curl")
        .arg("-u")
        .arg("si:bugbear")
        .arg("http://127.0.0.1:8093/query/service")
        .arg("-d")
        .arg("statement=create primary index on `si_integration`")
        .output();

    std::thread::sleep(std::time::Duration::from_secs(10));

    let db = Db::new(&SETTINGS).expect("failed to connect to database cluster");
    let service = Service::new(db);
    futures::executor::block_on(async { service.migrate().await.expect("migrations should work") });
    service
}

async fn create_test_account() {
    let mut test_account_option = TEST_ACCOUNT.lock().await;
    if test_account_option.is_some() {
        return;
    }
    let response = SERVICE
        .billing_account_signup(tonic::Request::new(protobuf::BillingAccountSignupRequest {
            billing_account: Some(protobuf::BillingAccountSignupRequestBillingAccount {
                name: Some("slayer".to_string()),
                display_name: Some("Slayer".to_string()),
            }),
            user: Some(protobuf::BillingAccountSignupRequestUser {
                name: Some("Jeff".to_string()),
                display_name: Some("Jeff Hanneman".to_string()),
                email: Some("jeff@slayer.com".to_string()),
                password: Some("south0fheaven".to_string()),
            }),
        }))
        .await
        .expect("failed to signup with the billing account");
    let response_ref = response.get_ref();
    test_account_option.get_or_insert(TestAccount {
        user: response_ref.user.as_ref().unwrap().clone(),
        billing_account: response_ref.billing_account.as_ref().unwrap().clone(),
    });
}

mod login {
    use super::{create_test_account, SERVICE, TEST_ACCOUNT};
    use si_account::{protobuf, protobuf::account_server::Account};

    #[tokio::test]
    async fn bare_request_invalid() {
        create_test_account().await;
        // Bare login requests should be Invalid
        match SERVICE
            .user_login_internal(tonic::Request::new(Default::default()))
            .await
        {
            Ok(re) => assert_eq!(re.get_ref().authenticated, Some(false)),
            Err(e) => panic!("login failed, and it should never: {}", e),
        };
    }

    #[tokio::test]
    async fn valid_login() {
        create_test_account().await;
        let tao = TEST_ACCOUNT.lock().await;
        let ta = tao.as_ref().expect("should have an account");

        match SERVICE
            .user_login_internal(tonic::Request::new(protobuf::UserLoginInternalRequest {
                email: Some(
                    ta.user
                        .email
                        .as_ref()
                        .expect("email should be Some")
                        .to_string(),
                ),
                password: Some("south0fheaven".to_string()),
                billing_account_name: Some(
                    ta.billing_account
                        .name
                        .as_ref()
                        .expect("name should be Some")
                        .to_string(),
                ),
            }))
            .await
        {
            Ok(re) => {
                assert_eq!(
                    re.get_ref().authenticated,
                    Some(true),
                    "Authentication passed"
                );
                assert_eq!(re.get_ref().user_id, ta.user.id, "User ID matches");
                assert_eq!(
                    re.get_ref().billing_account_id,
                    ta.billing_account.id,
                    "Billing account ID matches"
                );
            }
            Err(e) => panic!("login failed, and it should never: {}", e),
        };
    }

    #[tokio::test]
    async fn bad_password() {
        create_test_account().await;
        let tao = TEST_ACCOUNT.lock().await;
        let ta = tao.as_ref().expect("should have an account");

        match SERVICE
            .user_login_internal(tonic::Request::new(protobuf::UserLoginInternalRequest {
                email: Some(
                    ta.user
                        .email
                        .as_ref()
                        .expect("email should be Some")
                        .to_string(),
                ),
                password: Some("g0dhat3susAll".to_string()),
                billing_account_name: Some(
                    ta.billing_account
                        .name
                        .as_ref()
                        .expect("name should be Some")
                        .to_string(),
                ),
            }))
            .await
        {
            Ok(re) => assert_eq!(re.get_ref().authenticated, Some(false)),
            Err(e) => panic!("login failed, and it should never: {}", e),
        };
    }

    #[tokio::test]
    async fn bad_billing_account() {
        create_test_account().await;
        let tao = TEST_ACCOUNT.lock().await;
        let ta = tao.as_ref().expect("should have an account");

        match SERVICE
            .user_login_internal(tonic::Request::new(protobuf::UserLoginInternalRequest {
                email: Some(
                    ta.user
                        .email
                        .as_ref()
                        .expect("name should be Some")
                        .to_string(),
                ),
                password: Some("south0fheaven".to_string()),
                billing_account_name: Some("asdfasf12312312312312".to_string()),
            }))
            .await
        {
            Ok(re) => assert_eq!(re.get_ref().authenticated, Some(false)),
            Err(e) => panic!("login failed, and it should never: {}", e),
        };
    }
}

mod billing_account_signup {
    use super::SERVICE;
    use si_account::{protobuf, protobuf::account_server::Account};

    #[tokio::test]
    async fn bare_request_invalid() {
        // Bare account requests should be Invalid
        match SERVICE
            .billing_account_signup(tonic::Request::new(Default::default()))
            .await
        {
            Ok(_) => panic!("created account with bare request"),
            Err(e @ tonic::Status { .. }) => match e.code() {
                tonic::Code::InvalidArgument => true,
                _ => panic!("create account failed with wrong status code: {}", e),
            },
        };
    }

    #[tokio::test]
    async fn bare_billing_account_invalid() {
        match SERVICE
            .billing_account_signup(tonic::Request::new(protobuf::BillingAccountSignupRequest {
                billing_account: Some(protobuf::BillingAccountSignupRequestBillingAccount {
                    ..Default::default()
                }),
                ..Default::default()
            }))
            .await
        {
            Ok(_) => panic!("created account with bare billing account request"),
            Err(e @ tonic::Status { .. }) => match e.code() {
                tonic::Code::InvalidArgument => true,
                _ => panic!(
                    "create billing account failed with wrong status code: {}",
                    e
                ),
            },
        };
    }

    #[tokio::test]
    async fn partial_billing_account_invalid() {
        match SERVICE
            .billing_account_signup(tonic::Request::new(protobuf::BillingAccountSignupRequest {
                billing_account: Some(protobuf::BillingAccountSignupRequestBillingAccount {
                    name: Some("floopsie".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }))
            .await
        {
            Ok(_) => panic!("created account with bare billing account request"),
            Err(e @ tonic::Status { .. }) => match e.code() {
                tonic::Code::InvalidArgument => true,
                _ => panic!(
                    "create billing account failed with wrong status code: {}",
                    e
                ),
            },
        };
    }

    #[tokio::test]
    async fn bare_user_account_invalid() {
        match SERVICE
            .billing_account_signup(tonic::Request::new(protobuf::BillingAccountSignupRequest {
                billing_account: Some(protobuf::BillingAccountSignupRequestBillingAccount {
                    display_name: Some("System Initiative".to_string()),
                    name: Some("sysinit".to_string()),
                }),
                user: Some(protobuf::BillingAccountSignupRequestUser {
                    ..Default::default()
                }),
            }))
            .await
        {
            Ok(_) => panic!("created account with bare request"),
            Err(e @ tonic::Status { .. }) => match e.code() {
                tonic::Code::InvalidArgument => true,
                _ => panic!("create account failed with wrong status code: {}", e),
            },
        };
    }

    #[tokio::test]
    async fn partial_user_account_invalid() {
        match SERVICE
            .billing_account_signup(tonic::Request::new(protobuf::BillingAccountSignupRequest {
                billing_account: Some(protobuf::BillingAccountSignupRequestBillingAccount {
                    display_name: Some("Slipknot".to_string()),
                    name: Some("slipknot".to_string()),
                }),
                user: Some(protobuf::BillingAccountSignupRequestUser {
                    display_name: Some("Corey Taylor".to_string()),
                    ..Default::default()
                }),
            }))
            .await
        {
            Ok(_) => panic!("created account with bare request"),
            Err(e @ tonic::Status { .. }) => match e.code() {
                tonic::Code::InvalidArgument => true,
                _ => panic!("create account failed with wrong status code: {}", e),
            },
        };
    }

    #[tokio::test]
    async fn signup_succeeds() {
        SERVICE
            .billing_account_signup(tonic::Request::new(protobuf::BillingAccountSignupRequest {
                billing_account: Some(protobuf::BillingAccountSignupRequestBillingAccount {
                    display_name: Some("Slipknot".to_string()),
                    name: Some("slipknot".to_string()),
                }),
                user: Some(protobuf::BillingAccountSignupRequestUser {
                    name: Some("Corey".to_string()),
                    display_name: Some("Corey Taylor".to_string()),
                    email: Some("corey@slipknot.com".to_string()),
                    password: Some("urnotourkind".to_string()),
                }),
            }))
            .await
            .expect("create account failed");
    }
}
