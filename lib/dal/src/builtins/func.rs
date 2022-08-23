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

    si_poem_workflow(ctx).await?;
    si_exceptional_workflow(ctx).await?;
    si_finalizing_workflow(ctx).await?;

    si_title_command(ctx).await?;
    si_title2_command(ctx).await?;
    si_first_stanza_command(ctx).await?;
    si_second_stanza_command(ctx).await?;
    si_third_stanza_command(ctx).await?;
    si_fourth_stanza_command(ctx).await?;
    si_fifth_stanza_command(ctx).await?;
    si_sixth_stanza_command(ctx).await?;
    si_seventh_stanza_command(ctx).await?;
    si_question_command(ctx).await?;
    si_bye_command(ctx).await?;

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
    let func_name = "Always True".to_string();
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
    let func_name = "Inspect Docker image name".to_string();
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

        new_func
            .set_description(ctx, "Verifies that the docker image exists".into())
            .await
            .expect("Set func description");
        new_func
            .set_link(ctx, "http://docker.com".into())
            .await
            .expect("set func link");
    }

    Ok(())
}

async fn si_qualification_yaml_kubeval(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let func_name = "Run kubeval on YAML".to_string();
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

        new_func
            .set_description(ctx, Some("Runs kubeval on the generated YAML".to_string()))
            .await
            .expect("Set func description");
    }

    Ok(())
}

async fn si_qualification_docker_hub_login(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let func_name = "Docker Hub login".to_string();
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

        new_func
            .set_description(ctx, "Ensures docker hub login credentials work".into())
            .await
            .expect("set func description");
        new_func
            .set_link(ctx, "http://hub.docker.com".into())
            .await
            .expect("set func link");
    }

    Ok(())
}

async fn si_poem_workflow(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let func_name = "si:poem".to_string();
    let existing_func = Func::find_by_attr(ctx, "name", &func_name).await?;
    if existing_func.is_empty() {
        let mut new_func = Func::new(
            ctx,
            &func_name,
            FuncBackendKind::JsWorkflow,
            FuncBackendResponseType::Workflow,
        )
        .await
        .expect("cannot create func");

        let workflow_code = base64::encode(include_str!("./func/poemWorkflow.js"));

        new_func
            .set_handler(ctx, Some("poem".to_string()))
            .await
            .expect("cannot set handler");
        new_func
            .set_code_base64(ctx, Some(workflow_code))
            .await
            .expect("cannot set code");
    }

    Ok(())
}

async fn si_exceptional_workflow(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let func_name = "si:exceptional".to_string();
    let existing_func = Func::find_by_attr(ctx, "name", &func_name).await?;
    if existing_func.is_empty() {
        let mut new_func = Func::new(
            ctx,
            &func_name,
            FuncBackendKind::JsWorkflow,
            FuncBackendResponseType::Workflow,
        )
        .await
        .expect("cannot create func");

        let workflow_code = base64::encode(include_str!("./func/exceptionalWorkflow.js"));

        new_func
            .set_handler(ctx, Some("exceptional".to_string()))
            .await
            .expect("cannot set handler");
        new_func
            .set_code_base64(ctx, Some(workflow_code))
            .await
            .expect("cannot set code");
    }

    Ok(())
}

async fn si_finalizing_workflow(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let func_name = "si:finalizing".to_string();
    let existing_func = Func::find_by_attr(ctx, "name", &func_name).await?;
    if existing_func.is_empty() {
        let mut new_func = Func::new(
            ctx,
            &func_name,
            FuncBackendKind::JsWorkflow,
            FuncBackendResponseType::Workflow,
        )
        .await
        .expect("cannot create func");

        let workflow_code = base64::encode(include_str!("./func/finalizingWorkflow.js"));

        new_func
            .set_handler(ctx, Some("finalizing".to_string()))
            .await
            .expect("cannot set handler");
        new_func
            .set_code_base64(ctx, Some(workflow_code))
            .await
            .expect("cannot set code");
    }

    Ok(())
}

