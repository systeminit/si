use si_data::{NatsTxn, PgTxn};

use crate::{
    Func, FuncBackendKind, FuncBackendResponseType, FuncResult, HistoryActor, StandardModel,
    Visibility, WriteTenancy,
};

pub async fn migrate(txn: &PgTxn<'_>, nats: &NatsTxn) -> FuncResult<()> {
    let (tenancy, visibility, history_actor) = (
        WriteTenancy::new_universal(),
        Visibility::new_head(false),
        HistoryActor::SystemInit,
    );

    si_set_array(txn, nats, &tenancy, &visibility, &history_actor).await?;
    si_set_boolean(txn, nats, &tenancy, &visibility, &history_actor).await?;
    si_set_integer(txn, nats, &tenancy, &visibility, &history_actor).await?;
    si_set_map(txn, nats, &tenancy, &visibility, &history_actor).await?;
    si_set_prop_object(txn, nats, &tenancy, &visibility, &history_actor).await?;
    si_set_string(txn, nats, &tenancy, &visibility, &history_actor).await?;
    si_unset(txn, nats, &tenancy, &visibility, &history_actor).await?;

    si_validate_string_equals(txn, nats, &tenancy, &visibility, &history_actor).await?;
    si_qualification_always_true(txn, nats, &tenancy, &visibility, &history_actor).await?;
    si_number_of_parents(txn, nats, &tenancy, &visibility, &history_actor).await?;
    si_generate_yaml(txn, nats, &tenancy, &visibility, &history_actor).await?;
    si_qualification_docker_image_name_inspect(txn, nats, &tenancy, &visibility, &history_actor)
        .await?;
    si_resource_sync_hammer(txn, nats, &tenancy, &visibility, &history_actor).await?;
    si_qualification_yaml_kubeval(txn, nats, &tenancy, &visibility, &history_actor).await?;
    si_qualification_docker_hub_login(txn, nats, &tenancy, &visibility, &history_actor).await?;

    Ok(())
}

