//! Expansion implementation of the `dal_test` attribute macro.
//!
//! This implementation is a combination of a configurable threaded Tokio runtime (formerly
//! provided via the `tokio::test` macro from the `tokio` crate), support for optional
//! tracing/logging support (formerly provided via the `test` mecro from the `test-env-log` crate),
//! and an "extractor"-style dependency setup a little like axum's extractors.
//!
//! # Reference Implementations and Credits
//!
//! * [`tokio::test` macro](https://github.com/tokio-rs/tokio/blob/121769c762ad6b1686ecd0e8618005aab8b7e980/tokio-macros/src/entry.rs)
//! * [`test_env_log::test` macro](https://github.com/d-e-s-o/test-log/blob/544dbac50321aaf580959ad7a7997358517db198/src/lib.rs)

use std::sync::Arc;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    parse_quote, punctuated::Punctuated, token::Comma, AttributeArgs, Expr, FnArg, ItemFn, Path,
    ReturnType, Type,
};

const LOG_ENV_VAR: &str = "SI_TEST_LOG";
const SPAN_EVENTS_ENV_VAR: &str = "SI_TEST_LOG_SPAN_EVENTS";

const RT_DEFAULT_WORKER_THREADS: usize = 2;
const RT_DEFAULT_THREAD_STACK_SIZE: usize = 2 * 1024 * 1024 * 3;

