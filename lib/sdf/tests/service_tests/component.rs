use std::collections::HashSet;

use crate::service_tests::api_request_auth_query;
use crate::test_setup;
use dal::{Component, HistoryActor, StandardModel, Tenancy, Visibility};
use sdf::service::component::list_components_names_only::{
    ListComponentNamesOnlyRequest, ListComponentNamesOnlyResponse,
};

#[tokio::test]
async fn list_components_names_only() {
    test_setup!(
        _ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        app,
        nba,
        auth_token
    );
    let visibility = Visibility::new_head(false);
    let tenancy = Tenancy::new_workspace(vec![*nba.workspace.id()]);
    let history_actor = HistoryActor::SystemInit;

    let component_name1 = "poop";
    let component_name2 = "ilikemybutt";
    for name in vec![component_name1, component_name2] {
        let _component = Component::new(&txn, &nats, &tenancy, &visibility, &history_actor, &name)
            .await
            .expect("cannot create new component");
    }
    txn.commit().await.expect("cannot commit transaction");

    let request = ListComponentNamesOnlyRequest {
        visibility,
        workspace_id: *nba.workspace.id(),
    };
    let response: ListComponentNamesOnlyResponse = api_request_auth_query(
        app,
        "/api/component/list_components_names_only",
        &auth_token,
        &request,
    )
    .await;

    let filtered_components_names_only: HashSet<String> = response
        .list
        .iter()
        .filter_map(|list_item| match &list_item.label {
            component_name
                if component_name == component_name1 || component_name == component_name2 =>
            {
                Some(component_name.to_string())
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        filtered_components_names_only,
        vec![component_name1.to_string(), component_name2.to_string()]
            .into_iter()
            .collect()
    );
}