async fn si_generate_yaml(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    write_tenancy: &WriteTenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> FuncResult<()> {
    let existing_func = Func::find_by_attr(
        txn,
        &write_tenancy.into(),
        visibility,
        "name",
        &"si:generateYAML".to_string(),
    )
    .await?;
    if existing_func.is_empty() {
        let mut func = Func::new(
            txn,
            nats,
            write_tenancy,
            visibility,
            history_actor,
            "si:generateYAML",
            FuncBackendKind::JsCodeGeneration,
            FuncBackendResponseType::CodeGeneration,
        )
        .await
        .expect("cannot create func");
        func.set_handler(txn, nats, visibility, history_actor, Some("generateYAML"))
            .await?;

        let code = base64::encode(include_str!("./builtins/generateYAML.js",));
        func.set_code_base64(txn, nats, visibility, history_actor, Some(code))
            .await?;
    }
    Ok(())
}

async fn si_number_of_parents(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    write_tenancy: &WriteTenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> FuncResult<()> {
    let existing_func = Func::find_by_attr(
        txn,
        &write_tenancy.into(),
        visibility,
        "name",
        &"si:numberOfParents".to_string(),
    )
    .await?;
    if existing_func.is_empty() {
        let mut func = Func::new(
            txn,
            nats,
            write_tenancy,
            visibility,
            history_actor,
            "si:numberOfParents",
            FuncBackendKind::JsAttribute, // Should be integer, but the js integer backend isn't 100% there yet and is being worked on in parallel
            FuncBackendResponseType::String,
        )
        .await
        .expect("cannot create func");
        func.set_handler(
            txn,
            nats,
            visibility,
            history_actor,
            Some("numberOfParents"),
        )
        .await?;
        func.set_code_base64(
            txn,
            nats,
            visibility,
            history_actor,
            Some(base64::encode(
                "function numberOfParents(component) { return `${component.parents.length}`; }",
            )),
        )
        .await?;
    }
    Ok(())
}

async fn si_set_array(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    write_tenancy: &WriteTenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> FuncResult<()> {
    let existing_func = Func::find_by_attr(
        txn,
        &write_tenancy.into(),
        visibility,
        "name",
        &"si:setArray".to_string(),
    )
    .await?;
    if existing_func.is_empty() {
        Func::new(
            txn,
            nats,
            write_tenancy,
            visibility,
            history_actor,
            "si:setArray",
            FuncBackendKind::Array,
            FuncBackendResponseType::Array,
        )
        .await
        .expect("cannot create func");
    }
    Ok(())
}

async fn si_set_boolean(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    write_tenancy: &WriteTenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> FuncResult<()> {
    let existing_func = Func::find_by_attr(
        txn,
        &write_tenancy.into(),
        visibility,
        "name",
        &"si:setBoolean".to_string(),
    )
    .await?;
    if existing_func.is_empty() {
        Func::new(
            txn,
            nats,
            write_tenancy,
            visibility,
            history_actor,
            "si:setBoolean",
            FuncBackendKind::Boolean,
            FuncBackendResponseType::Boolean,
        )
        .await
        .expect("cannot create func");
    }
    Ok(())
}

async fn si_set_integer(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    write_tenancy: &WriteTenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> FuncResult<()> {
    let existing_func = Func::find_by_attr(
        txn,
        &write_tenancy.into(),
        visibility,
        "name",
        &"si:setInteger".to_string(),
    )
    .await?;
    if existing_func.is_empty() {
        Func::new(
            txn,
            nats,
            write_tenancy,
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
    write_tenancy: &WriteTenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> FuncResult<()> {
    let existing_func = Func::find_by_attr(
        txn,
        &write_tenancy.into(),
        visibility,
        "name",
        &"si:setPropObject".to_string(),
    )
    .await?;
    if existing_func.is_empty() {
        Func::new(
            txn,
            nats,
            write_tenancy,
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

async fn si_set_map(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    write_tenancy: &WriteTenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> FuncResult<()> {
    let existing_func = Func::find_by_attr(
        txn,
        &write_tenancy.into(),
        visibility,
        "name",
        &"si:setMap".to_string(),
    )
    .await?;
    if existing_func.is_empty() {
        Func::new(
            txn,
            nats,
            write_tenancy,
            visibility,
            history_actor,
            "si:setMap",
            FuncBackendKind::Map,
            FuncBackendResponseType::Map,
        )
        .await
        .expect("cannot create func");
    }

    Ok(())
}

async fn si_set_string(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    write_tenancy: &WriteTenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> FuncResult<()> {
    let existing_func = Func::find_by_attr(
        txn,
        &write_tenancy.into(),
        visibility,
        "name",
        &"si:setString".to_string(),
    )
    .await?;
    if existing_func.is_empty() {
        Func::new(
            txn,
            nats,
            write_tenancy,
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

async fn si_unset(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    write_tenancy: &WriteTenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> FuncResult<()> {
    let existing_func = Func::find_by_attr(
        txn,
        &write_tenancy.into(),
        visibility,
        "name",
        &"si:unset".to_string(),
    )
    .await?;
    if existing_func.is_empty() {
        Func::new(
            txn,
            nats,
            write_tenancy,
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
    write_tenancy: &WriteTenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> FuncResult<()> {
    let func_name = "si:validateStringEquals".to_string();
    let existing_func =
        Func::find_by_attr(txn, &write_tenancy.into(), visibility, "name", &func_name).await?;
    if existing_func.is_empty() {
        Func::new(
            txn,
            nats,
            write_tenancy,
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
    write_tenancy: &WriteTenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> FuncResult<()> {
    let func_name = "si:qualificationAlwaysTrue".to_string();
    let existing_func =
        Func::find_by_attr(txn, &write_tenancy.into(), visibility, "name", &func_name).await?;
    if existing_func.is_empty() {
        let mut new_func = Func::new(
            txn,
            nats,
            write_tenancy,
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
    write_tenancy: &WriteTenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> FuncResult<()> {
    let func_name = "si:resourceSyncHammer".to_string();
    let existing_func =
        Func::find_by_attr(txn, &write_tenancy.into(), visibility, "name", &func_name).await?;
    if existing_func.is_empty() {
        let mut new_func = Func::new(
            txn,
            nats,
            write_tenancy,
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

async fn si_qualification_docker_image_name_inspect(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    write_tenancy: &WriteTenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> FuncResult<()> {
    let func_name = "si:qualificationDockerImageNameInspect".to_string();
    let existing_func =
        Func::find_by_attr(txn, &write_tenancy.into(), visibility, "name", &func_name).await?;
    if existing_func.is_empty() {
        let mut new_func = Func::new(
            txn,
            nats,
            write_tenancy,
            visibility,
            history_actor,
            &func_name,
            FuncBackendKind::JsQualification,
            FuncBackendResponseType::Qualification,
        )
        .await
        .expect("cannot create func");

        let qualification_code = base64::encode(include_str!(
            "./builtins/qualificationDockerImageNameInspect.js"
        ));

        new_func
            .set_handler(
                txn,
                nats,
                visibility,
                history_actor,
                Some("qualificationDockerImageNameInspect".to_string()),
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

async fn si_qualification_yaml_kubeval(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    write_tenancy: &WriteTenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> FuncResult<()> {
    let func_name = "si:qualificationYamlKubeval".to_string();
    let existing_func =
        Func::find_by_attr(txn, &write_tenancy.into(), visibility, "name", &func_name).await?;
    if existing_func.is_empty() {
        let mut new_func = Func::new(
            txn,
            nats,
            write_tenancy,
            visibility,
            history_actor,
            &func_name,
            FuncBackendKind::JsQualification,
            FuncBackendResponseType::Qualification,
        )
        .await
        .expect("cannot create func");

        let qualification_code =
            base64::encode(include_str!("./builtins/qualificationYamlKubeval.js"));

        new_func
            .set_handler(
                txn,
                nats,
                visibility,
                history_actor,
                Some("qualificationYamlKubeval".to_string()),
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
async fn si_qualification_docker_hub_login(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    write_tenancy: &WriteTenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> FuncResult<()> {
    let func_name = "si:qualificationDockerHubLogin".to_string();
    let existing_func =
        Func::find_by_attr(txn, &write_tenancy.into(), visibility, "name", &func_name).await?;
    if existing_func.is_empty() {
        let mut new_func = Func::new(
            txn,
            nats,
            write_tenancy,
            visibility,
            history_actor,
            &func_name,
            FuncBackendKind::JsQualification,
            FuncBackendResponseType::Qualification,
        )
        .await
        .expect("cannot create func");

        let qualification_code =
            base64::encode(include_str!("./builtins/qualificationDockerHubLogin.js"));

        new_func
            .set_handler(
                txn,
                nats,
                visibility,
                history_actor,
                Some("qualificationDockerHubLogin".to_string()),
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