async fn si_title_command(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let func_name = "si:title".to_string();
    let existing_func = Func::find_by_attr(ctx, "name", &func_name).await?;
    if existing_func.is_empty() {
        let mut new_func = Func::new(
            ctx,
            &func_name,
            FuncBackendKind::JsCommand,
            FuncBackendResponseType::Command,
        )
        .await
        .expect("cannot create func");

        let workflow_code = base64::encode(include_str!("./func/leroLeroTitleCommand.js"));

        new_func
            .set_handler(ctx, Some("title".to_string()))
            .await
            .expect("cannot set handler");
        new_func
            .set_code_base64(ctx, Some(workflow_code))
            .await
            .expect("cannot set code");
    }

    Ok(())
}

async fn si_title2_command(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let func_name = "si:title2".to_string();
    let existing_func = Func::find_by_attr(ctx, "name", &func_name).await?;
    if existing_func.is_empty() {
        let mut new_func = Func::new(
            ctx,
            &func_name,
            FuncBackendKind::JsCommand,
            FuncBackendResponseType::Command,
        )
        .await
        .expect("cannot create func");

        let workflow_code = base64::encode(include_str!("./func/leroLeroTitle2Command.js"));

        new_func
            .set_handler(ctx, Some("title2".to_string()))
            .await
            .expect("cannot set handler");
        new_func
            .set_code_base64(ctx, Some(workflow_code))
            .await
            .expect("cannot set code");
    }

    Ok(())
}

async fn si_first_stanza_command(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let func_name = "si:firstStanza".to_string();
    let existing_func = Func::find_by_attr(ctx, "name", &func_name).await?;
    if existing_func.is_empty() {
        let mut new_func = Func::new(
            ctx,
            &func_name,
            FuncBackendKind::JsCommand,
            FuncBackendResponseType::Command,
        )
        .await
        .expect("cannot create func");

        let workflow_code = base64::encode(include_str!("./func/leroLeroFirstStanzaCommand.js"));

        new_func
            .set_handler(ctx, Some("firstStanza".to_string()))
            .await
            .expect("cannot set handler");
        new_func
            .set_code_base64(ctx, Some(workflow_code))
            .await
            .expect("cannot set code");
    }

    Ok(())
}

async fn si_second_stanza_command(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let func_name = "si:secondStanza".to_string();
    let existing_func = Func::find_by_attr(ctx, "name", &func_name).await?;
    if existing_func.is_empty() {
        let mut new_func = Func::new(
            ctx,
            &func_name,
            FuncBackendKind::JsCommand,
            FuncBackendResponseType::Command,
        )
        .await
        .expect("cannot create func");

        let workflow_code = base64::encode(include_str!("./func/leroLeroSecondStanzaCommand.js"));

        new_func
            .set_handler(ctx, Some("secondStanza".to_string()))
            .await
            .expect("cannot set handler");
        new_func
            .set_code_base64(ctx, Some(workflow_code))
            .await
            .expect("cannot set code");
    }

    Ok(())
}

async fn si_third_stanza_command(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let func_name = "si:thirdStanza".to_string();
    let existing_func = Func::find_by_attr(ctx, "name", &func_name).await?;
    if existing_func.is_empty() {
        let mut new_func = Func::new(
            ctx,
            &func_name,
            FuncBackendKind::JsCommand,
            FuncBackendResponseType::Command,
        )
        .await
        .expect("cannot create func");

        let workflow_code = base64::encode(include_str!("./func/leroLeroThirdStanzaCommand.js"));

        new_func
            .set_handler(ctx, Some("thirdStanza".to_string()))
            .await
            .expect("cannot set handler");
        new_func
            .set_code_base64(ctx, Some(workflow_code))
            .await
            .expect("cannot set code");
    }

    Ok(())
}

async fn si_fourth_stanza_command(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let func_name = "si:fourthStanza".to_string();
    let existing_func = Func::find_by_attr(ctx, "name", &func_name).await?;
    if existing_func.is_empty() {
        let mut new_func = Func::new(
            ctx,
            &func_name,
            FuncBackendKind::JsCommand,
            FuncBackendResponseType::Command,
        )
        .await
        .expect("cannot create func");

        let workflow_code = base64::encode(include_str!("./func/leroLeroFourthStanzaCommand.js"));

        new_func
            .set_handler(ctx, Some("fourthStanza".to_string()))
            .await
            .expect("cannot set handler");
        new_func
            .set_code_base64(ctx, Some(workflow_code))
            .await
            .expect("cannot set code");
    }

    Ok(())
}

