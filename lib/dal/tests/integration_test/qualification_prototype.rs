use crate::test_setup;

use dal::func::backend::FuncBackendJsQualificationArgs;
use dal::qualification_prototype::QualificationPrototypeContext;
use dal::{
    qualification_prototype::UNSET_ID_VALUE, test_harness::billing_account_signup, Component,
    ComponentQualificationView, Func, HistoryActor, QualificationPrototype, Schema, StandardModel,
    Tenancy, Visibility,
};

#[tokio::test]
async fn new() {
    test_setup!(ctx, _secret_key, _pg, _conn, txn, nats_conn, nats);
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;

    let name = "docker_image".to_string();
    let schema = Schema::find_by_attr(&txn, &tenancy, &visibility, "name", &name)
        .await
        .expect("cannot find docker image")
        .pop()
        .expect("no docker image found");
    let (component, _node) = Component::new_for_schema_with_node(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &name,
        schema.id(),
    )
    .await
    .expect("could not create component");

    let func_name = "si:qualificationDockerImageNameEqualsComponentName".to_string();
    let mut funcs = Func::find_by_attr(&txn, &tenancy, &visibility, "name", &func_name)
        .await
        .expect("Error fetching builtin function");
    let func = funcs
        .pop()
        .expect("Missing builtin function si:qualificationDockerImageNameEqualsComponentName");

    let args = FuncBackendJsQualificationArgs {
        component: ComponentQualificationView::new(&txn, &tenancy, &visibility, component.id())
            .await
            .expect("could not create component qualification view"),
    };

    let mut prototype_context = QualificationPrototypeContext::new();
    prototype_context.set_component_id(*component.id());
    let _prototype = QualificationPrototype::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *func.id(),
        serde_json::to_value(&args).expect("serialization failed"),
        prototype_context,
    )
    .await
    .expect("cannot create new prototype");
}

#[tokio::test]
async fn find_for_component_id() {
    test_setup!(ctx, secret_key, pg, _conn, txn, nats_conn, nats);
    let (nba, _token) = billing_account_signup(&txn, &nats, secret_key).await;
    let mut tenancy = Tenancy::new_workspace(vec![*nba.workspace.id()]);
    tenancy.universal = true;
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;

    let name = "docker_image".to_string();
    let schema = Schema::find_by_attr(&txn, &tenancy, &visibility, "name", &name)
        .await
        .expect("cannot find docker image")
        .pop()
        .expect("no docker image found");
    let (component, _node) = Component::new_for_schema_with_node(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &name,
        schema.id(),
    )
    .await
    .expect("could not create component");

    let func_name = "si:qualificationDockerImageNameEqualsComponentName".to_string();
    let mut funcs = Func::find_by_attr(&txn, &tenancy, &visibility, "name", &func_name)
        .await
        .expect("Error fetching builtin function");
    let func = funcs
        .pop()
        .expect("Missing builtin function si:qualificationDockerImageNameEqualsComponentName");

    let args = FuncBackendJsQualificationArgs {
        component: ComponentQualificationView::new(&txn, &tenancy, &visibility, component.id())
            .await
            .expect("could not create component qualification view"),
    };

    let mut prototype_context = QualificationPrototypeContext::new();
    prototype_context.set_component_id(*component.id());
    let created = QualificationPrototype::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *func.id(),
        serde_json::to_value(&args).expect("serialization failed"),
        prototype_context,
    )
    .await
    .expect("cannot create new attribute prototype");

    let mut found_prototypes = QualificationPrototype::find_for_component_id(
        &txn,
        &tenancy,
        &visibility,
        *component.id(),
        UNSET_ID_VALUE.into(),
    )
    .await
    .expect("could not create component qualification view");
    assert_eq!(found_prototypes.len(), 1);
    let found = found_prototypes
        .pop()
        .expect("found no qualification prototypes");
    assert_eq!(created, found);
}
