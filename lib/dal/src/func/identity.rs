//! This module contains a special helper for finding the identity [`Func`](crate::Func). Generally
//! speaking, [`Funcs`](crate::Func) should not receive special treatment, but due to the
//! prevalence of the identity [`Func`](crate::Func) across the library, this helper should help
//! ease some friction.

use crate::{DalContext, Func, FuncArgument, FuncBinding, FuncBindingReturnValue, StandardModel};

const IDENTITY_FUNC_NAME: &str = "si:identity";

// impl Func {
//     /// Returns the identity [`Func`](Self) with its corresponding
//     /// [`FuncBinding`](crate::FuncBinding) and
//     /// [`FuncBindingReturnValue`](crate::FuncBindingReturnValue).
//     pub async fn identity_with_binding_and_return_value(
//         ctx: &DalContext,
//     ) -> FuncResult<(Func, FuncBinding, FuncBindingReturnValue)> {
//         let func = Self::identity_func(ctx).await?;
//         let (func_binding, func_binding_return_value) = FuncBinding::create_and_execute(
//             ctx,
//             serde_json::json![{ "identity": null }],
//             *func.id(),
//         )
//         .await
//         .map_err(|e| FuncError::FuncBinding(e.to_string()))?;

//         Ok((func, func_binding, func_binding_return_value))
//     }

//     /// Returns the identity [`Func`](Self) with its corresponding
//     /// [`FuncArgument`](crate::FuncArgument).
//     pub async fn identity_with_argument(ctx: &DalContext) -> FuncResult<(Func, FuncArgument)> {
//         let func = Self::identity_func(ctx).await?;
//         let func_argument = FuncArgument::find_by_name_for_func(ctx, "identity", *func.id())
//             .await?
//             .ok_or(FuncError::IdentityFuncArgumentNotFound)?;
//         Ok((func, func_argument))
//     }

//     /// Returns the identity [`Func`](Self).
//     pub async fn identity_func(ctx: &DalContext) -> FuncResult<Func> {
//         let mut found_funcs = Func::find_by_attr(ctx, "name", &IDENTITY_FUNC_NAME).await?;
//         let func = found_funcs.pop().ok_or(FuncError::IdentityFuncNotFound)?;
//         match found_funcs.is_empty() {
//             true => Ok(func),
//             false => Err(FuncError::TooManyFuncsFoundForIdentity),
//         }
//     }
// }
