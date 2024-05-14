//! This modules contains [`execute_critical_section`].

use veritech_client::{BeforeFunction, ResolverFunctionComponent};

use crate::func::backend::array::FuncBackendArray;
use crate::func::backend::boolean::FuncBackendBoolean;
use crate::func::backend::diff::FuncBackendDiff;
use crate::func::backend::identity::FuncBackendIdentity;
use crate::func::backend::integer::FuncBackendInteger;
use crate::func::backend::js_action::FuncBackendJsAction;
use crate::func::backend::js_attribute::{FuncBackendJsAttribute, FuncBackendJsAttributeArgs};
use crate::func::backend::js_reconciliation::FuncBackendJsReconciliation;
use crate::func::backend::js_schema_variant_definition::FuncBackendJsSchemaVariantDefinition;
use crate::func::backend::json::FuncBackendJson;
use crate::func::backend::map::FuncBackendMap;
use crate::func::backend::object::FuncBackendObject;
use crate::func::backend::string::FuncBackendString;
use crate::func::backend::validation::FuncBackendValidation;
use crate::func::backend::{FuncBackend, FuncBackendError, FuncDispatch, FuncDispatchContext};
use crate::func::binding::{FuncBindingError, FuncBindingResult};
use crate::{Func, FuncBackendKind};

/// Perform function execution to veritech for a given [`Func`] and [`FuncDispatchContext`].
///
/// This function does not persist data to "layerdb". If data persistence for execution is required,
/// use [`FuncBinding::create_and_execute`](crate::FuncBinding::create_and_execute).
///
/// _Warning:_ this function should be used with caution!
pub(crate) async fn execute_critical_section(
    func: Func,
    args: &serde_json::Value,
    context: FuncDispatchContext,
    before: Vec<BeforeFunction>,
) -> FuncBindingResult<(Option<serde_json::Value>, Option<serde_json::Value>)> {
    let execution_result = match func.backend_kind {
        FuncBackendKind::JsAction => {
            FuncBackendJsAction::create_and_execute(context, &func, args, before).await
        }
        FuncBackendKind::JsReconciliation => {
            FuncBackendJsReconciliation::create_and_execute(context, &func, args, before).await
        }
        FuncBackendKind::JsAttribute => {
            let args = FuncBackendJsAttributeArgs {
                component: ResolverFunctionComponent {
                    data: veritech_client::ComponentView {
                        properties: args.to_owned(),
                        ..Default::default()
                    },
                    parents: Vec::new(),
                },
                response_type: func.backend_response_type.try_into()?,
            };
            FuncBackendJsAttribute::create_and_execute(
                context,
                &func,
                &serde_json::to_value(args)?,
                before,
            )
            .await
        }
        FuncBackendKind::JsSchemaVariantDefinition => {
            FuncBackendJsSchemaVariantDefinition::create_and_execute(
                context,
                &func,
                &serde_json::Value::Null,
                before,
            )
            .await
        }
        FuncBackendKind::Json => FuncBackendJson::create_and_execute(args).await,
        FuncBackendKind::Array => FuncBackendArray::create_and_execute(args).await,
        FuncBackendKind::Boolean => FuncBackendBoolean::create_and_execute(args).await,
        FuncBackendKind::Identity => FuncBackendIdentity::create_and_execute(args).await,
        FuncBackendKind::Diff => FuncBackendDiff::create_and_execute(args).await,
        FuncBackendKind::Integer => FuncBackendInteger::create_and_execute(args).await,
        FuncBackendKind::Map => FuncBackendMap::create_and_execute(args).await,
        FuncBackendKind::Object => FuncBackendObject::create_and_execute(args).await,
        FuncBackendKind::String => FuncBackendString::create_and_execute(args).await,
        FuncBackendKind::Unset => Ok((None, None)),
        FuncBackendKind::Validation => {
            FuncBackendValidation::create_and_execute(context, &func, args, before).await
        }
        FuncBackendKind::JsValidation => {
            unimplemented!("direct Validation function execution is deprecated")
        }
        FuncBackendKind::JsAuthentication => {
            unimplemented!("direct JsAuthentication function execution is not currently supported")
        }
    };

    match execution_result {
        Ok(value) => Ok(value),
        Err(FuncBackendError::ResultFailure {
            kind,
            message,
            backend,
        }) => Err(FuncBindingError::FuncBackendResultFailure {
            kind,
            message,
            backend,
        }),
        Err(err) => Err(err)?,
    }
}
