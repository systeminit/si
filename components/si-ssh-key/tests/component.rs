mod common;

use si_ssh_key::ssh_key::{
    query_expression_option::Qe, CreateEntityRequest, GetComponentRequest, GetEntityRequest,
    KeyFormat, KeyType, ListComponentsRequest, ListEntitiesRequest, PageToken, Query,
    QueryComparison, QueryExpression, QueryExpressionOption,
};
use tokio;

#[test]
fn run_server_and_client() {
    let _ = common::SERVER.clone();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let f = async { common::get_connected_client().await };
    rt.block_on(f);
}

#[test]
fn create_entity() {
    let _ = common::SERVER.clone();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let f = async {
        let mut client = common::get_connected_client().await;

        // Create an entity, mother-trucker - with the default
        let request = tonic::Request::new(CreateEntityRequest {
            tenant_id: "insomnium".to_string(),
            ..Default::default()
        });
        let reply = client
            .create_entity(request)
            .await
            .expect("Failed to get a response");
        let rd = reply.into_inner();
        let entity = rd.entity.expect("Did not return an entity");
        assert_eq!(
            entity.key_format,
            KeyFormat::Rfc4716 as i32,
            "KeyType is wrong"
        );
        assert_eq!(entity.key_type, KeyType::Rsa as i32, "KeyType is wrong");
        assert_eq!(entity.bits, 3072, "bits is wrong");
        assert_eq!(entity.type_name, "entity:ssh_key");
    };
    rt.block_on(f);
}

#[test]
fn get_entity() {
    let _ = common::SERVER.clone();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let f = async {
        let mut client = common::get_connected_client().await;

        // Create an entity so we can get it again
        let request = tonic::Request::new(CreateEntityRequest {
            tenant_id: "insomnium".to_string(),
            ..Default::default()
        });
        let reply = client
            .create_entity(request)
            .await
            .expect("Failed to get a response to create");
        let rd = reply.into_inner();
        let created_entity = rd.entity.expect("Did not return an entity on creation");

        // Get the entity!
        let request = tonic::Request::new(GetEntityRequest {
            tenant_id: "insomnium".to_string(),
            entity_id: created_entity.id.clone(),
        });
        let reply = client
            .get_entity(request)
            .await
            .expect("Failed to get a response to get");
        let rd = reply.into_inner();
        let entity = rd.entity.expect("Did not return an entity on get");
        assert_eq!(
            created_entity.id, entity.id,
            "Created and get entity do not match"
        );

        // Get the entity with a wrong tenant id
        let request = tonic::Request::new(GetEntityRequest {
            tenant_id: "inflames".to_string(),
            entity_id: created_entity.id.clone(),
        });
        let reply = client.get_entity(request).await;
        match reply {
            Ok(_) => assert!(false, "Get succeeded with invalid tenant id"),
            Err(e) => {
                assert_eq!(e.code(), tonic::Code::InvalidArgument);
                assert_eq!(e.message(), "Invalid tenant specified for entity");
            }
        }
    };
    rt.block_on(f);
}

#[test]
fn list_entities() {
    let _ = common::SERVER.clone();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let f = async {
        let mut client = common::get_connected_client().await;

        // Create an entity so we can get it again
        let request = tonic::Request::new(CreateEntityRequest {
            tenant_id: "insomnium".to_string(),
            ..Default::default()
        });
        let reply = client
            .create_entity(request)
            .await
            .expect("Failed to get a response to create");
        let rd = reply.into_inner();
        let created_entity = rd.entity.expect("Did not return an entity on creation");

        // Create an entity so we can get it again
        let request = tonic::Request::new(CreateEntityRequest {
            tenant_id: "insomnium".to_string(),
            ..Default::default()
        });
        let reply = client
            .create_entity(request)
            .await
            .expect("Failed to get a response to create");
        let rd = reply.into_inner();
        let created_entity = rd.entity.expect("Did not return an entity on creation");

        // Get a list of entities
        let request = tonic::Request::new(ListEntitiesRequest {
            page_size: 2,
            order_by: "bits".into(),
            ..Default::default()
        });
        let reply = client
            .list_entities(request)
            .await
            .expect("Failed to get a response");
        let rd = reply.into_inner();

        // The number of current combinations
        assert!(
            rd.total_count >= 1,
            "Should have the total number of entities"
        );

        // Should have a next page token
        PageToken::unseal(&rd.next_page_token, &common::SETTINGS.paging.key)
            .expect("Could not unseal page token");
        let mut next_page_token = rd.next_page_token.clone();

        let total_entries = rd.total_count as usize;
        let mut current_count = rd.entity.len();

        // Safeguard against infinite loop
        let page_limit = 50;
        let mut current_page: usize = 1;

        // Iterate over all the remaining items by fetching the next_page_token
        while next_page_token != "" && current_page < page_limit {
            let request = tonic::Request::new(ListEntitiesRequest {
                page_token: next_page_token,
                ..Default::default()
            });
            let reply = client
                .list_entities(request)
                .await
                .expect("Failed to get a response");
            let rd = reply.into_inner();
            next_page_token = rd.next_page_token.clone();
            current_page = current_page + 1;
            current_count = current_count + rd.entity.len();
        }
        assert!(current_page < page_limit, "Exceeded page limit");
        assert_eq!(
            current_count, total_entries,
            "Incorrect number of total entries"
        );
    };
    rt.block_on(f);
}

