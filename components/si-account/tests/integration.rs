use lazy_static::lazy_static;
use tokio;
use tokio::sync::Mutex;
use tracing;
use tracing_subscriber::{self, EnvFilter, FmtSubscriber};

use std::env;

use si_account::{protobuf, protobuf::account_server::Account, service::Service};
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
        let subscriber = FmtSubscriber::builder()
            .with_env_filter(EnvFilter::from_default_env())
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting tracing default failed");
        env::set_var("RUN_ENV", "testing");
        Settings::new().expect("Failed to load settings")
    };
    pub static ref SERVICE: Service = { delete_and_create_buckets() };
    pub static ref TEST_ACCOUNT: Arc<Mutex<Option<TestAccount>>> = { Arc::new(Mutex::new(None)) };
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

    let create_result = Command::new("curl")
        .arg("-u")
        .arg("si:bugbear")
        .arg("http://127.0.0.1:8093/query/service")
        .arg("-d")
        .arg("statement=create primary index on `si_integration`")
        .output();

    std::thread::sleep(std::time::Duration::from_secs(10));

    let db = Db::new(&SETTINGS).expect("failed to connect to database cluster");
    Service::new(db)
}

async fn create_test_account() {
    let mut test_account_option = TEST_ACCOUNT.lock().await;
    if test_account_option.is_some() {
        return;
    }
    let response = SERVICE
        .create_account(tonic::Request::new(protobuf::CreateAccountRequest {
            billing_account: Some(protobuf::CreateBillingAccountRequest {
                display_name: "Slayer".to_string(),
                short_name: "slayer".to_string(),
            }),
            user: Some(protobuf::CreateUserRequest {
                display_name: "Jeff Hanneman".to_string(),
                given_name: "Jeff".to_string(),
                family_name: "Hanneman".to_string(),
                email: "jeff@slayer.com".to_string(),
                password: "south0fheaven".to_string(),
                ..Default::default()
            }),
        }))
        .await
        .expect("Failed to create the account");
    let response_ref = response.get_ref();
    test_account_option.get_or_insert(TestAccount {
        user: response_ref.user.as_ref().unwrap().clone(),
        billing_account: response_ref.billing_account.as_ref().unwrap().clone(),
    });
}

mod authorize {
    use super::{create_test_account, SERVICE, TEST_ACCOUNT};
    use si_account::authorize::authorize;

    #[tokio::test]
    async fn authorize_works_for_new_user() {
        create_test_account().await;
        let tao = TEST_ACCOUNT.lock().await;
        let ta = tao.as_ref().expect("should have an account");
        let result = authorize(
            &SERVICE.db(),
            &ta.user.id,
            &ta.billing_account.id,
            "any",
            &ta.billing_account,
        )
        .await
        .expect("authorize should give a result");
        assert_eq!(result, (), "initial user should always authorize true");
    }
}

mod login {
    use super::{create_test_account, SERVICE, TEST_ACCOUNT};
    use si_account::{protobuf, protobuf::account_server::Account};

    #[tokio::test]
    async fn bare_request_invalid() {
        create_test_account().await;
        // Bare login requests should be Invalid
        match SERVICE
            .login(tonic::Request::new(protobuf::LoginRequest {
                ..Default::default()
            }))
            .await
        {
            Ok(re) => assert_eq!(re.get_ref().authenticated, false),
            Err(e) => panic!("login failed, and it should never: {}", e),
        };
    }

    #[tokio::test]
    async fn valid_login() {
        create_test_account().await;
        let tao = TEST_ACCOUNT.lock().await;
        let ta = tao.as_ref().expect("should have an account");

        match SERVICE
            .login(tonic::Request::new(protobuf::LoginRequest {
                email: ta.user.email.to_string(),
                password: "south0fheaven".to_string(),
                billing_account_short_name: ta.billing_account.short_name.to_string(),
            }))
            .await
        {
            Ok(re) => {
                assert_eq!(re.get_ref().authenticated, true, "Authentication passed");
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
            .login(tonic::Request::new(protobuf::LoginRequest {
                email: ta.user.email.to_string(),
                password: "g0dhat3susAll".to_string(),
                billing_account_short_name: ta.billing_account.short_name.to_string(),
            }))
            .await
        {
            Ok(re) => assert_eq!(re.get_ref().authenticated, false),
            Err(e) => panic!("login failed, and it should never: {}", e),
        };
    }

    #[tokio::test]
    async fn bad_billing_account() {
        create_test_account().await;
        let tao = TEST_ACCOUNT.lock().await;
        let ta = tao.as_ref().expect("should have an account");

        match SERVICE
            .login(tonic::Request::new(protobuf::LoginRequest {
                email: ta.user.email.to_string(),
                password: "south0fheaven".to_string(),
                billing_account_short_name: "asdfasf12312312312312".to_string(),
            }))
            .await
        {
            Ok(re) => assert_eq!(re.get_ref().authenticated, false),
            Err(e) => panic!("login failed, and it should never: {}", e),
        };
    }
}

mod create_account {
    use super::SERVICE;
    use si_account::{protobuf, protobuf::account_server::Account};

    #[tokio::test]
    async fn bare_request_invalid() {
        // Bare account requests should be Invalid
        match SERVICE
            .create_account(tonic::Request::new(protobuf::CreateAccountRequest {
                ..Default::default()
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
    async fn bare_billing_account_invalid() {
        match SERVICE
            .create_account(tonic::Request::new(protobuf::CreateAccountRequest {
                billing_account: Some(protobuf::CreateBillingAccountRequest {
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
            .create_account(tonic::Request::new(protobuf::CreateAccountRequest {
                billing_account: Some(protobuf::CreateBillingAccountRequest {
                    short_name: "floopsie".to_string(),
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
            .create_account(tonic::Request::new(protobuf::CreateAccountRequest {
                billing_account: Some(protobuf::CreateBillingAccountRequest {
                    display_name: "System Initiative".to_string(),
                    short_name: "sysinit".to_string(),
                }),
                user: Some(protobuf::CreateUserRequest {
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
            .create_account(tonic::Request::new(protobuf::CreateAccountRequest {
                billing_account: Some(protobuf::CreateBillingAccountRequest {
                    display_name: "Slipknot".to_string(),
                    short_name: "slipknot".to_string(),
                }),
                user: Some(protobuf::CreateUserRequest {
                    display_name: "Corey Taylor".to_string(),
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
    async fn create_succeeds() {
        SERVICE
            .create_account(tonic::Request::new(protobuf::CreateAccountRequest {
                billing_account: Some(protobuf::CreateBillingAccountRequest {
                    display_name: "Slipknot".to_string(),
                    short_name: "slipknot".to_string(),
                }),
                user: Some(protobuf::CreateUserRequest {
                    display_name: "Corey Taylor".to_string(),
                    given_name: "Corey".to_string(),
                    family_name: "Taylor".to_string(),
                    email: "corey@slipknot.com".to_string(),
                    password: "urnotourkind".to_string(),
                    ..Default::default()
                }),
            }))
            .await
            .expect("create account failed");
    }
}