async fn si_fifth_stanza_command(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let func_name = "si:fifthStanza".to_string();
    let existing_func = Func::find_by_attr(ctx, "name", &func_name).await?;
    if existing_func.is_empty() {
        let mut new_func = Func::new(
            ctx,
            &func_name,
            FuncBackendKind::JsCommand,
            FuncBackendResponseType::Command,
        )
        .await
        .expect("cannot create func");

        let workflow_code = base64::encode(include_str!("./func/leroLeroFifthStanzaCommand.js"));

        new_func
            .set_handler(ctx, Some("fifthStanza".to_string()))
            .await
            .expect("cannot set handler");
        new_func
            .set_code_base64(ctx, Some(workflow_code))
            .await
            .expect("cannot set code");
    }

    Ok(())
}

async fn si_sixth_stanza_command(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let func_name = "si:sixthStanza".to_string();
    let existing_func = Func::find_by_attr(ctx, "name", &func_name).await?;
    if existing_func.is_empty() {
        let mut new_func = Func::new(
            ctx,
            &func_name,
            FuncBackendKind::JsCommand,
            FuncBackendResponseType::Command,
        )
        .await
        .expect("cannot create func");

        let workflow_code = base64::encode(include_str!("./func/leroLeroSixthStanzaCommand.js"));

        new_func
            .set_handler(ctx, Some("sixthStanza".to_string()))
            .await
            .expect("cannot set handler");
        new_func
            .set_code_base64(ctx, Some(workflow_code))
            .await
            .expect("cannot set code");
    }

    Ok(())
}

async fn si_seventh_stanza_command(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let func_name = "si:seventhStanza".to_string();
    let existing_func = Func::find_by_attr(ctx, "name", &func_name).await?;
    if existing_func.is_empty() {
        let mut new_func = Func::new(
            ctx,
            &func_name,
            FuncBackendKind::JsCommand,
            FuncBackendResponseType::Command,
        )
        .await
        .expect("cannot create func");

        let workflow_code = base64::encode(include_str!("./func/leroLeroSeventhStanzaCommand.js"));

        new_func
            .set_handler(ctx, Some("seventhStanza".to_string()))
            .await
            .expect("cannot set handler");
        new_func
            .set_code_base64(ctx, Some(workflow_code))
            .await
            .expect("cannot set code");
    }

    Ok(())
}

async fn si_question_command(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let func_name = "si:question".to_string();
    let existing_func = Func::find_by_attr(ctx, "name", &func_name).await?;
    if existing_func.is_empty() {
        let mut new_func = Func::new(
            ctx,
            &func_name,
            FuncBackendKind::JsCommand,
            FuncBackendResponseType::Command,
        )
        .await
        .expect("cannot create func");

        let workflow_code = base64::encode(include_str!("./func/leroLeroQuestionCommand.js"));

        new_func
            .set_handler(ctx, Some("question".to_string()))
            .await
            .expect("cannot set handler");
        new_func
            .set_code_base64(ctx, Some(workflow_code))
            .await
            .expect("cannot set code");
    }

    Ok(())
}

async fn si_bye_command(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let func_name = "si:bye".to_string();
    let existing_func = Func::find_by_attr(ctx, "name", &func_name).await?;
    if existing_func.is_empty() {
        let mut new_func = Func::new(
            ctx,
            &func_name,
            FuncBackendKind::JsCommand,
            FuncBackendResponseType::Command,
        )
        .await
        .expect("cannot create func");

        let workflow_code = base64::encode(include_str!("./func/leroLeroByeCommand.js"));

        new_func
            .set_handler(ctx, Some("bye".to_string()))
            .await
            .expect("cannot set handler");
        new_func
            .set_code_base64(ctx, Some(workflow_code))
            .await
            .expect("cannot set code");
    }

    Ok(())
}