pub(crate) fn expand(item: ItemFn, _args: AttributeArgs) -> TokenStream {
    if item.sig.asyncness.is_none() {
        panic!("test function must be async--blocking tests not supported");
    }

    let attrs = &item.attrs;
    let body = &item.block;
    let test_name = &item.sig.ident;
    let params = &item.sig.inputs;
    // Note that Rust doesn't allow a test function with `#[should_panic]` that has a non-unit
    // return value. Huh
    let output = match &item.sig.output {
        ReturnType::Default => quote! {},
        ReturnType::Type(_, typeness) => quote! {-> #typeness},
    };
    let test_attr = quote! {#[::core::prelude::v1::test]};

    let worker_threads = RT_DEFAULT_WORKER_THREADS;
    let thread_stack_size = RT_DEFAULT_THREAD_STACK_SIZE;

    let fn_setup = fn_setup(item.sig.inputs.iter());
    let fn_setups = fn_setup.code;
    let fn_args = fn_setup.fn_args;
    let tracing_init = expand_tracing_init();
    let rt = expand_runtime(worker_threads, thread_stack_size);

    quote! {
        #test_attr
        #(#attrs)*
        fn #test_name() #output {
            async fn inner(#params) #output #body

            #[inline]
            async fn imp() #output {
                #fn_setups
                inner(#fn_args).await
            }

            #tracing_init

            let thread_builder = ::std::thread::Builder::new().stack_size(#thread_stack_size);
            let thread_handler = thread_builder.spawn(|| {
                #[allow(clippy::expect_used)]
                #rt.block_on(imp())
            }).unwrap();
            thread_handler.join().unwrap();
        }
    }
}

fn fn_setup<'a>(params: impl Iterator<Item = &'a FnArg>) -> FnSetup {
    let mut expander = FnSetupExpander::new();

    for param in params {
        match param {
            FnArg::Typed(pat_type) => match &*pat_type.ty {
                Type::Path(type_path) => {
                    let path = path_as_string(&type_path.path);
                    if let Some(ty_str) = path.split("::").last() {
                        // Each string match corresponds to an imported type that corresponds to an
                        // **owned** variable. For example:
                        //
                        // ```ignore
                        // #[test]
                        // async fn does_things(bid: BillingAccountId) {
                        //      // ...
                        // }
                        // ```
                        //
                        // Note that several types such as `DalContextHead` may have interior
                        // references and/or mutability, however the surrounding type is passed as
                        // an owned type.
                        match ty_str {
                            "BillingAccountId" => {
                                let var = expander.setup_billing_account_id();
                                let var = var.as_ref();
                                expander.push_arg(parse_quote! {#var});
                            }
                            "BillingAccountSignup" => {
                                let var = expander.setup_billing_account_signup();
                                let var = var.0.as_ref();
                                expander.push_arg(parse_quote! {#var});
                            }
                            "DalContext" => {
                                let var = expander.setup_dal_context_default();
                                let var = var.as_ref();
                                expander.push_arg(parse_quote! {#var});
                            }
                            "DalContextBuilder" => {
                                let var = expander.setup_dal_context_builder();
                                let var = var.as_ref();
                                expander.push_arg(parse_quote! {#var});
                            }
                            "DalContextHead" => {
                                let var = expander.setup_dal_context_head();
                                let var = var.as_ref();
                                expander.push_arg(parse_quote! {#var});
                            }
                            "DalContextHeadRef" => {
                                let var = expander.setup_dal_context_head_ref();
                                let var = var.as_ref();
                                expander.push_arg(parse_quote! {#var});
                            }
                            "DalContextHeadMutRef" => {
                                let var = expander.setup_dal_context_head_mut_ref();
                                let var = var.as_ref();
                                expander.push_arg(parse_quote! {#var});
                            }
                            "DalContextUniversalHead" => {
                                let var = expander.setup_dal_context_universal_head();
                                let var = var.as_ref();
                                expander.push_arg(parse_quote! {#var});
                            }
                            "DalContextUniversalHeadRef" => {
                                let var = expander.setup_dal_context_universal_head_ref();
                                let var = var.as_ref();
                                expander.push_arg(parse_quote! {#var});
                            }
                            "DalContextUniversalHeadMutRef" => {
                                let var = expander.setup_dal_context_universal_head_mut_ref();
                                let var = var.as_ref();
                                expander.push_arg(parse_quote! {#var});
                            }
                            "OrganizationId" => {
                                let var = expander.setup_organization_id();
                                let var = var.as_ref();
                                expander.push_arg(parse_quote! {#var});
                            }
                            "ServicesContext" => {
                                let var = expander.setup_services_context();
                                let var = var.as_ref();
                                expander.push_arg(parse_quote! {#var});
                            }
                            "ShutdownHandle" => {
                                let var = expander.setup_veritech_shutdown_handle();
                                let var = var.as_ref();
                                expander.push_arg(parse_quote! {#var});
                            }
                            "TransactionsStarter" => {
                                let var = expander.setup_owned_transactions_starter();
                                let var = var.as_ref();
                                expander.push_arg(parse_quote! {#var});
                            }
                            "WorkspaceId" => {
                                let var = expander.setup_workspace_id();
                                let var = var.as_ref();
                                expander.push_arg(parse_quote! {#var});
                            }
                            "ApplicationId" => {
                                let var = expander.setup_application_id();
                                let var = var.as_ref();
                                expander.push_arg(parse_quote! {#var});
                            }
                            _ => panic!("unexpected argument type: {:?}", type_path),
                        };
                    }
                }
                Type::Reference(type_ref) => match &*type_ref.elem {
                    Type::Path(type_path) => {
                        let path = path_as_string(&type_path.path);
                        if let Some(ty_str) = path.split("::").last() {
                            // Each string match corresponds to an imported type that corresponds
                            // to an **borrowed**/**referenced** variable. For example:
                            //
                            // ```ignore
                            // #[test]
                            // async fn does_things(
                            //      ctx: &mut DalContext<'_, '_>,
                            //      nba: &BillingAccountSignup
                            //  ) {
                            //      // ...
                            // }
                            // ```
                            //
                            // In the above example, both would be matched types in this section,
                            // even though `ctx` is a mutable reference and `nba` is an immutable
                            // reference.
                            match ty_str {
                                "BillingAccountSignup" => {
                                    let var = expander.setup_billing_account_signup();
                                    let var = var.0.as_ref();
                                    expander.push_arg(parse_quote! {&#var});
                                }
                                "DalContext" => {
                                    if type_ref.mutability.is_some() {
                                        let var = expander.setup_dal_context_default_mut();
                                        let var = var.as_ref();
                                        expander.push_arg(parse_quote! {&mut #var});
                                    } else {
                                        let var = expander.setup_dal_context_default();
                                        let var = var.as_ref();
                                        expander.push_arg(parse_quote! {&#var});
                                    }
                                }
                                "DalContextBuilder" => {
                                    let var = expander.setup_dal_context_builder();
                                    let var = var.as_ref();
                                    expander.push_arg(parse_quote! {&#var});
                                }
                                "JwtSecretKey" => {
                                    let var = expander.setup_jwt_secret_key();
                                    let var = var.as_ref();
                                    expander.push_arg(parse_quote! {#var});
                                }
                                "ServicesContext" => {
                                    let var = expander.setup_services_context();
                                    let var = var.as_ref();
                                    expander.push_arg(parse_quote! {&#var});
                                }
                                _ => panic!("unexpected argument reference type: {:?}", type_ref),
                            }
                        }
                    }
                    unsupported => {
                        panic!("argument reference type not supported: {:?}", unsupported)
                    }
                },
                unsupported => panic!("argument type not supported: {:?}", unsupported),
            },
            FnArg::Receiver(_) => {
                panic!("argument does not support receiver/method style (i.e. using `self`)")
            }
        }
    }

    if expander.has_args() {
        // TODO(fnichol): we can use a macro attribute to opt-out and not run a veritech server in the
        // future, but for now (as before), every test starts with its own veritech server with a
        // randomized subject prefix
        expander.setup_start_veritech_server();
    }

    expander.finish()
}

struct FnSetup {
    code: TokenStream,
    fn_args: Punctuated<Expr, Comma>,
}

struct FnSetupExpander {
    code: TokenStream,
    args: Punctuated<Expr, Comma>,

    test_context: Option<Arc<Ident>>,
    jwt_secret_key: Option<Arc<Ident>>,
    nats_subject_prefix: Option<Arc<Ident>>,
    veritech_server: Option<Arc<Ident>>,
    veritech_shutdown_handle: Option<Arc<Ident>>,
    start_veritech_server: Option<()>,
    services_context: Option<Arc<Ident>>,
    dal_context_builder: Option<Arc<Ident>>,
    transaction_starter: Option<Arc<Ident>>,
    owned_transaction_starter: Option<Arc<Ident>>,
    transactions: Option<Arc<Ident>>,
    application_id: Option<Arc<Ident>>,
    billing_account_signup: Option<(Arc<Ident>, Arc<Ident>)>,
    billing_account_id: Option<Arc<Ident>>,
    organization_id: Option<Arc<Ident>>,
    workspace_id: Option<Arc<Ident>>,
    dal_context_default: Option<Arc<Ident>>,
    dal_context_default_mut: Option<Arc<Ident>>,
    dal_context_head: Option<Arc<Ident>>,
    dal_context_head_ref: Option<Arc<Ident>>,
    dal_context_head_mut_ref: Option<Arc<Ident>>,
    dal_context_universal_head: Option<Arc<Ident>>,
    dal_context_universal_head_ref: Option<Arc<Ident>>,
    dal_context_universal_head_mut_ref: Option<Arc<Ident>>,
}

impl FnSetupExpander {
    fn new() -> Self {
        Self {
            code: TokenStream::new(),
            args: Punctuated::new(),
            test_context: None,
            jwt_secret_key: None,
            nats_subject_prefix: None,
            veritech_server: None,
            veritech_shutdown_handle: None,
            start_veritech_server: None,
            services_context: None,
            dal_context_builder: None,
            transaction_starter: None,
            owned_transaction_starter: None,
            transactions: None,
            application_id: None,
            billing_account_signup: None,
            billing_account_id: None,
            organization_id: None,
            workspace_id: None,
            dal_context_default: None,
            dal_context_default_mut: None,
            dal_context_head: None,
            dal_context_head_ref: None,
            dal_context_head_mut_ref: None,
            dal_context_universal_head: None,
            dal_context_universal_head_ref: None,
            dal_context_universal_head_mut_ref: None,
        }
    }

    fn push_arg(&mut self, arg: Expr) {
        self.args.push(arg);
    }

    fn setup_test_context(&mut self) -> Arc<Ident> {
        if let Some(ref ident) = self.test_context {
            return ident.clone();
        }

        let var = Ident::new("test_context", Span::call_site());
        self.code.extend(quote! {
            let test_context = ::dal::test::TestContext::global().await;
        });
        self.test_context = Some(Arc::new(var));

        self.test_context.as_ref().unwrap().clone()
    }

    fn setup_jwt_secret_key(&mut self) -> Arc<Ident> {
        if let Some(ref ident) = self.jwt_secret_key {
            return ident.clone();
        }

        let test_context = self.setup_test_context();
        let test_context = test_context.as_ref();

        let var = Ident::new("jwt_secret_key", Span::call_site());
        self.code.extend(quote! {
            let #var = #test_context.jwt_secret_key();
        });
        self.jwt_secret_key = Some(Arc::new(var));

        self.jwt_secret_key.as_ref().unwrap().clone()
    }

    fn setup_nats_subject_prefix(&mut self) -> Arc<Ident> {
        if let Some(ref ident) = self.nats_subject_prefix {
            return ident.clone();
        }

        let var = Ident::new("nats_subject_prefix", Span::call_site());
        self.code.extend(quote! {
            let #var = ::dal::test::nats_subject_prefix();
        });
        self.nats_subject_prefix = Some(Arc::new(var));

        self.nats_subject_prefix.as_ref().unwrap().clone()
    }

    fn setup_veritech_server(&mut self) -> Arc<Ident> {
        if let Some(ref ident) = self.veritech_server {
            return ident.clone();
        }

        let test_context = self.setup_test_context();
        let test_context = test_context.as_ref();
        let nats_subject_prefix = self.setup_nats_subject_prefix();
        let nats_subject_prefix = nats_subject_prefix.as_ref();

        let var = Ident::new("veritech_server", Span::call_site());
        self.code.extend(quote! {
            let #var = ::dal::test::veritech_server_for_uds_cyclone(
                #test_context.nats_config().clone(),
                #nats_subject_prefix.clone(),
            ).await;
        });
        self.veritech_server = Some(Arc::new(var));

        self.veritech_server.as_ref().unwrap().clone()
    }

    fn setup_veritech_shutdown_handle(&mut self) -> Arc<Ident> {
        if let Some(ref ident) = self.veritech_shutdown_handle {
            return ident.clone();
        }

        let veritech_server = self.setup_veritech_server();
        let veritech_server = veritech_server.as_ref();

        let var = Ident::new("veritech_shutdown_handle", Span::call_site());
        self.code.extend(quote! {
            let #var = #veritech_server.shutdown_handle();
        });
        self.veritech_shutdown_handle = Some(Arc::new(var));

        self.veritech_shutdown_handle.as_ref().unwrap().clone()
    }

    fn setup_start_veritech_server(&mut self) {
        if self.start_veritech_server.is_some() {
            return;
        }

        let veritech_server = self.setup_veritech_server();
        let veritech_server = veritech_server.as_ref();

        self.code.extend(quote! {
            ::tokio::spawn(#veritech_server.run());
        });
        self.start_veritech_server = Some(());
    }

    fn setup_services_context(&mut self) -> Arc<Ident> {
        if let Some(ref ident) = self.services_context {
            return ident.clone();
        }

        let test_context = self.setup_test_context();
        let test_context = test_context.as_ref();
        let nats_subject_prefix = self.setup_nats_subject_prefix();
        let nats_subject_prefix = nats_subject_prefix.as_ref();

        let var = Ident::new("services_context", Span::call_site());
        self.code.extend(quote! {
            let #var = #test_context.create_services_context(&#nats_subject_prefix).await;
        });
        self.services_context = Some(Arc::new(var));

        self.services_context.as_ref().unwrap().clone()
    }

    fn setup_dal_context_builder(&mut self) -> Arc<Ident> {
        if let Some(ref ident) = self.dal_context_builder {
            return ident.clone();
        }

        let services_context = self.setup_services_context();
        let services_context = services_context.as_ref();

        let var = Ident::new("dal_context_builder", Span::call_site());
        self.code.extend(quote! {
            let #var = #services_context.into_builder();
        });
        self.dal_context_builder = Some(Arc::new(var));

        self.dal_context_builder.as_ref().unwrap().clone()
    }

    fn setup_transactions_starter(&mut self) -> Arc<Ident> {
        if let Some(ref ident) = self.transaction_starter {
            return ident.clone();
        }

        let dal_context_builder = self.setup_dal_context_builder();
        let dal_context_builder = dal_context_builder.as_ref();

        let var = Ident::new("transactions_starter", Span::call_site());
        self.code.extend(quote! {
            let mut #var = #dal_context_builder
                .transactions_starter()
                .await
                .expect("failed to build transactions starter");
        });
        self.transaction_starter = Some(Arc::new(var));

        self.transaction_starter.as_ref().unwrap().clone()
    }

    fn setup_owned_transactions_starter(&mut self) -> Arc<Ident> {
        if let Some(ref ident) = self.owned_transaction_starter {
            return ident.clone();
        }

        let dal_context_builder = self.setup_dal_context_builder();
        let dal_context_builder = dal_context_builder.as_ref();

        let var = Ident::new("owned_transactions_starter", Span::call_site());
        self.code.extend(quote! {
            let #var = #dal_context_builder
                .transactions_starter()
                .await
                .expect("failed to build transactions starter");
        });
        self.owned_transaction_starter = Some(Arc::new(var));

        self.owned_transaction_starter.as_ref().unwrap().clone()
    }

    fn setup_transactions(&mut self) -> Arc<Ident> {
        if let Some(ref ident) = self.transactions {
            return ident.clone();
        }

        let transactions_starter = self.setup_transactions_starter();
        let transactions_starter = transactions_starter.as_ref();

        let var = Ident::new("transactions", Span::call_site());
        self.code.extend(quote! {
            let #var = #transactions_starter
                .start()
                .await
                .expect("failed to start transactions");
        });
        self.transactions = Some(Arc::new(var));

        self.transactions.as_ref().unwrap().clone()
    }

    fn setup_application_id(&mut self) -> Arc<Ident> {
        if let Some(ident) = &self.application_id {
            return ident.clone();
        }

        let dal_context_builder = self.setup_dal_context_builder();
        let dal_context_builder = dal_context_builder.as_ref();
        let transactions = self.setup_transactions();
        let transactions = transactions.as_ref();

        let bas = self.setup_billing_account_signup();
        let nba = bas.0.as_ref();

        let var = Ident::new("application_id", Span::call_site());
        self.code.extend(quote! {
            let #var = ::dal::test::helpers::create_application(
                &#dal_context_builder,
                &#transactions,
                &#nba
            ).await;
            let #var = {
                use dal::StandardModel;
                *#var.id()
            };
        });
        self.application_id = Some(Arc::new(var));

        self.application_id.as_ref().unwrap().clone()
    }

    fn setup_billing_account_signup(&mut self) -> (Arc<Ident>, Arc<Ident>) {
        if let Some(ref idents) = self.billing_account_signup {
            return idents.clone();
        }

        let test_context = self.setup_test_context();
        let test_context = test_context.as_ref();
        let dal_context_builder = self.setup_dal_context_builder();
        let dal_context_builder = dal_context_builder.as_ref();
        let transactions = self.setup_transactions();
        let transactions = transactions.as_ref();

        let var_nba = Ident::new("nba", Span::call_site());
        let var_auth_token = Ident::new("auth_token", Span::call_site());
        self.code.extend(quote! {
            let (#var_nba, #var_auth_token) = ::dal::test::helpers::billing_account_signup(
                &#dal_context_builder,
                &#transactions,
                #test_context.jwt_secret_key(),
            ).await;
        });
        self.billing_account_signup = Some((Arc::new(var_nba), Arc::new(var_auth_token)));

        self.billing_account_signup.as_ref().unwrap().clone()
    }

    fn setup_billing_account_id(&mut self) -> Arc<Ident> {
        if let Some(ref idents) = self.billing_account_id {
            return idents.clone();
        }

        let bas = self.setup_billing_account_signup();
        let nba = bas.0.as_ref();

        let var = Ident::new("nba_billing_account_id", Span::call_site());
        self.code.extend(quote! {
            let #var = *#nba.billing_account.id();
        });
        self.billing_account_id = Some(Arc::new(var));

        self.billing_account_id.as_ref().unwrap().clone()
    }

    fn setup_organization_id(&mut self) -> Arc<Ident> {
        if let Some(ref idents) = self.organization_id {
            return idents.clone();
        }

        let bas = self.setup_billing_account_signup();
        let nba = bas.0.as_ref();

        let var = Ident::new("nba_organization_id", Span::call_site());
        self.code.extend(quote! {
            let #var = *#nba.organization.id();
        });
        self.organization_id = Some(Arc::new(var));

        self.organization_id.as_ref().unwrap().clone()
    }

    fn setup_workspace_id(&mut self) -> Arc<Ident> {
        if let Some(ref idents) = self.workspace_id {
            return idents.clone();
        }

        let bas = self.setup_billing_account_signup();
        let nba = bas.0.as_ref();

        let var = Ident::new("nba_workspace_id", Span::call_site());
        self.code.extend(quote! {
            let #var = *#nba.workspace.id();
        });
        self.workspace_id = Some(Arc::new(var));

        self.workspace_id.as_ref().unwrap().clone()
    }

    fn setup_dal_context_default(&mut self) -> Arc<Ident> {
        if let Some(ref ident) = self.dal_context_default {
            return ident.clone();
        }

        let dal_context_builder = self.setup_dal_context_builder();
        let dal_context_builder = dal_context_builder.as_ref();
        let transactions = self.setup_transactions();
        let transactions = transactions.as_ref();
        let bas = self.setup_billing_account_signup();
        let nba = bas.0.as_ref();

        let var = Ident::new("default_dal_context", Span::call_site());
        let application_id = self.setup_application_id();
        let application_id = application_id.as_ref();
        self.code.extend(quote! {
            let #var = ::dal::test::helpers::create_ctx_for_new_change_set_and_edit_session(
                &#dal_context_builder,
                &#transactions,
                &#nba,
                #application_id
            ).await;
        });
        self.dal_context_default = Some(Arc::new(var));

        self.dal_context_default.as_ref().unwrap().clone()
    }

    fn setup_dal_context_default_mut(&mut self) -> Arc<Ident> {
        if let Some(ref ident) = self.dal_context_default_mut {
            return ident.clone();
        }

        let dal_context_builder = self.setup_dal_context_builder();
        let dal_context_builder = dal_context_builder.as_ref();
        let transactions = self.setup_transactions();
        let transactions = transactions.as_ref();
        let bas = self.setup_billing_account_signup();
        let nba = bas.0.as_ref();

        let var = Ident::new("dal_context_default_mut", Span::call_site());
        let application_id = self.setup_application_id();
        let application_id = application_id.as_ref();
        self.code.extend(quote! {
            let mut #var = ::dal::test::helpers::create_ctx_for_new_change_set_and_edit_session(
                &#dal_context_builder,
                &#transactions,
                &#nba,
                #application_id,
            ).await;
        });
        self.dal_context_default_mut = Some(Arc::new(var));

        self.dal_context_default_mut.as_ref().unwrap().clone()
    }

    fn setup_dal_context_universal_head(&mut self) -> Arc<Ident> {
        if let Some(ref ident) = self.dal_context_universal_head {
            return ident.clone();
        }

        let dal_context_builder = self.setup_dal_context_builder();
        let dal_context_builder = dal_context_builder.as_ref();
        let transactions = self.setup_transactions();
        let transactions = transactions.as_ref();

        let var = Ident::new("dal_context_universal_head", Span::call_site());
        self.code.extend(quote! {
            let #var = ::dal::test::DalContextUniversalHead(
                    #dal_context_builder.build(
                    ::dal::RequestContext::new_universal_head(::dal::HistoryActor::SystemInit),
                    &#transactions
                )
            );
        });
        self.dal_context_universal_head = Some(Arc::new(var));

        self.dal_context_universal_head.as_ref().unwrap().clone()
    }

    fn setup_dal_context_universal_head_ref(&mut self) -> Arc<Ident> {
        if let Some(ref ident) = self.dal_context_universal_head_ref {
            return ident.clone();
        }

        let dal_context_builder = self.setup_dal_context_builder();
        let dal_context_builder = dal_context_builder.as_ref();
        let transactions = self.setup_transactions();
        let transactions = transactions.as_ref();

        let var = Ident::new("dal_context_universal_head_ref", Span::call_site());
        self.code.extend(quote! {
            let _dcuhr = #dal_context_builder.build(
                ::dal::RequestContext::new_universal_head(::dal::HistoryActor::SystemInit),
                &#transactions
            );
            let #var = ::dal::test::DalContextUniversalHeadRef(&_dcuhr);
        });
        self.dal_context_universal_head_ref = Some(Arc::new(var));

        self.dal_context_universal_head_ref
            .as_ref()
            .unwrap()
            .clone()
    }

    fn setup_dal_context_universal_head_mut_ref(&mut self) -> Arc<Ident> {
        if let Some(ref ident) = self.dal_context_universal_head_mut_ref {
            return ident.clone();
        }

        let dal_context_builder = self.setup_dal_context_builder();
        let dal_context_builder = dal_context_builder.as_ref();
        let transactions = self.setup_transactions();
        let transactions = transactions.as_ref();

        let var = Ident::new("dal_context_universal_head_mut_ref", Span::call_site());
        self.code.extend(quote! {
            let mut _dcuhmr = #dal_context_builder.build(
                ::dal::RequestContext::new_universal_head(::dal::HistoryActor::SystemInit),
                &#transactions
            );
            let #var = ::dal::test::DalContextUniversalHeadMutRef(&mut _dcuhmr);
        });
        self.dal_context_universal_head_mut_ref = Some(Arc::new(var));

        self.dal_context_universal_head_mut_ref
            .as_ref()
            .unwrap()
            .clone()
    }

    fn setup_dal_context_head(&mut self) -> Arc<Ident> {
        if let Some(ref ident) = self.dal_context_head {
            return ident.clone();
        }

        let dal_context_builder = self.setup_dal_context_builder();
        let dal_context_builder = dal_context_builder.as_ref();
        let transactions = self.setup_transactions();
        let transactions = transactions.as_ref();
        let bas = self.setup_billing_account_signup();
        let nba = bas.0.as_ref();

        let var = Ident::new("dal_context_head", Span::call_site());
        let application_id = self.setup_application_id();
        let application_id = application_id.as_ref();
        self.code.extend(quote! {
            let #var = {
                let request_context = ::dal::RequestContext::new_workspace_head(
                    &#transactions.pg(),
                    ::dal::HistoryActor::SystemInit,
                    *#nba.workspace.id(),
                    Some(#application_id),
                ).await.expect("failed to create new workspace head request context");

                ::dal::test::DalContextHead(
                    #dal_context_builder.build(request_context, &#transactions)
                );
            };
        });
        self.dal_context_head = Some(Arc::new(var));

        self.dal_context_head.as_ref().unwrap().clone()
    }

    fn setup_dal_context_head_ref(&mut self) -> Arc<Ident> {
        if let Some(ref ident) = self.dal_context_head_ref {
            return ident.clone();
        }

        let dal_context_builder = self.setup_dal_context_builder();
        let dal_context_builder = dal_context_builder.as_ref();
        let transactions = self.setup_transactions();
        let transactions = transactions.as_ref();
        let bas = self.setup_billing_account_signup();
        let nba = bas.0.as_ref();

        let var = Ident::new("dal_context_head_ref", Span::call_site());
        let application_id = self.setup_application_id();
        let application_id = application_id.as_ref();
        self.code.extend(quote! {
            let _dchr = {
                let request_context = ::dal::RequestContext::new_workspace_head(
                    &#transactions.pg(),
                    ::dal::HistoryActor::SystemInit,
                    *#nba.workspace.id(),
                    Some(#application_id),
                ).await.expect("failed to create new workspace head request context");

                #dal_context_builder.build(request_context, &#transactions)
            };
            let #var = ::dal::test::DalContextHeadRef(&_dchr);
        });
        self.dal_context_head_ref = Some(Arc::new(var));

        self.dal_context_head_ref.as_ref().unwrap().clone()
    }

    fn setup_dal_context_head_mut_ref(&mut self) -> Arc<Ident> {
        if let Some(ref ident) = self.dal_context_head_mut_ref {
            return ident.clone();
        }

        let dal_context_builder = self.setup_dal_context_builder();
        let dal_context_builder = dal_context_builder.as_ref();
        let transactions = self.setup_transactions();
        let transactions = transactions.as_ref();
        let bas = self.setup_billing_account_signup();
        let nba = bas.0.as_ref();

        let var = Ident::new("dal_context_head_mut_ref", Span::call_site());
        let application_id = self.setup_application_id();
        let application_id = application_id.as_ref();
        self.code.extend(quote! {
            let mut _dchmr = {
                let request_context = ::dal::RequestContext::new_workspace_head(
                    &#transactions.pg(),
                    ::dal::HistoryActor::SystemInit,
                    *#nba.workspace.id(),
                    Some(#application_id),
                ).await.expect("failed to create new workspace head request context");

                #dal_context_builder.build(request_context, &#transactions)
            };
            let #var = ::dal::test::DalContextHeadMutRef(&mut _dchmr);
        });
        self.dal_context_head_mut_ref = Some(Arc::new(var));

        self.dal_context_head_mut_ref.as_ref().unwrap().clone()
    }

    fn has_args(&self) -> bool {
        !self.args.is_empty()
    }

    fn finish(self) -> FnSetup {
        FnSetup {
            code: self.code,
            fn_args: self.args,
        }
    }
}

fn path_as_string(path: &Path) -> String {
    path.segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn expand_tracing_init() -> TokenStream {
    let span_events_env_var = SPAN_EVENTS_ENV_VAR;
    let log_env_var = LOG_ENV_VAR;

    quote! {
        let event_filter = {
            use ::tracing_subscriber::fmt::format::FmtSpan;

            match ::std::env::var(#span_events_env_var) {
                Ok(value) => {
                    value
                        .to_ascii_lowercase()
                        .split(",")
                        .map(|filter| match filter.trim() {
                            "new" => FmtSpan::NEW,
                            "enter" => FmtSpan::ENTER,
                            "exit" => FmtSpan::EXIT,
                            "close" => FmtSpan::CLOSE,
                            "active" => FmtSpan::ACTIVE,
                            "full" => FmtSpan::FULL,
                            _ => panic!(
                                "{}: {} must contain filters separated by `,`.\n\t\
                                For example: `active` or `new,close`\n\t
                                Got: {}",
                                concat!(env!("CARGO_PKG_NAME"), "::dal_test"),
                                #span_events_env_var,
                                value,
                            ),
                        })
                        .fold(FmtSpan::NONE, |acc, filter| filter | acc)
                },
                Err(::std::env::VarError::NotUnicode(_)) => {
                    panic!(
                        "{}: {} must contain a valid UTF-8 string",
                        concat!(env!("CARGO_PKG_NAME"), "::dal_test"),
                        #span_events_env_var,
                    )
                }
                Err(::std::env::VarError::NotPresent) => FmtSpan::NONE,
            }
        };

        let subscriber = ::tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(::tracing_subscriber::EnvFilter::from_env(#log_env_var))
            .with_span_events(event_filter)
            .with_test_writer()
            .finish();
        let _ = ::tracing::subscriber::set_global_default(subscriber);
    }
}

fn expand_runtime(worker_threads: usize, thread_stack_size: usize) -> TokenStream {
    quote! {
        ::tokio::runtime::Builder::new_multi_thread()
            .worker_threads(#worker_threads)
            .thread_stack_size(#thread_stack_size)
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
    }
}
