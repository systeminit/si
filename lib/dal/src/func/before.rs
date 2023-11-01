use serde::Deserialize;
use veritech_client::BeforeFunctionRequest;

use crate::{
    standard_model, ComponentId, DalContext, EncryptedSecret, Func, FuncError, FuncResult,
};

const AUTH_FUNCS_FOR_COMPONENT: &str =
    include_str!("../queries/func/authentication_funcs_for_component.sql");

#[derive(Deserialize, Debug)]
struct EncryptedSecretAndFunc {
    encrypted_secret: EncryptedSecret,
    func: Func,
}

pub async fn before_funcs_for_component(
    ctx: &DalContext,
    component_id: &ComponentId,
) -> FuncResult<Vec<BeforeFunctionRequest>> {
    println!("before_funcs_for_component");

    let rows = ctx
        .txns()
        .await?
        .pg()
        .query(
            AUTH_FUNCS_FOR_COMPONENT,
            &[ctx.tenancy(), ctx.visibility(), component_id],
        )
        .await?;

    let mut results = vec![];

    for EncryptedSecretAndFunc {
        encrypted_secret,
        func,
    } in standard_model::objects_from_rows::<EncryptedSecretAndFunc>(rows)?
    {
        results.push(BeforeFunctionRequest {
            execution_id: "rick_allen".to_string(),
            handler: func
                .handler
                .ok_or_else(|| FuncError::MissingHandler(func.id))?,
            code_base64: func
                .code_base64
                .ok_or_else(|| FuncError::MissingCode(func.id))?,
            arg: encrypted_secret.decrypt(ctx).await?.message().into_inner(),
        })
    }

    dbg!(&results);

    Ok(results)
}