#[test]
fn list_components() {
    let _ = common::SERVER.clone();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let f = async {
        let mut client = common::get_connected_client().await;

        // Get a list of components
        let request = tonic::Request::new(ListComponentsRequest {
            page_size: 2,
            order_by: "bits".into(),
            ..Default::default()
        });
        let reply = client
            .list_components(request)
            .await
            .expect("Failed to get a response");
        let rd = reply.into_inner();

        // The number of current combinations
        assert!(
            rd.total_count >= 1,
            "Should have the total number of components"
        );

        // Should have a next page token
        PageToken::unseal(&rd.next_page_token, &common::SETTINGS.paging.key)
            .expect("Could not unseal page token");
        let mut next_page_token = rd.next_page_token.clone();

        let total_entries = rd.total_count as usize;
        let mut current_count = rd.component.len();

        // Safeguard against infinite loop
        let page_limit = 50;
        let mut current_page: usize = 1;

        // Iterate over all the remaining items by fetching the next_page_token
        while next_page_token != "" && current_page < page_limit {
            let request = tonic::Request::new(ListComponentsRequest {
                page_token: next_page_token,
                ..Default::default()
            });
            let reply = client
                .list_components(request)
                .await
                .expect("Failed to get a response");
            let rd = reply.into_inner();
            next_page_token = rd.next_page_token.clone();
            current_page = current_page + 1;
            current_count = current_count + rd.component.len();
        }
        assert!(current_page < page_limit, "Exceeded page limit");
        assert_eq!(
            current_count, total_entries,
            "Incorrect number of total entries"
        );

        // Reject invalid order by operations
        let request = tonic::Request::new(ListComponentsRequest {
            order_by: "not-a-real-thing".into(),
            ..Default::default()
        });
        let result = client.list_components(request).await;
        match result {
            Ok(_) => assert!(false, "Succeded on bad order_by lookup"),
            Err(e) => assert_eq!(e.code(), tonic::Code::InvalidArgument),
        };

        // Allow for queries
        let request = tonic::Request::new(ListComponentsRequest {
            query: Some(Query {
                items: vec![QueryExpressionOption {
                    qe: Some(Qe::Expression(QueryExpression {
                        field: "name".to_string(),
                        comparison: QueryComparison::Equals as i32,
                        value: "ED25519 256 RFC4716".to_string(),
                        ..Default::default()
                    })),
                }],
                ..Default::default()
            }),
            ..Default::default()
        });
        let reply = client
            .list_components(request)
            .await
            .expect("Failed request for query");
        let rd = reply.into_inner();

        assert_eq!(rd.total_count, 1, "Should have one entry in query result");
        let item = rd
            .component
            .first()
            .expect("Should have one item in the list");
        assert_eq!(
            item.name, "ED25519 256 RFC4716",
            "Should match the name we queried for"
        );
    };
    rt.block_on(f);
}

#[test]
fn get_component() {
    let _ = common::SERVER.clone();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let f = async {
        let mut client = common::get_connected_client().await;

        // Get a component to... get
        let lrequest = tonic::Request::new(ListComponentsRequest {
            page_size: 1,
            ..Default::default()
        });
        let lreply = client
            .list_components(lrequest)
            .await
            .expect("Failed to get a response");
        let lrd = lreply.into_inner();
        let component_id = lrd
            .component
            .first()
            .expect("No components returned, so we can't look them up")
            .id
            .clone();

        let request = tonic::Request::new(GetComponentRequest {
            component_id: component_id,
            ..Default::default()
        });
        let reply = client
            .get_component(request)
            .await
            .expect("Failed to get a response");
        let rd = reply.into_inner().component.expect("Cannot find component");
        assert_eq!(rd.type_name, "component:ssh_key");
    };
    rt.block_on(f);
}
