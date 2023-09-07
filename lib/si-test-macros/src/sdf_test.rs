//! Expansion implementation of the `sdf_test` attribute macro.
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

use std::rc::Rc;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{parse_quote, punctuated::Punctuated, token::Comma, Expr, FnArg, ItemFn, Type};

use crate::{
    expand::{expand_test, FnSetup, FnSetupExpander},
    path_as_string, Args,
};

pub(crate) fn expand(item: ItemFn, args: Args) -> TokenStream {
    let fn_setup = fn_setup(item.sig.inputs.iter());

    expand_test(item, args, fn_setup)
}

fn fn_setup<'a>(params: impl Iterator<Item = &'a FnArg>) -> SdfTestFnSetup {
    let mut expander = SdfTestFnSetupExpander::new();

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
                        // async fn does_things(wid: WorkspacePk) {
                        //      // ...
                        // }
                        // ```
                        //
                        // Note that several types such as `DalContextHead` may have interior
                        // references and/or mutability, however the surrounding type is passed as
                        // an owned type.
                        match ty_str {
                            "AuthToken" => {
                                let var = expander.setup_auth_token();
                                let var = var.as_ref();
                                expander.push_arg(parse_quote! {#var});
                            }
                            "AuthTokenRef" => {
                                let var = expander.setup_auth_token_ref();
                                let var = var.as_ref();
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
                            "Router" => {
                                let var = expander.setup_router();
                                let var = var.as_ref();
                                expander.push_arg(parse_quote! {#var});
                            }
                            "PingaShutdownHandle" => {
                                let var = expander.setup_pinga_shutdown_handle();
                                let var = var.as_ref();
                                expander.push_arg(parse_quote! {#var});
                            }
                            "RebaserShutdownHandle" => {
                                let var = expander.setup_rebaser_shutdown_handle();
                                let var = var.as_ref();
                                expander.push_arg(parse_quote! {#var});
                            }
                            "ServicesContext" => {
                                let var = expander.setup_services_context();
                                let var = var.as_ref();
                                expander.push_arg(parse_quote! {#var});
                            }
                            "VeritechShutdownHandle" => {
                                let var = expander.setup_veritech_shutdown_handle();
                                let var = var.as_ref();
                                expander.push_arg(parse_quote! {#var});
                            }
                            "WorkspacePk" => {
                                let var = expander.setup_workspace_pk();
                                let var = var.as_ref();
                                expander.push_arg(parse_quote! {#var});
                            }
                            "WorkspaceSignup" => {
                                let var = expander.setup_workspace_signup();
                                let var = var.0.as_ref();
                                expander.push_arg(parse_quote! {#var});
                            }
                            _ => panic!("unexpected argument type: {type_path:?}"),
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
                            //      ctx: &mut DalContext,
                            //      nw: &WorkspaceSignup
                            //  ) {
                            //      // ...
                            // }
                            // ```
                            //
                            // In the above example, both would be matched types in this section,
                            // even though `ctx` is a mutable reference and `nw` is an immutable
                            // reference.
                            match ty_str {
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
                                "ServicesContext" => {
                                    let var = expander.setup_services_context();
                                    let var = var.as_ref();
                                    expander.push_arg(parse_quote! {&#var});
                                }
                                "WorkspaceSignup" => {
                                    let var = expander.setup_workspace_signup();
                                    let var = var.0.as_ref();
                                    expander.push_arg(parse_quote! {&#var});
                                }
                                _ => panic!("unexpected argument reference type: {type_ref:?}"),
                            }
                        }
                    }
                    unsupported => {
                        panic!("argument reference type not supported: {unsupported:?}")
                    }
                },
                unsupported => panic!("argument type not supported: {unsupported:?}"),
            },
            FnArg::Receiver(_) => {
                panic!("argument does not support receiver/method style (i.e. using `self`)")
            }
        }
    }

    if expander.has_args() {
        // TODO(fnichol): we can use a macro attribute to opt-out and not run a veritech server in
        // the future, but for now (as before), every test starts with its own veritech server with
        // a randomized subject prefix
        expander.setup_start_veritech_server();
        expander.setup_start_pinga_server();
        expander.setup_start_council_server();
        expander.setup_start_rebaser_server();
    }

    expander.finish()
}

struct SdfTestFnSetup {
    code: TokenStream,
    fn_args: Punctuated<Expr, Comma>,
}

impl FnSetup for SdfTestFnSetup {
    fn into_parts(self) -> (TokenStream, Punctuated<Expr, Comma>) {
        (self.code, self.fn_args)
    }
}

struct SdfTestFnSetupExpander {
    code: TokenStream,
    args: Punctuated<Expr, Comma>,

    test_context: Option<Rc<Ident>>,
    nats_subject_prefix: Option<Rc<Ident>>,
    council_server: Option<Rc<Ident>>,
    start_council_server: Option<()>,
    pinga_server: Option<Rc<Ident>>,
    pinga_shutdown_handle: Option<Rc<Ident>>,
    start_pinga_server: Option<()>,
    rebaser_server: Option<Rc<Ident>>,
    rebaser_shutdown_handle: Option<Rc<Ident>>,
    start_rebaser_server: Option<()>,
    veritech_server: Option<Rc<Ident>>,
    veritech_shutdown_handle: Option<Rc<Ident>>,
    start_veritech_server: Option<()>,
    services_context: Option<Rc<Ident>>,
    dal_context_builder: Option<Rc<Ident>>,
    workspace_signup: Option<(Rc<Ident>, Rc<Ident>)>,
    workspace_pk: Option<Rc<Ident>>,
    dal_context_default: Option<Rc<Ident>>,
    dal_context_default_mut: Option<Rc<Ident>>,
    dal_context_head: Option<Rc<Ident>>,
    dal_context_head_ref: Option<Rc<Ident>>,
    dal_context_head_mut_ref: Option<Rc<Ident>>,
    jwt_public_signing_key: Option<Rc<Ident>>,
    signup_secret: Option<Rc<Ident>>,
    posthog_client: Option<Rc<Ident>>,
    router: Option<Rc<Ident>>,
    auth_token: Option<Rc<Ident>>,
    auth_token_ref: Option<Rc<Ident>>,
}

impl SdfTestFnSetupExpander {
    fn new() -> Self {
        Self {
            code: TokenStream::new(),
            args: Punctuated::new(),
            test_context: None,
            nats_subject_prefix: None,
            council_server: None,
            start_council_server: None,
            pinga_server: None,
            pinga_shutdown_handle: None,
            start_pinga_server: None,
            rebaser_server: None,
            rebaser_shutdown_handle: None,
            start_rebaser_server: None,
            veritech_server: None,
            veritech_shutdown_handle: None,
            start_veritech_server: None,
            services_context: None,
            dal_context_builder: None,
            workspace_signup: None,
            workspace_pk: None,
            dal_context_default: None,
            dal_context_default_mut: None,
            dal_context_head: None,
            dal_context_head_ref: None,
            dal_context_head_mut_ref: None,
            jwt_public_signing_key: None,
            signup_secret: None,
            posthog_client: None,
            router: None,
            auth_token: None,
            auth_token_ref: None,
        }
    }

    fn has_args(&self) -> bool {
        !self.args.is_empty()
    }

    fn setup_jwt_public_signing_key(&mut self) -> Rc<Ident> {
        if let Some(ref ident) = self.jwt_public_signing_key {
            return ident.clone();
        }

        let var = Ident::new("jwt_public_signing_key", Span::call_site());
        self.code_extend(quote! {
            let #var = ::dal_test::jwt_public_signing_key().await?;
        });
        self.jwt_public_signing_key = Some(Rc::new(var));

        self.jwt_public_signing_key.as_ref().unwrap().clone()
    }

    fn setup_signup_secret(&mut self) -> Rc<Ident> {
        if let Some(ref ident) = self.signup_secret {
            return ident.clone();
        }

        let var = Ident::new("signup_secret", Span::call_site());
        self.code_extend(quote! {
            let #var: ::si_std::SensitiveString = "sign-me-up".into();
        });
        self.signup_secret = Some(Rc::new(var));

        self.signup_secret.as_ref().unwrap().clone()
    }

    fn setup_posthog_client(&mut self) -> Rc<Ident> {
        if let Some(ref ident) = self.posthog_client {
            return ident.clone();
        }

        let var = Ident::new("posthog_client", Span::call_site());
        self.code_extend(quote! {
            let #var = {
                let (client, sender) = ::si_posthog::new()
                    .api_endpoint("http://localhost:9999")
                    .api_key("not-a-key")
                    .enabled(false)
                    .build()
                    .wrap_err("failed to create posthost client and sender")?;
                drop(::tokio::spawn(sender.run()));
                client
            };
        });
        self.posthog_client = Some(Rc::new(var));

        self.posthog_client.as_ref().unwrap().clone()
    }

    fn setup_router(&mut self) -> Rc<Ident> {
        if let Some(ref ident) = self.router {
            return ident.clone();
        }

        let test_context = self.setup_test_context();
        let test_context = test_context.as_ref();
        let jwt_public_signing_key = self.setup_jwt_public_signing_key();
        let jwt_public_signing_key = jwt_public_signing_key.as_ref();
        let signup_secret = self.setup_signup_secret();
        let signup_secret = signup_secret.as_ref();
        let posthog_client = self.setup_posthog_client();
        let posthog_client = posthog_client.as_ref();

        let var = Ident::new("router", Span::call_site());
        self.code_extend(quote! {
            let #var = {
                let s_ctx = #test_context.create_services_context().await;
                let (service, _, _) = ::sdf_server::build_service_for_tests(
                    s_ctx,
                    #jwt_public_signing_key.clone(),
                    #signup_secret.clone(),
                    #posthog_client,
                ).wrap_err("failed to build sdf router")?;
                service
            };
        });
        self.router = Some(Rc::new(var));

        self.router.as_ref().unwrap().clone()
    }

    fn setup_auth_token(&mut self) -> Rc<Ident> {
        if let Some(ref ident) = self.auth_token {
            return ident.clone();
        }

        let workspace_signup = self.setup_workspace_signup();
        let auth_token = workspace_signup.1.as_ref();

        let var = Ident::new("auth_token_owned", Span::call_site());
        self.code_extend(quote! {
            let #var = ::dal_test::AuthToken(#auth_token.clone());
        });
        self.auth_token = Some(Rc::new(var));

        self.auth_token.as_ref().unwrap().clone()
    }

    fn setup_auth_token_ref(&mut self) -> Rc<Ident> {
        if let Some(ref ident) = self.auth_token_ref {
            return ident.clone();
        }

        let workspace_signup = self.setup_workspace_signup();
        let auth_token = workspace_signup.1.as_ref();

        let var = Ident::new("auth_token_ref", Span::call_site());
        self.code_extend(quote! {
            let #var = ::dal_test::AuthTokenRef(&#auth_token);
        });
        self.auth_token_ref = Some(Rc::new(var));

        self.auth_token_ref.as_ref().unwrap().clone()
    }

    fn finish(self) -> SdfTestFnSetup {
        SdfTestFnSetup {
            code: self.code,
            fn_args: self.args,
        }
    }
}

impl FnSetupExpander for SdfTestFnSetupExpander {
    fn code_extend<I: IntoIterator<Item = proc_macro2::TokenTree>>(&mut self, stream: I) {
        self.code.extend(stream)
    }

    fn push_arg(&mut self, arg: Expr) {
        self.args.push(arg);
    }

    fn test_context(&self) -> Option<&Rc<Ident>> {
        self.test_context.as_ref()
    }

    fn set_test_context(&mut self, value: Option<Rc<Ident>>) {
        self.test_context = value;
    }

    fn nats_subject_prefix(&self) -> Option<&Rc<Ident>> {
        self.nats_subject_prefix.as_ref()
    }

    fn set_nats_subject_prefix(&mut self, value: Option<Rc<Ident>>) {
        self.nats_subject_prefix = value;
    }

    fn council_server(&self) -> Option<&Rc<Ident>> {
        self.council_server.as_ref()
    }

    fn set_council_server(&mut self, value: Option<Rc<Ident>>) {
        self.council_server = value;
    }

    fn start_council_server(&self) -> Option<()> {
        self.start_council_server
    }

    fn set_start_council_server(&mut self, value: Option<()>) {
        self.start_council_server = value;
    }

    fn pinga_server(&self) -> Option<&Rc<Ident>> {
        self.pinga_server.as_ref()
    }

    fn set_pinga_server(&mut self, value: Option<Rc<Ident>>) {
        self.pinga_server = value;
    }

    fn pinga_shutdown_handle(&self) -> Option<&Rc<Ident>> {
        self.pinga_shutdown_handle.as_ref()
    }

    fn set_pinga_shutdown_handle(&mut self, value: Option<Rc<Ident>>) {
        self.pinga_shutdown_handle = value;
    }

    fn start_pinga_server(&self) -> Option<()> {
        self.start_pinga_server
    }

    fn set_start_pinga_server(&mut self, value: Option<()>) {
        self.start_pinga_server = value;
    }

    fn rebaser_server(&self) -> Option<&Rc<Ident>> {
        self.rebaser_server.as_ref()
    }

    fn set_rebaser_server(&mut self, value: Option<Rc<Ident>>) {
        self.rebaser_server = value;
    }

    fn rebaser_shutdown_handle(&self) -> Option<&Rc<Ident>> {
        self.rebaser_shutdown_handle.as_ref()
    }

    fn set_rebaser_shutdown_handle(&mut self, value: Option<Rc<Ident>>) {
        self.rebaser_shutdown_handle = value;
    }

    fn start_rebaser_server(&self) -> Option<()> {
        self.start_rebaser_server
    }

    fn set_start_rebaser_server(&mut self, value: Option<()>) {
        self.start_rebaser_server = value;
    }

    fn veritech_server(&self) -> Option<&Rc<Ident>> {
        self.veritech_server.as_ref()
    }

    fn set_veritech_server(&mut self, value: Option<Rc<Ident>>) {
        self.veritech_server = value;
    }

    fn veritech_shutdown_handle(&self) -> Option<&Rc<Ident>> {
        self.veritech_shutdown_handle.as_ref()
    }

    fn set_veritech_shutdown_handle(&mut self, value: Option<Rc<Ident>>) {
        self.veritech_shutdown_handle = value;
    }

    fn start_veritech_server(&self) -> Option<()> {
        self.start_veritech_server
    }

    fn set_start_veritech_server(&mut self, value: Option<()>) {
        self.start_veritech_server = value;
    }

    fn services_context(&self) -> Option<&Rc<Ident>> {
        self.services_context.as_ref()
    }

    fn set_services_context(&mut self, value: Option<Rc<Ident>>) {
        self.services_context = value;
    }

    fn dal_context_builder(&self) -> Option<&Rc<Ident>> {
        self.dal_context_builder.as_ref()
    }

    fn set_dal_context_builder(&mut self, value: Option<Rc<Ident>>) {
        self.dal_context_builder = value;
    }

    fn workspace_signup(&self) -> Option<&(Rc<Ident>, Rc<Ident>)> {
        self.workspace_signup.as_ref()
    }

    fn set_workspace_signup(&mut self, value: Option<(Rc<Ident>, Rc<Ident>)>) {
        self.workspace_signup = value;
    }

    fn workspace_pk(&self) -> Option<&Rc<Ident>> {
        self.workspace_pk.as_ref()
    }

    fn set_workspace_pk(&mut self, value: Option<Rc<Ident>>) {
        self.workspace_pk = value;
    }

    fn dal_context_default(&self) -> Option<&Rc<Ident>> {
        self.dal_context_default.as_ref()
    }

    fn set_dal_context_default(&mut self, value: Option<Rc<Ident>>) {
        self.dal_context_default = value;
    }

    fn dal_context_default_mut(&self) -> Option<&Rc<Ident>> {
        self.dal_context_default_mut.as_ref()
    }

    fn set_dal_context_default_mut(&mut self, value: Option<Rc<Ident>>) {
        self.dal_context_default_mut = value;
    }

    fn dal_context_head(&self) -> Option<&Rc<Ident>> {
        self.dal_context_head.as_ref()
    }

    fn set_dal_context_head(&mut self, value: Option<Rc<Ident>>) {
        self.dal_context_head = value;
    }

    fn dal_context_head_ref(&self) -> Option<&Rc<Ident>> {
        self.dal_context_head_ref.as_ref()
    }

    fn set_dal_context_head_ref(&mut self, value: Option<Rc<Ident>>) {
        self.dal_context_head_ref = value;
    }

    fn dal_context_head_mut_ref(&self) -> Option<&Rc<Ident>> {
        self.dal_context_head_mut_ref.as_ref()
    }

    fn set_dal_context_head_mut_ref(&mut self, value: Option<Rc<Ident>>) {
        self.dal_context_head_mut_ref = value;
    }
}
