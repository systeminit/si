use std::rc::Rc;

use proc_macro2::{
    Ident,
    Span,
    TokenStream,
    TokenTree,
};
use quote::quote;
use syn::{
    Expr,
    ItemFn,
    ReturnType,
    punctuated::Punctuated,
    token::Comma,
};

use crate::{
    Args,
    LOG_ENV_VAR,
    RT_DEFAULT_THREAD_STACK_SIZE,
    RT_DEFAULT_WORKER_THREADS,
    SPAN_EVENTS_ENV_VAR,
};

pub(crate) trait FnSetup {
    fn into_parts(self) -> (TokenStream, Punctuated<Expr, Comma>);
}

pub(crate) fn expand_test(item: ItemFn, _args: Args, fn_setup: impl FnSetup) -> TokenStream {
    if item.sig.asyncness.is_none() {
        panic!("test function must be async--blocking tests not supported");
    }
    let attrs = &item.attrs;
    let body = &item.block;
    let test_name = &item.sig.ident;
    let params = &item.sig.inputs;
    // Note that Rust doesn't allow a test function with `#[should_panic]` that has a non-unit
    // return value. Huh
    let (rt_is_result, output) = match &item.sig.output {
        ReturnType::Default => (false, quote! {}),
        ReturnType::Type(_, typeness) => (true, quote! {-> #typeness}),
    };
    let test_attr = quote! {#[::core::prelude::v1::test]};

    let thread_stack_size = RT_DEFAULT_THREAD_STACK_SIZE;

    let (fn_setups, fn_args) = fn_setup.into_parts();

    let fn_call = if rt_is_result {
        quote! {let _ = test_fn(#fn_args).await?;}
    } else {
        quote! {test_fn(#fn_args).await;}
    };
    let color_eyre_init = expand_color_eyre_init();
    let tracing_init = expand_tracing_init();
    let rt = expand_default_runtime();

    quote! {
        #test_attr
        #(#attrs)*
        fn #test_name() -> ::dal_test::Result<()> {
            use ::dal_test::WrapErr;
            use ::std::io::Write;
            use ::dal_test::telemetry::tracing;

            // Could add a custom `name` attribute here to send through the test name
            // SI_TEST_LOG=test_integration::integration_test::rebaser=info,off SI_TEST_LOG_SPAN_EVENTS=new,close buck2 run lib/dal:test-integration -- integration_test::rebaser --nocapture
            #[tracing::instrument(level = "info", name="example-test-name", skip_all)]
            async fn test_fn(#params) #output #body

            #[inline]
            async fn spawned_task() -> ::dal_test::Result<()> {
                #fn_setups
                #fn_call
                Ok(())
            }

            ::dal_test::COLOR_EYRE_INIT.call_once(|| {
                #color_eyre_init
                #tracing_init
            });
            let start = ::std::time::Instant::now();
            let thread_builder = ::std::thread::Builder::new().stack_size(#thread_stack_size);
            let thread_join_handle = thread_builder.spawn(|| {
                #[allow(clippy::expect_used)]
                #rt.block_on(spawned_task())
            }).expect("failed to spawn thread at OS level");
            let test_result = match thread_join_handle.join() {
                Ok(r) => r,
                Err(err) => {
                    // Spawned test task panicked
                    ::std::panic::resume_unwind(err);
                }
            };
            let _ = test_result?;
            let dur = ::std::time::Instant::now();
            let secs = dur.duration_since(start).as_secs();
            write!(
                        ::std::io::stderr(),
                        " (took {} seconds) ",
                        secs
                    )?;
            Ok(())
        }
    }
}

pub(crate) fn expand_color_eyre_init() -> TokenStream {
    quote! {
        ::dal_test::color_eyre::config::HookBuilder::default()
            .add_frame_filter(Box::new(|frames| {
                let mut displayed = ::std::collections::HashSet::new();
                let filters = &[
                    "tokio::",
                    "<futures_util::",
                    "std::panic",
                    "test::run_test_in_process",
                    "core::ops::function::FnOnce::call_once",
                    "std::thread::local",
                    "<core::future::",
                    "<alloc::boxed::Box",
                    "<std::panic::AssertUnwindSafe",
                    "core::result::Result",
                    "<T as futures_util",
                    "<tracing_futures::Instrumented",
                    "test::assert_test_result",
                    "spandoc::",
                ];

                frames.retain(|frame| {
                    let loc = (frame.lineno, &frame.filename);
                    let inserted = displayed.insert(loc);

                    if !inserted {
                        return false;
                    }

                    !filters.iter().any(|f| {
                        let name = if let Some(name) = frame.name.as_ref() {
                            name.as_str()
                        } else {
                            return true;
                        };

                        name.starts_with(f)
                    })
                });
            }))
            .install()
            .unwrap();
    }
}

fn expand_tracing_init() -> TokenStream {
    let span_events_env_var = SPAN_EVENTS_ENV_VAR;
    let log_env_var = LOG_ENV_VAR;
    quote! {
        ::dal_test::expand_helpers::tracing_init(#span_events_env_var, #log_env_var);
    }
}

fn expand_default_runtime() -> TokenStream {
    let worker_threads = RT_DEFAULT_WORKER_THREADS;
    let thread_stack_size = RT_DEFAULT_THREAD_STACK_SIZE;

    expand_runtime(worker_threads, thread_stack_size)
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

pub(crate) trait FnSetupExpander {
    fn code_extend<I: IntoIterator<Item = TokenTree>>(&mut self, stream: I);
    fn push_arg(&mut self, arg: Expr);

    fn test_context(&self) -> Option<&Rc<Ident>>;
    fn set_test_context(&mut self, value: Option<Rc<Ident>>);

    fn cancellation_token(&self) -> Option<&Rc<Ident>>;
    fn set_cancellation_token(&mut self, value: Option<Rc<Ident>>);

    fn task_tracker(&self) -> Option<&Rc<Ident>>;
    fn set_task_tracker(&mut self, value: Option<Rc<Ident>>);

    fn pinga_server(&self) -> Option<&Rc<Ident>>;
    fn set_pinga_server(&mut self, value: Option<Rc<Ident>>);

    fn start_pinga_server(&self) -> Option<()>;
    fn set_start_pinga_server(&mut self, value: Option<()>);

    fn edda_server(&self) -> Option<&Rc<Ident>>;
    fn set_edda_server(&mut self, value: Option<Rc<Ident>>);

    fn start_edda_server(&self) -> Option<()>;
    fn set_start_edda_server(&mut self, value: Option<()>);

    fn rebaser_server(&self) -> Option<&Rc<Ident>>;
    fn set_rebaser_server(&mut self, value: Option<Rc<Ident>>);

    fn start_rebaser_server(&self) -> Option<()>;
    fn set_start_rebaser_server(&mut self, value: Option<()>);

    fn forklift_server(&self) -> Option<&Rc<Ident>>;
    fn set_forklift_server(&mut self, value: Option<Rc<Ident>>);

    fn start_forklift_server(&self) -> Option<()>;
    fn set_start_forklift_server(&mut self, value: Option<()>);

    fn veritech_server(&self) -> Option<&Rc<Ident>>;
    fn set_veritech_server(&mut self, value: Option<Rc<Ident>>);

    fn start_veritech_server(&self) -> Option<()>;
    fn set_start_veritech_server(&mut self, value: Option<()>);

    fn services_context(&self) -> Option<&Rc<Ident>>;
    fn set_services_context(&mut self, value: Option<Rc<Ident>>);

    fn dal_context_builder(&self) -> Option<&Rc<Ident>>;
    fn set_dal_context_builder(&mut self, value: Option<Rc<Ident>>);

    fn workspace_signup(&self) -> Option<&(Rc<Ident>, Rc<Ident>)>;
    fn set_workspace_signup(&mut self, value: Option<(Rc<Ident>, Rc<Ident>)>);

    fn workspace_pk(&self) -> Option<&Rc<Ident>>;
    fn set_workspace_pk(&mut self, value: Option<Rc<Ident>>);

    fn dal_context_default(&self) -> Option<&Rc<Ident>>;
    fn set_dal_context_default(&mut self, value: Option<Rc<Ident>>);

    fn dal_context_default_mut(&self) -> Option<&Rc<Ident>>;
    fn set_dal_context_default_mut(&mut self, value: Option<Rc<Ident>>);

    fn dal_context_head(&self) -> Option<&Rc<Ident>>;
    fn set_dal_context_head(&mut self, value: Option<Rc<Ident>>);

    fn dal_context_head_ref(&self) -> Option<&Rc<Ident>>;
    fn set_dal_context_head_ref(&mut self, value: Option<Rc<Ident>>);

    fn dal_context_head_mut_ref(&self) -> Option<&Rc<Ident>>;
    fn set_dal_context_head_mut_ref(&mut self, value: Option<Rc<Ident>>);

    fn setup_test_context(&mut self) -> Rc<Ident> {
        if let Some(ident) = self.test_context() {
            return ident.clone();
        }

        let var = Ident::new("test_context", Span::call_site());
        self.code_extend(quote! {
            let test_context = ::dal_test::TestContext::global(
                crate::TEST_PG_DBNAME,
                crate::SI_TEST_LAYER_CACHE_PG_DBNAME,
                crate::SI_TEST_AUDIT_PG_DBNAME
            ).await?;
        });
        self.set_test_context(Some(Rc::new(var)));

        self.test_context().unwrap().clone()
    }

    fn setup_cancellation_token(&mut self) -> Rc<Ident> {
        if let Some(ident) = self.cancellation_token() {
            return ident.clone();
        }

        let var = Ident::new("cancellation_token", Span::call_site());
        self.code_extend(quote! {
            let cancellation_token = ::tokio_util::sync::CancellationToken::new();
        });
        self.set_cancellation_token(Some(Rc::new(var)));

        self.cancellation_token().unwrap().clone()
    }

    fn setup_task_tracker(&mut self) -> Rc<Ident> {
        if let Some(ident) = self.task_tracker() {
            return ident.clone();
        }

        let var = Ident::new("task_tracker", Span::call_site());
        self.code_extend(quote! {
            let task_tracker = ::tokio_util::task::TaskTracker::new();
        });
        self.set_task_tracker(Some(Rc::new(var)));

        self.task_tracker().unwrap().clone()
    }

    fn setup_pinga_server(&mut self) -> Rc<Ident> {
        if let Some(ident) = self.pinga_server() {
            return ident.clone();
        }

        let test_context = self.setup_test_context();
        let test_context = test_context.as_ref();

        let cancellation_token = self.setup_cancellation_token();
        let cancellation_token = cancellation_token.as_ref();

        let task_tracker = self.setup_task_tracker();
        let task_tracker = task_tracker.as_ref();

        let var = Ident::new("pinga_server", Span::call_site());
        self.code_extend(quote! {
            let #var = {
                let s_ctx = #test_context
                    .create_services_context(
                        #cancellation_token.clone(),
                        #task_tracker.clone(),
                    )
                    .await;
                ::dal_test::pinga_server(
                    s_ctx,
                    #cancellation_token.clone(),
                ).await?
            };
        });
        self.set_pinga_server(Some(Rc::new(var)));

        self.pinga_server().unwrap().clone()
    }

    fn setup_start_pinga_server(&mut self) {
        if self.start_pinga_server().is_some() {
            return;
        }

        let pinga_server = self.setup_pinga_server();
        let pinga_server = pinga_server.as_ref();

        self.code_extend(quote! {
            ::tokio::spawn(#pinga_server.run());
        });
        self.set_start_pinga_server(Some(()));
    }

    fn setup_edda_server(&mut self) -> Rc<Ident> {
        if let Some(ident) = self.edda_server() {
            return ident.clone();
        }

        let test_context = self.setup_test_context();
        let test_context = test_context.as_ref();

        let cancellation_token = self.setup_cancellation_token();
        let cancellation_token = cancellation_token.as_ref();

        let task_tracker = self.setup_task_tracker();
        let task_tracker = task_tracker.as_ref();

        let var = Ident::new("edda_server", Span::call_site());
        self.code_extend(quote! {
            let #var = {
                let s_ctx = #test_context
                    .create_services_context(
                        #cancellation_token.clone(),
                        #task_tracker.clone(),
                    )
                    .await;
                ::dal_test::edda_server(
                    s_ctx,
                    #cancellation_token.clone(),
                ).await?
            };
        });
        self.set_edda_server(Some(Rc::new(var)));

        self.edda_server().unwrap().clone()
    }

    fn setup_start_edda_server(&mut self) {
        if self.start_edda_server().is_some() {
            return;
        }

        let edda_server = self.setup_edda_server();
        let edda_server = edda_server.as_ref();

        self.code_extend(quote! {
            ::tokio::spawn(#edda_server.run());
        });
        self.set_start_edda_server(Some(()));
    }

    fn setup_rebaser_server(&mut self) -> Rc<Ident> {
        if let Some(ident) = self.rebaser_server() {
            return ident.clone();
        }

        let test_context = self.setup_test_context();
        let test_context = test_context.as_ref();

        let cancellation_token = self.setup_cancellation_token();
        let cancellation_token = cancellation_token.as_ref();

        let task_tracker = self.setup_task_tracker();
        let task_tracker = task_tracker.as_ref();
        let var = Ident::new("rebaser_server", Span::call_site());

        self.code_extend(quote! {
            let #var = {
                let s_ctx = #test_context
                    .create_services_context(
                        #cancellation_token.clone(),
                        #task_tracker.clone(),
                    )
                    .await;
                ::dal_test::rebaser_server(
                    s_ctx,
                    #cancellation_token.clone(),
                )
                .await?
            };
        });
        self.set_rebaser_server(Some(Rc::new(var)));

        self.rebaser_server().unwrap().clone()
    }

    fn setup_start_rebaser_server(&mut self) {
        if self.start_rebaser_server().is_some() {
            return;
        }

        let rebaser_server = self.setup_rebaser_server();
        let rebaser_server = rebaser_server.as_ref();

        self.code_extend(quote! {
            ::tokio::spawn(#rebaser_server.run());
        });
        self.set_start_rebaser_server(Some(()));
    }

    fn setup_veritech_server(&mut self) -> Rc<Ident> {
        if let Some(ident) = self.veritech_server() {
            return ident.clone();
        }

        let test_context = self.setup_test_context();
        let test_context = test_context.as_ref();

        let cancellation_token = self.setup_cancellation_token();
        let cancellation_token = cancellation_token.as_ref();

        let var = Ident::new("veritech_server", Span::call_site());
        self.code_extend(quote! {
            let #var = ::dal_test::veritech_server_for_uds_cyclone(
                #test_context.nats_config().clone(),
                #cancellation_token.clone(),
            ).await?;
        });
        self.set_veritech_server(Some(Rc::new(var)));

        self.veritech_server().unwrap().clone()
    }

    fn setup_start_veritech_server(&mut self) {
        if self.start_veritech_server().is_some() {
            return;
        }

        let veritech_server = self.setup_veritech_server();
        let veritech_server = veritech_server.as_ref();

        self.code_extend(quote! {
            ::tokio::spawn(#veritech_server.run());
        });
        self.set_start_veritech_server(Some(()));
    }

    fn setup_forklift_server(&mut self) -> Rc<Ident> {
        if let Some(ident) = self.forklift_server() {
            return ident.clone();
        }

        let test_context = self.setup_test_context();
        let test_context = test_context.as_ref();

        let cancellation_token = self.setup_cancellation_token();
        let cancellation_token = cancellation_token.as_ref();

        let var = Ident::new("forklift_server", Span::call_site());
        self.code_extend(quote! {
            let #var = ::dal_test::forklift_server(
                #test_context.nats_conn().clone(),
                #test_context.audit_database_context().to_owned(),
                #cancellation_token.clone(),
            ).await?;
        });
        self.set_forklift_server(Some(Rc::new(var)));

        self.forklift_server().unwrap().clone()
    }

    fn setup_start_forklift_server(&mut self) {
        if self.start_forklift_server().is_some() {
            return;
        }

        let forklift_server = self.setup_forklift_server();
        let forklift_server = forklift_server.as_ref();

        self.code_extend(quote! {
            ::tokio::spawn(#forklift_server.run());
        });
        self.set_start_forklift_server(Some(()));
    }

    fn setup_services_context(&mut self) -> Rc<Ident> {
        if let Some(ident) = self.services_context() {
            return ident.clone();
        }

        let test_context = self.setup_test_context();
        let test_context = test_context.as_ref();

        let cancellation_token = self.setup_cancellation_token();
        let cancellation_token = cancellation_token.as_ref();

        let task_tracker = self.setup_task_tracker();
        let task_tracker = task_tracker.as_ref();

        let var = Ident::new("services_context", Span::call_site());
        self.code_extend(quote! {
            let #var = #test_context
                .create_services_context(#cancellation_token.clone(), #task_tracker.clone())
                .await;
        });
        self.set_services_context(Some(Rc::new(var)));

        self.services_context().unwrap().clone()
    }

    fn setup_dal_context_builder(&mut self) -> Rc<Ident> {
        if let Some(ident) = self.dal_context_builder() {
            return ident.clone();
        }

        let services_context = self.setup_services_context();
        let services_context = services_context.as_ref();

        let var = Ident::new("dal_context_builder", Span::call_site());

        self.code_extend(quote! {
            let #var = #services_context.clone().into_builder(false);
        });

        self.set_dal_context_builder(Some(Rc::new(var)));

        self.dal_context_builder().unwrap().clone()
    }

    fn setup_workspace_signup(&mut self) -> (Rc<Ident>, Rc<Ident>) {
        if let Some(idents) = self.workspace_signup() {
            return idents.clone();
        }

        let dal_context_builder = self.setup_dal_context_builder();
        let dal_context_builder = dal_context_builder.as_ref();

        let var_nw = Ident::new("nw", Span::call_site());
        let var_auth_token = Ident::new("auth_token", Span::call_site());
        self.code_extend(quote! {
            let (#var_nw, #var_auth_token) = {
                let ctx = #dal_context_builder
                    .build_default(None)
                    .await
                    .wrap_err("failed to build default dal ctx for workspace_signup")?;
                let r = ::dal_test::expand_helpers::workspace_signup(&ctx).await?;
                ctx.blocking_commit()
                    .await
                    .wrap_err("failed to commit workspace_signup")?;

                r
            };
        });
        self.set_workspace_signup(Some((Rc::new(var_nw), Rc::new(var_auth_token))));

        self.workspace_signup().unwrap().clone()
    }

    fn setup_workspace_pk(&mut self) -> Rc<Ident> {
        if let Some(idents) = self.workspace_pk() {
            return idents.clone();
        }

        let bas = self.setup_workspace_signup();
        let nw = bas.0.as_ref();

        let var = Ident::new("nw_workspace_pk", Span::call_site());
        self.code_extend(quote! {
            let #var = *#nw.workspace.pk();
        });
        self.set_workspace_pk(Some(Rc::new(var)));

        self.workspace_pk().unwrap().clone()
    }

    fn setup_dal_context_default(&mut self) -> Rc<Ident> {
        if let Some(ident) = self.dal_context_default() {
            return ident.clone();
        }

        let dal_context_builder = self.setup_dal_context_builder();
        let dal_context_builder = dal_context_builder.as_ref();
        let bas = self.setup_workspace_signup();
        let nw = bas.0.as_ref();

        let var = Ident::new("default_dal_context", Span::call_site());
        self.code_extend(quote! {
            let #var = {
                let mut ctx = #dal_context_builder
                    .build_default(None)
                    .await
                    .wrap_err("failed to build default dal ctx for dal_context_default")?;
                ctx.update_tenancy(::dal::Tenancy::new(*#nw.workspace.pk()));
                ::dal_test::expand_helpers::create_change_set_and_update_ctx(&mut ctx, #nw.workspace.default_change_set_id()).await;
                ::dal_test::expand_helpers::setup_history_actor_ctx(&mut ctx).await;
                ctx.commit_no_rebase()
                    .await
                    .wrap_err("failed to commit create_change_set_and_update_ctx")?;

                ctx
            };
        });
        self.set_dal_context_default(Some(Rc::new(var)));

        self.dal_context_default().unwrap().clone()
    }

    fn setup_dal_context_default_mut(&mut self) -> Rc<Ident> {
        if let Some(ident) = self.dal_context_default_mut() {
            return ident.clone();
        }

        let dal_context_builder = self.setup_dal_context_builder();
        let dal_context_builder = dal_context_builder.as_ref();
        let bas = self.setup_workspace_signup();
        let nw = bas.0.as_ref();

        let var = Ident::new("dal_context_default_mut", Span::call_site());
        self.code_extend(quote! {
            let mut #var = {
                let mut ctx = #dal_context_builder
                    .build_default(None)
                    .await
                    .wrap_err("failed to build default dal ctx for dal_context_default_mut")?;
                ctx.update_tenancy(::dal::Tenancy::new(*#nw.workspace.pk()));
                ::dal_test::expand_helpers::create_change_set_and_update_ctx(&mut ctx, #nw.workspace.default_change_set_id()).await;
                ::dal_test::expand_helpers::setup_history_actor_ctx(&mut ctx).await;
                ctx.commit_no_rebase()
                    .await
                    .wrap_err("failed to commit create_change_set_and_update_ctx_mut")?;

                ctx
            };
        });
        self.set_dal_context_default_mut(Some(Rc::new(var)));

        self.dal_context_default_mut().unwrap().clone()
    }

    fn setup_dal_context_head(&mut self) -> Rc<Ident> {
        if let Some(ident) = self.dal_context_head() {
            return ident.clone();
        }

        let dal_context_builder = self.setup_dal_context_builder();
        let dal_context_builder = dal_context_builder.as_ref();
        let bas = self.setup_workspace_signup();
        let nw = bas.0.as_ref();

        let var = Ident::new("dal_context_head", Span::call_site());
        self.code_extend(quote! {
            let #var = {
                let mut ctx = #dal_context_builder
                    .build_default(None)
                    .await
                    .wrap_err("failed to build default dal ctx for dal_context_head")?;
                ctx.update_tenancy(::dal::Tenancy::new(*#nw.workspace.pk()));

                ::dal_test::DalContextHead(ctx)
            };
        });
        self.set_dal_context_head(Some(Rc::new(var)));

        self.dal_context_head().unwrap().clone()
    }

    fn setup_dal_context_head_ref(&mut self) -> Rc<Ident> {
        if let Some(ident) = self.dal_context_head_ref() {
            return ident.clone();
        }

        let dal_context_builder = self.setup_dal_context_builder();
        let dal_context_builder = dal_context_builder.as_ref();
        let bas = self.setup_workspace_signup();
        let nw = bas.0.as_ref();

        let var = Ident::new("dal_context_head_ref", Span::call_site());
        self.code_extend(quote! {
            let _dchr = {
                let mut ctx = #dal_context_builder
                    .build_default(None)
                    .await
                    .wrap_err("failed to build default dal ctx for dal_context_head_ref")?;
                ctx.update_tenancy(::dal::Tenancy::new(*#nw.workspace.pk()));

                ctx
            };
            let #var = ::dal_test::DalContextHeadRef(&_dchr);
        });
        self.set_dal_context_head_ref(Some(Rc::new(var)));

        self.dal_context_head_ref().unwrap().clone()
    }

    fn setup_dal_context_head_mut_ref(&mut self) -> Rc<Ident> {
        if let Some(ident) = self.dal_context_head_mut_ref() {
            return ident.clone();
        }

        let dal_context_builder = self.setup_dal_context_builder();
        let dal_context_builder = dal_context_builder.as_ref();
        let bas = self.setup_workspace_signup();
        let nw = bas.0.as_ref();

        let var = Ident::new("dal_context_head_mut_ref", Span::call_site());
        self.code_extend(quote! {
            let mut _dchmr = {
                let mut ctx = #dal_context_builder
                    .build_default(None)
                    .await
                    .wrap_err("failed to build default dal ctx for dal_context_head_mut_ref")?;
                ctx.update_tenancy(::dal::Tenancy::new(*#nw.workspace.pk()));

                ctx
            };
            let #var = ::dal_test::DalContextHeadMutRef(&mut _dchmr);
        });
        self.set_dal_context_head_mut_ref(Some(Rc::new(var)));

        self.dal_context_head_mut_ref().unwrap().clone()
    }

    fn setup_audit_database_context(&mut self) -> Rc<Ident> {
        let test_context = self.setup_test_context();
        let test_context = test_context.as_ref();

        let var_audit_database_context = Ident::new("audit_database_context", Span::call_site());
        self.code_extend(quote! {
            let #var_audit_database_context  = {
                let r = #test_context.audit_database_context().to_owned();
                r
            };
        });
        Rc::new(var_audit_database_context)
    }
}
