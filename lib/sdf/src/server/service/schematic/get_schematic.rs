use axum::{extract::Query, Json};
use dal::{Visibility, Tenancy, schematic::Schematic, WorkspaceId, node::NodeId, system::SystemId};
use serde::{Deserialize, Serialize};

use super::SchematicResult;
use crate::server::extract::{Authorization, PgRoTxn};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetSchematicRequest {
    pub root_node_id: NodeId,
    pub system_id: Option<SystemId>,
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type GetSchematicResponse = Schematic;

pub async fn get_schematic(
    mut txn: PgRoTxn,
    Authorization(_claim): Authorization,
    Query(request): Query<GetSchematicRequest>,
) -> SchematicResult<Json<GetSchematicResponse>> {
    let txn = txn.start().await?;
    let tenancy = Tenancy::new_workspace(vec![request.workspace_id]);
    let response = Schematic::find(&txn, &tenancy, &request.visibility, request.system_id, request.root_node_id).await?;
    println!("{:?}", response);

    //let response = serde_json::json![{
    //  "nodes": [{
    //      "id": "A:1",
    //      "label": {
    //        "title": "k8s service",
    //        "name": "whiskers malandragem",
    //      },
    //      "classification": {
    //        "component": "application",
    //        "kind": "kubernetes",
    //        "type": "service",
    //      },
    //      "status": {
    //        "qualification": "succeeded",
    //        "resource": "healthy",
    //        "changeCount": 3,
    //        "action": {
    //          "name": "aaa",
    //          "timestamp": "123",
    //          "status": "succeeded",
    //        },
    //      },
    //      "position": {
    //        "ctx": [
    //          {
    //            "id": "aaa",
    //            "position": {
    //              "x": 300,
    //              "y": 100,
    //            },
    //          },
    //        ],
    //      },
    //      "input": [
    //        {
    //          "id": "A:1.S:1",
    //          "type": "kubernetes.namespace",
    //          "name": "namespace",
    //        },
    //        {
    //          "id": "A:1.S:2",
    //          "type": "kubernetes.deployment",
    //          "name": "deployment",
    //        },
    //        {
    //          "id": "A:1.S:3",
    //          "type": "kubernetes.service",
    //          "name": "service",
    //        },
    //        {
    //          "id": "A:1.S:4",
    //          "type": "kubernetes.env",
    //          "name": "env",
    //        },
    //      ],
    //      "output": [
    //        {
    //          "id": "A:1.S:5",
    //          "type": "kubernetes.service",
    //        },
    //      ],
    //      "display": {
    //        "color": "0x32b832",
    //      },
    //      "connections": [],
    //      "lastUpdated": "1234",
    //      "checksum": "j4j4j4j4j4j4j4j4j4j4j4",
    //      "schematic": {
    //        "deployment": false,
    //        "component": true,
    //      },
    //    }, {
    //      "id": "B:1",
    //      "label": {
    //        "title": "k8s namespace",
    //        "name": "dev",
    //      },
    //      "classification": {
    //        "component": "application",
    //        "kind": "kubernetes",
    //        "type": "namespace",
    //      },
    //      "status": {
    //        "qualification": "succeeded",
    //        "resource": "healthy",
    //        "changeCount": 0,
    //        "action": {
    //          "name": "aaa",
    //          "timestamp": "123",
    //          "status": "succeeded",
    //        },
    //      },
    //      "position": {
    //        "ctx": [
    //          {
    //            "id": "B:1.S:1",
    //            "position": {
    //              "x": 100,
    //              "y": 100,
    //            },
    //          },
    //        ],
    //      },
    //      "input": [
    //      ],
    //      "output": [
    //        {
    //          "id": "B:1.S:5",
    //          "type": "kubernetes.namespace",
    //        },
    //      ],
    //      "display": {
    //        "color": "0x3251b8",
    //      },
    //      "connections": [],
    //      "lastUpdated": "1234",
    //      "checksum": "j4j4j4j4j4j4j4j4j4j4j4",
    //      "schematic": {
    //        "deployment": false,
    //        "component": true,
    //      },
    //    },
    //  ],
    //  "connections": [],
    //  "lastUpdated": "123",
    //  "checksum": "i5i5i55i5i5i5i55i5i5i",
    //}];
    Ok(Json(response))
}
