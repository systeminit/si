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

use std::rc::Rc;

use proc_macro2::{Ident, TokenStream};
use syn::{parse_quote, punctuated::Punctuated, token::Comma, Expr, FnArg, ItemFn, Type};

use crate::{
    expand::{expand_test, FnSetup, FnSetupExpander},
    path_as_string, Args,
};

pub(crate) fn expand(item: ItemFn, args: Args) -> TokenStream {
    let fn_setup = fn_setup(item.sig.inputs.iter());

    expand_test(item, args, fn_setup)
}

fn fn_setup<'a>(params: impl Iterator<Item = &'a FnArg>) -> DalTestFnSetup {
    let mut expander = DalTestFnSetupExpander::new();

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

struct DalTestFnSetup {
    code: TokenStream,
    fn_args: Punctuated<Expr, Comma>,
}

impl FnSetup for DalTestFnSetup {
    fn into_parts(self) -> (TokenStream, Punctuated<Expr, Comma>) {
        (self.code, self.fn_args)
    }
}

struct DalTestFnSetupExpander {
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
}

impl DalTestFnSetupExpander {
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
        }
    }

    fn has_args(&self) -> bool {
        !self.args.is_empty()
    }

    fn finish(self) -> DalTestFnSetup {
        DalTestFnSetup {
            code: self.code,
            fn_args: self.args,
        }
    }
}

impl FnSetupExpander for DalTestFnSetupExpander {
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
