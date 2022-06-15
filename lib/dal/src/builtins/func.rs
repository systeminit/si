use crate::{
    BuiltinsResult, DalContext, Func, FuncBackendKind, FuncBackendResponseType, StandardModel,
};

pub async fn migrate(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    si_set_array(ctx).await?;
    si_set_boolean(ctx).await?;
    si_set_integer(ctx).await?;
    si_set_map(ctx).await?;
    si_set_prop_object(ctx).await?;
    si_set_string(ctx).await?;

    si_identity(ctx).await?;
    si_unset(ctx).await?;

    si_validate_string_equals(ctx).await?;
    si_qualification_always_true(ctx).await?;
    si_docker_images_to_k8s_deployment_container_spec(ctx).await?;
    si_generate_yaml(ctx).await?;
    si_qualification_docker_image_name_inspect(ctx).await?;
    si_resource_sync_hammer(ctx).await?;
    si_qualification_yaml_kubeval(ctx).await?;
    si_qualification_docker_hub_login(ctx).await?;

    Ok(())
}

async fn si_generate_yaml(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let existing_func = Func::find_by_attr(ctx, "name", &"si:generateYAML".to_string()).await?;
    if existing_func.is_empty() {
        let mut func = Func::new(
            ctx,
            "si:generateYAML",
            FuncBackendKind::JsCodeGeneration,
            FuncBackendResponseType::CodeGeneration,
        )
        .await
        .expect("cannot create func");
        func.set_handler(ctx, Some("generateYAML")).await?;

        let code = base64::encode(include_str!("./func/generateYAML.js",));
        func.set_code_base64(ctx, Some(code)).await?;
    }
    Ok(())
}

async fn si_docker_images_to_k8s_deployment_container_spec(
    ctx: &DalContext<'_, '_>,
) -> BuiltinsResult<()> {
    let func_name = "si:dockerImagesToK8sDeploymentContainerSpec".to_string();
    let existing_func = Func::find_by_attr(ctx, "name", &func_name).await?;
    if existing_func.is_empty() {
        let mut func = Func::new(
            ctx,
            func_name,
            FuncBackendKind::Json,
            FuncBackendResponseType::Array,
        )
        .await
        .expect("cannot create func");
        func.set_handler(ctx, Some("dockerImagesToK8sDeploymentContainerSpec"))
            .await?;
        func.set_code_base64(
            ctx,
            Some(base64::encode(include_str!(
                "./func/dockerImagesToK8sDeploymentContainerSpec.js"
            ))),
        )
        .await?;
    }

    Ok(())
}

async fn si_set_array(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let existing_func = Func::find_by_attr(ctx, "name", &"si:setArray".to_string()).await?;
    if existing_func.is_empty() {
        Func::new(
            ctx,
            "si:setArray",
            FuncBackendKind::Array,
            FuncBackendResponseType::Array,
        )
        .await
        .expect("cannot create func");
    }
    Ok(())
}

async fn si_set_boolean(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let existing_func = Func::find_by_attr(ctx, "name", &"si:setBoolean".to_string()).await?;
    if existing_func.is_empty() {
        Func::new(
            ctx,
            "si:setBoolean",
            FuncBackendKind::Boolean,
            FuncBackendResponseType::Boolean,
        )
        .await
        .expect("cannot create func");
    }
    Ok(())
}

async fn si_set_integer(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let existing_func = Func::find_by_attr(ctx, "name", &"si:setInteger".to_string()).await?;
    if existing_func.is_empty() {
        Func::new(
            ctx,
            "si:setInteger",
            FuncBackendKind::Integer,
            FuncBackendResponseType::Integer,
        )
        .await
        .expect("cannot create func");
    }

    Ok(())
}

async fn si_set_prop_object(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let existing_func = Func::find_by_attr(ctx, "name", &"si:setPropObject".to_string()).await?;
    if existing_func.is_empty() {
        Func::new(
            ctx,
            "si:setPropObject",
            FuncBackendKind::PropObject,
            FuncBackendResponseType::PropObject,
        )
        .await
        .expect("cannot create func");
    }

    Ok(())
}

async fn si_set_map(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let existing_func = Func::find_by_attr(ctx, "name", &"si:setMap".to_string()).await?;
    if existing_func.is_empty() {
        Func::new(
            ctx,
            "si:setMap",
            FuncBackendKind::Map,
            FuncBackendResponseType::Map,
        )
        .await
        .expect("cannot create func");
    }

    Ok(())
}

async fn si_set_string(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let existing_func = Func::find_by_attr(ctx, "name", &"si:setString".to_string()).await?;
    if existing_func.is_empty() {
        Func::new(
            ctx,
            "si:setString",
            FuncBackendKind::String,
            FuncBackendResponseType::String,
        )
        .await
        .expect("cannot create func");
    }
    Ok(())
}

async fn si_identity(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let existing_func = Func::find_by_attr(ctx, "name", &"si:identity".to_string()).await?;
    if existing_func.is_empty() {
        Func::new(
            ctx,
            "si:identity",
            FuncBackendKind::Identity,
            FuncBackendResponseType::Identity,
        )
        .await
        .expect("cannot create func");
    }
    Ok(())
}

