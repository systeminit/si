use si_data::{NatsTxn, PgTxn};

use crate::{
    Func, FuncBackendKind, FuncBackendResponseType, FuncResult, HistoryActor, StandardModel,
    Tenancy, Visibility,
};

pub async fn migrate(txn: &PgTxn<'_>, nats: &NatsTxn) -> FuncResult<()> {
    let (tenancy, visibility, history_actor) = (
        Tenancy::new_universal(),
        Visibility::new_head(false),
        HistoryActor::SystemInit,
    );

    si_set_string(txn, nats, &tenancy, &visibility, &history_actor).await?;
    si_set_integer(txn, nats, &tenancy, &visibility, &history_actor).await?;
    si_set_prop_object(txn, nats, &tenancy, &visibility, &history_actor).await?;
    si_unset(txn, nats, &tenancy, &visibility, &history_actor).await?;
    si_validate_string_equals(txn, nats, &tenancy, &visibility, &history_actor).await?;
    si_qualification_always_true(txn, nats, &tenancy, &visibility, &history_actor).await?;
    si_totally_random_string(txn, nats, &tenancy, &visibility, &history_actor).await?;
    si_qualification_docker_image_name_equals_component_name(
        txn,
        nats,
        &tenancy,
        &visibility,
        &history_actor,
    )
    .await?;
    si_resource_sync_hammer(txn, nats, &tenancy, &visibility, &history_actor).await?;

    Ok(())
}

async fn si_totally_random_string(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> FuncResult<()> {
    let existing_func = Func::find_by_attr(
        txn,
        tenancy,
        visibility,
        "name",
        &"si:totallyRandomString".to_string(),
    )
    .await?;
    if existing_func.is_empty() {
        let mut func = Func::new(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            "si:totallyRandomString",
            FuncBackendKind::JsString,
            FuncBackendResponseType::String,
        )
        .await
        .expect("cannot create func");
        func.set_handler(
            txn,
            nats,
            visibility,
            history_actor,
            Some("totallyRandomString"),
        )
        .await?;
        func.set_code_base64(
            txn,
            nats,
            visibility,
            history_actor,
            Some(base64::encode(
                "function totallyRandomString() { return \"4\"; }",
            )),
        )
        .await?;
    }
    Ok(())
}

async fn si_set_string(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> FuncResult<()> {
    let existing_func = Func::find_by_attr(
        txn,
        tenancy,
        visibility,
        "name",
        &"si:setString".to_string(),
    )
    .await?;
    if existing_func.is_empty() {
        Func::new(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            "si:setString",
            FuncBackendKind::String,
            FuncBackendResponseType::String,
        )
        .await
        .expect("cannot create func");
    }
    Ok(())
}

async fn si_set_integer(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> FuncResult<()> {
    let existing_func = Func::find_by_attr(
        txn,
        tenancy,
        visibility,
        "name",
        &"si:setInteger".to_string(),
    )
    .await?;
    if existing_func.is_empty() {
        Func::new(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            "si:setInteger",
            FuncBackendKind::Integer,
            FuncBackendResponseType::Integer,
        )
        .await
        .expect("cannot create func");
    }

    Ok(())
}

async fn si_set_prop_object(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> FuncResult<()> {
    let existing_func = Func::find_by_attr(
        txn,
        tenancy,
        visibility,
        "name",
        &"si:setPropObject".to_string(),
    )
    .await?;
    if existing_func.is_empty() {
        Func::new(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            "si:setPropObject",
            FuncBackendKind::PropObject,
            FuncBackendResponseType::PropObject,
        )
        .await
        .expect("cannot create func");
    }

    Ok(())
}

async fn si_unset(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> FuncResult<()> {
    let existing_func =
        Func::find_by_attr(txn, tenancy, visibility, "name", &"si:unset".to_string()).await?;
    if existing_func.is_empty() {
        Func::new(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            "si:unset",
            FuncBackendKind::Unset,
            FuncBackendResponseType::Unset,
        )
        .await
        .expect("cannot create func");
    }
    Ok(())
}

async fn si_validate_string_equals(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> FuncResult<()> {
    let func_name = "si:validateStringEquals".to_string();
    let existing_func = Func::find_by_attr(txn, tenancy, visibility, "name", &func_name).await?;
    if existing_func.is_empty() {
        Func::new(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            &func_name,
            FuncBackendKind::ValidateStringValue,
            FuncBackendResponseType::Validation,
        )
        .await
        .expect("cannot create func");
    }

    Ok(())
}

async fn si_qualification_always_true(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> FuncResult<()> {
    let func_name = "si:qualificationAlwaysTrue".to_string();
    let existing_func = Func::find_by_attr(txn, tenancy, visibility, "name", &func_name).await?;
    if existing_func.is_empty() {
        let mut new_func = Func::new(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            &func_name,
            FuncBackendKind::JsQualification,
            FuncBackendResponseType::Qualification,
        )
        .await
        .expect("cannot create func");

        let qualification_code =
            base64::encode("function alwaysGood(_ignored) { return { qualified: true }; }");

        new_func
            .set_handler(
                txn,
                nats,
                visibility,
                history_actor,
                Some("alwaysGood".to_string()),
            )
            .await
            .expect("cannot set handler");
        new_func
            .set_code_base64(
                txn,
                nats,
                visibility,
                history_actor,
                Some(qualification_code),
            )
            .await
            .expect("cannot set code");
    }

    Ok(())
}

async fn si_resource_sync_hammer(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> FuncResult<()> {
    let func_name = "si:resourceSyncHammer".to_string();
    let existing_func = Func::find_by_attr(txn, tenancy, visibility, "name", &func_name).await?;
    if existing_func.is_empty() {
        let mut new_func = Func::new(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            &func_name,
            FuncBackendKind::JsResourceSync,
            FuncBackendResponseType::ResourceSync,
        )
        .await
        .expect("cannot create func");

        let qualification_code = base64::encode(include_str!("./builtins/resourceSyncHammer.js",));

        new_func
            .set_handler(
                txn,
                nats,
                visibility,
                history_actor,
                Some("resourceSyncHammer".to_string()),
            )
            .await
            .expect("cannot set handler");
        new_func
            .set_code_base64(
                txn,
                nats,
                visibility,
                history_actor,
                Some(qualification_code),
            )
            .await
            .expect("cannot set code");
    }

    Ok(())
}

async fn si_qualification_docker_image_name_equals_component_name(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> FuncResult<()> {
    let func_name = "si:qualificationDockerImageNameEqualsComponentName".to_string();
    let existing_func = Func::find_by_attr(txn, tenancy, visibility, "name", &func_name).await?;
    if existing_func.is_empty() {
        let mut new_func = Func::new(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            &func_name,
            FuncBackendKind::JsQualification,
            FuncBackendResponseType::Qualification,
        )
        .await
        .expect("cannot create func");

        let qualification_code = base64::encode(include_str!(
            "./builtins/qualificationDockerImageNameEqualsComponentName.js"
        ));

        new_func
            .set_handler(
                txn,
                nats,
                visibility,
                history_actor,
                Some("dockerImageNameEqualsComponentName".to_string()),
            )
            .await
            .expect("cannot set handler");
        new_func
            .set_code_base64(
                txn,
                nats,
                visibility,
                history_actor,
                Some(qualification_code),
            )
            .await
            .expect("cannot set code");
    }

    Ok(())
}