async fn si_unset(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let existing_func = Func::find_by_attr(ctx, "name", &"si:unset".to_string()).await?;
    if existing_func.is_empty() {
        Func::new(
            ctx,
            "si:unset",
            FuncBackendKind::Unset,
            FuncBackendResponseType::Unset,
        )
        .await
        .expect("cannot create func");
    }
    Ok(())
}

async fn si_validate_string_equals(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let func_name = "si:validateStringEquals".to_string();
    let existing_func = Func::find_by_attr(ctx, "name", &func_name).await?;
    if existing_func.is_empty() {
        Func::new(
            ctx,
            &func_name,
            FuncBackendKind::ValidateStringValue,
            FuncBackendResponseType::Validation,
        )
        .await
        .expect("cannot create func");
    }

    Ok(())
}

async fn si_qualification_always_true(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let func_name = "si:qualificationAlwaysTrue".to_string();
    let existing_func = Func::find_by_attr(ctx, "name", &func_name).await?;
    if existing_func.is_empty() {
        let mut new_func = Func::new(
            ctx,
            &func_name,
            FuncBackendKind::JsQualification,
            FuncBackendResponseType::Qualification,
        )
        .await
        .expect("cannot create func");

        let qualification_code =
            base64::encode("function alwaysGood(_ignored) { return { qualified: true }; }");

        new_func
            .set_handler(ctx, Some("alwaysGood".to_string()))
            .await
            .expect("cannot set handler");
        new_func
            .set_code_base64(ctx, Some(qualification_code))
            .await
            .expect("cannot set code");
    }

    Ok(())
}

async fn si_resource_sync_hammer(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let func_name = "si:resourceSyncHammer".to_string();
    let existing_func = Func::find_by_attr(ctx, "name", &func_name).await?;
    if existing_func.is_empty() {
        let mut new_func = Func::new(
            ctx,
            &func_name,
            FuncBackendKind::JsResourceSync,
            FuncBackendResponseType::ResourceSync,
        )
        .await
        .expect("cannot create func");

        let qualification_code = base64::encode(include_str!("./func/resourceSyncHammer.js",));

        new_func
            .set_handler(ctx, Some("resourceSyncHammer".to_string()))
            .await
            .expect("cannot set handler");
        new_func
            .set_code_base64(ctx, Some(qualification_code))
            .await
            .expect("cannot set code");
    }

    Ok(())
}

async fn si_qualification_docker_image_name_inspect(
    ctx: &DalContext<'_, '_>,
) -> BuiltinsResult<()> {
    let func_name = "si:qualificationDockerImageNameInspect".to_string();
    let existing_func = Func::find_by_attr(ctx, "name", &func_name).await?;
    if existing_func.is_empty() {
        let mut new_func = Func::new(
            ctx,
            &func_name,
            FuncBackendKind::JsQualification,
            FuncBackendResponseType::Qualification,
        )
        .await
        .expect("cannot create func");

        let qualification_code = base64::encode(include_str!(
            "./func/qualificationDockerImageNameInspect.js"
        ));

        new_func
            .set_handler(ctx, Some("qualificationDockerImageNameInspect".to_string()))
            .await
            .expect("cannot set handler");
        new_func
            .set_code_base64(ctx, Some(qualification_code))
            .await
            .expect("cannot set code");
    }

    Ok(())
}

async fn si_qualification_yaml_kubeval(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let func_name = "si:qualificationYamlKubeval".to_string();
    let existing_func = Func::find_by_attr(ctx, "name", &func_name).await?;
    if existing_func.is_empty() {
        let mut new_func = Func::new(
            ctx,
            &func_name,
            FuncBackendKind::JsQualification,
            FuncBackendResponseType::Qualification,
        )
        .await
        .expect("cannot create func");

        let qualification_code = base64::encode(include_str!("./func/qualificationYamlKubeval.js"));

        new_func
            .set_handler(ctx, Some("qualificationYamlKubeval".to_string()))
            .await
            .expect("cannot set handler");
        new_func
            .set_code_base64(ctx, Some(qualification_code))
            .await
            .expect("cannot set code");
    }

    Ok(())
}

async fn si_qualification_docker_hub_login(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let func_name = "si:qualificationDockerHubLogin".to_string();
    let existing_func = Func::find_by_attr(ctx, "name", &func_name).await?;
    if existing_func.is_empty() {
        let mut new_func = Func::new(
            ctx,
            &func_name,
            FuncBackendKind::JsQualification,
            FuncBackendResponseType::Qualification,
        )
        .await
        .expect("cannot create func");

        let qualification_code =
            base64::encode(include_str!("./func/qualificationDockerHubLogin.js"));

        new_func
            .set_handler(ctx, Some("qualificationDockerHubLogin".to_string()))
            .await
            .expect("cannot set handler");
        new_func
            .set_code_base64(ctx, Some(qualification_code))
            .await
            .expect("cannot set code");
    }

    Ok(())
}
