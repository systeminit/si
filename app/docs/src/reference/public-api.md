---
outline:
  level: [1, 2, 3, 4]
---

# System Initiative Public API

This is the API spec for the System Initiative Public API. All endpoints require a workspace scoped API token for the workspace to interact with.

# [root](#system-initiative-api-root)

Root API endpoints

## system_status_route

<a id="opIdsystem_status_route"></a>

> Request format

`GET /`

> Example responses

> 200 Response

```json
{
  "API Documentation": "Available at /swagger-ui"
}
```

<h3 id="system_status_route-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|System status information|[SystemStatusResponse](#schemasystemstatusresponse)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|503|[Service Unavailable](https://tools.ietf.org/html/rfc7231#section-6.6.4)|Service in maintenance mode|None|

# [whoami](#system-initiative-api-whoami)

User identity endpoints

## whoami

<a id="opIdwhoami"></a>

> Request format

`GET /whoami`

> Example responses

> 200 Response

```json
{
  "token": {},
  "userEmail": "user@example.com",
  "userId": "01H9ZQCBJ3E7HBTRN3J58JQX8K",
  "workspaceId": "01H9ZQD35JPMBGHH69BT0Q79VY"
}
```

<h3 id="whoami-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Successfully retrieved user information|[WhoamiResponse](#schemawhoamiresponse)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or expired token|None|
|403|[Forbidden](https://tools.ietf.org/html/rfc7231#section-6.5.3)|Forbidden - Insufficient permissions|None|

# [change_sets](#system-initiative-api-change_sets)

Change Set management endpoints

## List all active Change Sets

<a id="opIdlist_change_sets"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets`

<h3 id="list-all-active-change-sets-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|

> Example responses

> 200 Response

```json
{
  "changeSets": "[{\"id\":\"01H9ZQD35JPMBGHH69BT0Q79VY\",\"name\":\"Add new feature\",\"status\":\"Open\",\"isHead\": \"false\"},{\"id\":\"01H9ZQE356JPMBGHH69BT0Q70UO\",\"name\":\"HEAD\",\"status\":\"Open\", \"isHead\": \"true\"}]"
}
```

<h3 id="list-all-active-change-sets-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Change Sets listed successfully|[ListChangeSetV1Response](#schemalistchangesetv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Create a Change Set

<a id="opIdcreate_change_set"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets`

> Body parameter

```json
{
  "changeSetName": "My new feature"
}
```

<h3 id="create-a-change-set-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|body|body|[CreateChangeSetV1Request](#schemacreatechangesetv1request)|true|none|

> Example responses

> 200 Response

```json
{
  "changeSet": {
    "id": "string",
    "isHead": true,
    "name": "string",
    "status": "string"
  }
}
```

<h3 id="create-a-change-set-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Change Set created successfully|[CreateChangeSetV1Response](#schemacreatechangesetv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|422|[Unprocessable Entity](https://tools.ietf.org/html/rfc2518#section-10.3)|Validation error - Invalid request data|[ApiError](#schemaapierror)|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Abandon all active Change Sets

<a id="opIdpurge_open"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/purge_open`

<h3 id="abandon-all-active-change-sets-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|

> Example responses

> 200 Response

```json
{
  "success": {
    "success": "true"
  }
}
```

<h3 id="abandon-all-active-change-sets-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Change Sets purged successfully|[PurgeOpenChangeSetsV1Response](#schemapurgeopenchangesetsv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Get a Change Set by Change Set Id

<a id="opIdget_change_set"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}`

<h3 id="get-a-change-set-by-change-set-id-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|

> Example responses

> 200 Response

```json
{
  "changeSet": {
    "id": "string",
    "isHead": true,
    "name": "string",
    "status": "string"
  }
}
```

<h3 id="get-a-change-set-by-change-set-id-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Change details retrieved successfully|[GetChangeSetV1Response](#schemagetchangesetv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Change Set not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Delete a Change Set

<a id="opIdabandon_change_set"></a>

> Request format

`DELETE /v1/w/{workspace_id}/change-sets/{change_set_id}`

<h3 id="delete-a-change-set-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|

> Example responses

> 200 Response

```json
{
  "success": "true"
}
```

<h3 id="delete-a-change-set-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Change Set deleted successfully|[DeleteChangeSetV1Response](#schemadeletechangesetv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Merge Change Set without approval

<a id="opIdforce_apply"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/force_apply`

<h3 id="merge-change-set-without-approval-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|

> Example responses

> 200 Response

```json
{
  "success": "true"
}
```

<h3 id="merge-change-set-without-approval-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Change Set force applied successfully|[ForceApplyChangeSetV1Response](#schemaforceapplychangesetv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Get Change Set post merge status

<a id="opIdmerge_status"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/merge_status`

<h3 id="get-change-set-post-merge-status-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|

> Example responses

> 200 Response

```json
{
  "changeSet": {
    "id": "01FXNV4P306V3KGZ73YSVN8A60",
    "name": "My feature",
    "status": "Ready"
  },
  "actions": [
    {
      "id": "01H9ZQD35JPMBGHH69BT0Q79VY",
      "component": {
        "id": "01H9ZQD35JPMBGHH69BT0Q79AB",
        "name": "my-ec2-instance"
      },
      "state": "Pending",
      "kind": "Create",
      "name": "Create EC2 Instance"
    }
  ]
}
```

<h3 id="get-change-set-post-merge-status-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Change Set merge status retrieved successfully|[MergeStatusV1Response](#schemamergestatusv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Request Change Set merge approval

<a id="opIdrequest_approval"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/request_approval`

<h3 id="request-change-set-merge-approval-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|

> Example responses

> 200 Response

```json
{
  "success": "true"
}
```

<h3 id="request-change-set-merge-approval-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Change Set approval requested successfully|[RequestApprovalChangeSetV1Response](#schemarequestapprovalchangesetv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

# [components](#system-initiative-api-components)

Components management endpoints

## List all components

<a id="opIdlist_components"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/components`

<h3 id="list-all-components-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|limit|query|string|false|Maximum number of results to return (default: 50, max: 300)|
|cursor|query|string|false|Cursor for pagination (ComponentId of the last item from previous page)|

> Example responses

> 200 Response

```json
{
  "components": [
    "01H9ZQD35JPMBGHH69BT0Q79AA",
    "01H9ZQD35JPMBGHH69BT0Q79BB"
  ],
  "componentDetails": [
    {
      "component_id": "01H9ZQD35JPMBGHH69BT0Q79AA",
      "name": "my-vpc",
      "schema_name": "AWS::EC2::VPC"
    },
    {
      "component_id": "01H9ZQD35JPMBGHH69BT0Q79BB",
      "name": "Public 1",
      "schema_name": "AWS::EC2::Subnet"
    }
  ],
  "nextCursor": null
}
```

> 500 Response

```json
{
  "message": "Invalid request data",
  "statusCode": 422,
  "code": 4001
}
```

<h3 id="list-all-components-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Components retrieved successfully|[ListComponentsV1Response](#schemalistcomponentsv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Create a component

<a id="opIdcreate_component"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/components`

> Body parameter

```json
{
  "attributes": {
    "/domain/VpcId": {
      "$source": {
        "component": "01K0WRC69ZPEMD6SMTKC84FBWC",
        "path": "/resource_value/VpcId"
      }
    },
    "/domain/SubnetId": {
      "$source": {
        "component": "01K0WRC69ZPEMD6SMTKC84FBWD",
        "path": "/resource_value/SubnetId"
      }
    },
    "/domain/Version": {
      "$source": null
    }
  },
  "connections": {},
  "domain": {},
  "managedBy": {
    "component": "ComponentName"
  },
  "name": "MyComponentName",
  "resourceId": "i-12345678",
  "schemaName": "AWS::EC2::Instance",
  "secrets": {},
  "subscriptions": {},
  "viewName": "MyView"
}
```

<h3 id="create-a-component-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|body|body|[CreateComponentV1Request](#schemacreatecomponentv1request)|true|none|

> Example responses

> 200 Response

```json
{
  "component": {
    "attributes": {
      "/domain/region": "us-east-1",
      "/secrets/credential": {
        "$source": {
          "component": "demo-credential",
          "path": "/secrets/AWS Credential"
        }
      }
    },
    "canBeUpgraded": true,
    "connections": [
      {
        "incoming": {
          "from": "string",
          "fromComponentId": "string",
          "fromComponentName": "string",
          "to": "string"
        }
      }
    ],
    "domainProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": {}
      }
    ],
    "id": "string",
    "name": "string",
    "resourceId": "string",
    "resourceProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": {}
      }
    ],
    "schemaId": "string",
    "schemaVariantId": "string",
    "sockets": [
      {
        "arity": "one",
        "direction": "input",
        "id": "string",
        "name": "string",
        "value": {}
      }
    ],
    "toDelete": true,
    "views": [
      {
        "id": "string",
        "isDefault": true,
        "name": "string"
      }
    ]
  }
}
```

<h3 id="create-a-component-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Component created successfully|[CreateComponentV1Response](#schemacreatecomponentv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Component not found|None|
|412|[Precondition Failed](https://tools.ietf.org/html/rfc7232#section-4.2)|Precondition Failed - View not found|[ApiError](#schemaapierror)|
|422|[Unprocessable Entity](https://tools.ietf.org/html/rfc2518#section-10.3)|Validation error - Invalid request data|[ApiError](#schemaapierror)|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Duplicate a list of components

<a id="opIdduplicate_components"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/components/duplicate`

> Body parameter

```json
{
  "components": [
    "01H9ZQD35JPMBGHH69BT0Q79AA",
    "01H9ZQD35JPMBGHH69BT0Q79BB",
    "01H9ZQD35JPMBGHH69BT0Q79CC"
  ],
  "prefix": "copy-of-",
  "viewName": "MyView"
}
```

<h3 id="duplicate-a-list-of-components-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|body|body|[DuplicateComponentsV1Request](#schemaduplicatecomponentsv1request)|true|none|

> Example responses

> 200 Response

```json
{
  "components": [
    "01H9ZQD35JPMBGHH69BT0Q79AA",
    "01H9ZQD35JPMBGHH69BT0Q79BB",
    "01H9ZQD35JPMBGHH69BT0Q79CC"
  ]
}
```

<h3 id="duplicate-a-list-of-components-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Components duplicated successfully|[DuplicateComponentsV1Response](#schemaduplicatecomponentsv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Find a component by name or component Id

<a id="opIdfind_component"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/components/find`

<h3 id="find-a-component-by-name-or-component-id-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|component|query|string,null|false|none|
|componentId|query|string,null|false|none|

> Example responses

> 200 Response

```json
{
  "actionFunctions": [
    {
      "prototypeId": "01HAXYZF3GC9CYA6ZVSM3E4YGG",
      "funcName": "Terminate Instance"
    }
  ],
  "component": {
    "attributes": {
      "/domain/region": "us-east-1",
      "/secrets/credential": {
        "$source": {
          "component": "demo-credential",
          "path": "/secrets/AWS Credential"
        }
      }
    },
    "canBeUpgraded": true,
    "connections": [
      {
        "incoming": {
          "from": "string",
          "fromComponentId": "string",
          "fromComponentName": "string",
          "to": "string"
        }
      }
    ],
    "domainProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": {}
      }
    ],
    "id": "string",
    "name": "string",
    "resourceId": "string",
    "resourceProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": {}
      }
    ],
    "schemaId": "string",
    "schemaVariantId": "string",
    "sockets": [
      {
        "arity": "one",
        "direction": "input",
        "id": "string",
        "name": "string",
        "value": {}
      }
    ],
    "toDelete": true,
    "views": [
      {
        "id": "string",
        "isDefault": true,
        "name": "string"
      }
    ]
  },
  "managementFunctions": [
    {
      "managementPrototypeId": "01HAXYZF3GC9CYA6ZVSM3E4YFF",
      "funcName": "Start Instance"
    }
  ]
}
```

<h3 id="find-a-component-by-name-or-component-id-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Component retrieved successfully|[GetComponentV1Response](#schemagetcomponentv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Component not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Complex search for components

<a id="opIdsearch_components"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/components/search`

> Body parameter

```json
{
  "schemaName": "AWS::EC2::Instance"
}
```

<h3 id="complex-search-for-components-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|body|body|[SearchComponentsV1Request](#schemasearchcomponentsv1request)|true|none|

> Example responses

> 200 Response

```json
{
  "components": [
    "01H9ZQD35JPMBGHH69BT0Q79AA",
    "01H9ZQD35JPMBGHH69BT0Q79BB",
    "01H9ZQD35JPMBGHH69BT0Q79CC"
  ]
}
```

<h3 id="complex-search-for-components-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Components retrieved successfully|[SearchComponentsV1Response](#schemasearchcomponentsv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Component not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Get a component by component Id

<a id="opIdget_component"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}`

<h3 id="get-a-component-by-component-id-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|component_id|path|string|true|Component identifier|

> Example responses

> 200 Response

```json
{
  "actionFunctions": [
    {
      "prototypeId": "01HAXYZF3GC9CYA6ZVSM3E4YGG",
      "funcName": "Terminate Instance"
    }
  ],
  "component": {
    "attributes": {
      "/domain/region": "us-east-1",
      "/secrets/credential": {
        "$source": {
          "component": "demo-credential",
          "path": "/secrets/AWS Credential"
        }
      }
    },
    "canBeUpgraded": true,
    "connections": [
      {
        "incoming": {
          "from": "string",
          "fromComponentId": "string",
          "fromComponentName": "string",
          "to": "string"
        }
      }
    ],
    "domainProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": {}
      }
    ],
    "id": "string",
    "name": "string",
    "resourceId": "string",
    "resourceProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": {}
      }
    ],
    "schemaId": "string",
    "schemaVariantId": "string",
    "sockets": [
      {
        "arity": "one",
        "direction": "input",
        "id": "string",
        "name": "string",
        "value": {}
      }
    ],
    "toDelete": true,
    "views": [
      {
        "id": "string",
        "isDefault": true,
        "name": "string"
      }
    ]
  },
  "managementFunctions": [
    {
      "managementPrototypeId": "01HAXYZF3GC9CYA6ZVSM3E4YFF",
      "funcName": "Start Instance"
    }
  ]
}
```

<h3 id="get-a-component-by-component-id-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Component retrieved successfully|[GetComponentV1Response](#schemagetcomponentv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Component not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Update a component

<a id="opIdupdate_component"></a>

> Request format

`PUT /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}`

> Body parameter

```json
{
  "attributes": {
    "/domain/VpcId": {
      "$source": {
        "component": "01K0WRC69ZPEMD6SMTKC84FBWC",
        "path": "/resource_value/VpcId"
      }
    },
    "/domain/SubnetId": {
      "$source": {
        "component": "01K0WRC69ZPEMD6SMTKC84FBWD",
        "path": "/resource_value/SubnetId"
      }
    },
    "/domain/Version": {
      "$source": null
    }
  },
  "connectionChanges": {
    "add": {},
    "remove": {}
  },
  "domain": {},
  "name": "MyUpdatedComponentName",
  "resourceId": "i-12345678",
  "secrets": {},
  "subscriptions": {},
  "unset": {}
}
```

<h3 id="update-a-component-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|component_id|path|string|true|Component identifier|
|body|body|[UpdateComponentV1Request](#schemaupdatecomponentv1request)|true|none|

> Example responses

> 200 Response

```json
{
  "component": {
    "attributes": {
      "/domain/region": "us-east-1",
      "/secrets/credential": {
        "$source": {
          "component": "demo-credential",
          "path": "/secrets/AWS Credential"
        }
      }
    },
    "canBeUpgraded": true,
    "connections": [
      {
        "incoming": {
          "from": "string",
          "fromComponentId": "string",
          "fromComponentName": "string",
          "to": "string"
        }
      }
    ],
    "domainProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": {}
      }
    ],
    "id": "string",
    "name": "string",
    "resourceId": "string",
    "resourceProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": {}
      }
    ],
    "schemaId": "string",
    "schemaVariantId": "string",
    "sockets": [
      {
        "arity": "one",
        "direction": "input",
        "id": "string",
        "name": "string",
        "value": {}
      }
    ],
    "toDelete": true,
    "views": [
      {
        "id": "string",
        "isDefault": true,
        "name": "string"
      }
    ]
  }
}
```

<h3 id="update-a-component-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Component updated successfully|[UpdateComponentV1Response](#schemaupdatecomponentv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Component not found|None|
|412|[Precondition Failed](https://tools.ietf.org/html/rfc7232#section-4.2)|Precondition failed - Duplicate component name|None|
|422|[Unprocessable Entity](https://tools.ietf.org/html/rfc2518#section-10.3)|Validation error - Invalid request data|[ApiError](#schemaapierror)|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Delete a component

<a id="opIddelete_component"></a>

> Request format

`DELETE /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}`

<h3 id="delete-a-component-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|component_id|path|string|true|Component identifier|

> Example responses

> 200 Response

```json
{
  "status": "MarkedForDeletion"
}
```

<h3 id="delete-a-component-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Component deleted successfully|[DeleteComponentV1Response](#schemadeletecomponentv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Component not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Queue action for a component

<a id="opIdadd_action"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}/action`

> Body parameter

```json
{
  "action": {
    "function": "Create Asset"
  }
}
```

<h3 id="queue-action-for-a-component-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|component_id|path|string|true|Component identifier|
|body|body|[AddActionV1Request](#schemaaddactionv1request)|true|none|

> Example responses

> 200 Response

```json
{
  "success": true
}
```

<h3 id="queue-action-for-a-component-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Action successfully queued|[AddActionV1Response](#schemaaddactionv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Component or function not found|None|
|409|[Conflict](https://tools.ietf.org/html/rfc7231#section-6.5.8)|action already enqueued|[ApiError](#schemaapierror)|
|412|[Precondition Failed](https://tools.ietf.org/html/rfc7232#section-4.2)|Precondition Failed - View not found or duplicate function name|[ApiError](#schemaapierror)|
|422|[Unprocessable Entity](https://tools.ietf.org/html/rfc2518#section-10.3)|Validation error - Invalid request data|[ApiError](#schemaapierror)|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Execute a component's management function

<a id="opIdexecute_management_function"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}/execute-management-function`

> Body parameter

```json
{
  "viewName": "MyViewName",
  "managementFunction": {
    "function": "CreateVpc"
  }
}
```

<h3 id="execute-a-component's-management-function-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|component_id|path|string|true|Component identifier|
|body|body|[ExecuteManagementFunctionV1Request](#schemaexecutemanagementfunctionv1request)|true|none|

> Example responses

> 200 Response

```json
{
  "funcRunId": "string",
  "managementFuncJobStateId": "01H9ZQD35JPMBGHH69BT0Q79VY",
  "message": "enqueued",
  "status": "Ok"
}
```

<h3 id="execute-a-component's-management-function-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Function successfully dispatched|[ExecuteManagementFunctionV1Response](#schemaexecutemanagementfunctionv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Component or function not found|None|
|412|[Precondition Failed](https://tools.ietf.org/html/rfc7232#section-4.2)|Precondition Failed - View not found or duplicate function name|[ApiError](#schemaapierror)|
|422|[Unprocessable Entity](https://tools.ietf.org/html/rfc2518#section-10.3)|Validation error - Invalid request data|[ApiError](#schemaapierror)|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Putting a component under the management of another component

<a id="opIdmanage_component"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}/manage`

> Body parameter

```json
{
  "componentId": "string"
}
```

<h3 id="putting-a-component-under-the-management-of-another-component-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|component_id|path|string|true|Component identifier|
|body|body|[ManageComponentV1Request](#schemamanagecomponentv1request)|true|none|

> Example responses

> 200 Response

```json
{
  "component": {
    "attributes": {
      "/domain/region": "us-east-1",
      "/secrets/credential": {
        "$source": {
          "component": "demo-credential",
          "path": "/secrets/AWS Credential"
        }
      }
    },
    "canBeUpgraded": true,
    "connections": [
      {
        "incoming": {
          "from": "string",
          "fromComponentId": "string",
          "fromComponentName": "string",
          "to": "string"
        }
      }
    ],
    "domainProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": {}
      }
    ],
    "id": "string",
    "name": "string",
    "resourceId": "string",
    "resourceProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": {}
      }
    ],
    "schemaId": "string",
    "schemaVariantId": "string",
    "sockets": [
      {
        "arity": "one",
        "direction": "input",
        "id": "string",
        "name": "string",
        "value": {}
      }
    ],
    "toDelete": true,
    "views": [
      {
        "id": "string",
        "isDefault": true,
        "name": "string"
      }
    ]
  }
}
```

<h3 id="putting-a-component-under-the-management-of-another-component-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Component successfully under management|[ManageComponentV1Response](#schemamanagecomponentv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Component not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Upgrade a component to the latest schema variant

<a id="opIdupgrade_component"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}/upgrade`

<h3 id="upgrade-a-component-to-the-latest-schema-variant-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|component_id|path|string|true|Component identifier|

> Example responses

> 200 Response

```json
{
  "component": {
    "attributes": {
      "/domain/region": "us-east-1",
      "/secrets/credential": {
        "$source": {
          "component": "demo-credential",
          "path": "/secrets/AWS Credential"
        }
      }
    },
    "canBeUpgraded": true,
    "connections": [
      {
        "incoming": {
          "from": "string",
          "fromComponentId": "string",
          "fromComponentName": "string",
          "to": "string"
        }
      }
    ],
    "domainProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": {}
      }
    ],
    "id": "string",
    "name": "string",
    "resourceId": "string",
    "resourceProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": {}
      }
    ],
    "schemaId": "string",
    "schemaVariantId": "string",
    "sockets": [
      {
        "arity": "one",
        "direction": "input",
        "id": "string",
        "name": "string",
        "value": {}
      }
    ],
    "toDelete": true,
    "views": [
      {
        "id": "string",
        "isDefault": true,
        "name": "string"
      }
    ]
  }
}
```

<h3 id="upgrade-a-component-to-the-latest-schema-variant-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Component successfully upgraded|[UpgradeComponentV1Response](#schemaupgradecomponentv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Component not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

# [schemas](#system-initiative-api-schemas)

Schemas management endpoints

## List all schemas (paginated endpoint)

<a id="opIdlist_schemas"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/schemas`

<h3 id="list-all-schemas-(paginated-endpoint)-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|limit|query|string|false|Maximum number of results to return (default: 50, max: 300)|
|cursor|query|string|false|Cursor for pagination (SchemaId of the last item from previous page)|

> Example responses

> 200 Response

```json
{
  "nextCursor": "string",
  "schemas": [
    {
      "category": "AWS::EC2",
      "installed": "false",
      "schemaId": "01H9ZQD35JPMBGHH69BT0Q79VY",
      "schemaName": "AWS::EC2::Instance"
    }
  ]
}
```

<h3 id="list-all-schemas-(paginated-endpoint)-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Schemas listed successfully|[ListSchemaV1Response](#schemalistschemav1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Find schema by name or schema id

<a id="opIdfind_schema"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/find`

<h3 id="find-schema-by-name-or-schema-id-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|schema|query|string,null|false|none|
|schemaId|query|string,null|false|none|

> Example responses

> 200 Response

```json
{
  "category": "string",
  "installed": true,
  "schemaId": "string",
  "schemaName": "string"
}
```

<h3 id="find-schema-by-name-or-schema-id-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Schema retrieved successfully|[FindSchemaV1Response](#schemafindschemav1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Schema not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Get a schema by schema id

<a id="opIdget_schema"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}`

<h3 id="get-a-schema-by-schema-id-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|schema_id|path|string|true|Schema identifier|

> Example responses

> 200 Response

```json
{
  "defaultVariantId": "01H9ZQD35JPMBGHH69BT0Q79VZ",
  "name": "AWS::EC2::Instance",
  "variantIds": [
    "01H9ZQD35JPMBGHH69BT0Q79VZ",
    "01H9ZQD35JPMBGHH69BT0Q79VY"
  ]
}
```

<h3 id="get-a-schema-by-schema-id-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Schema retrieved successfully|[GetSchemaV1Response](#schemagetschemav1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Schema not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Get the default variant for a schema id

<a id="opIdget_default_variant"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/variant/default`

<h3 id="get-the-default-variant-for-a-schema-id-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|schema_id|path|string|true|Schema identifier|

> Example responses

> 200 Response

```json
{
  "assetFuncId": "01H9ZQD35JPMBGHH69BT0Q75XY",
  "category": "AWS::EC2",
  "color": "#FF5733",
  "description": "Amazon EC2 Instance resource type",
  "displayName": "AWS EC2 Instance",
  "domainProps": {
    "children": [
      {}
    ],
    "description": "string",
    "name": "string",
    "propId": "string",
    "propType": "string"
  },
  "isDefaultVariant": true,
  "isLocked": false,
  "link": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-instance.html",
  "variantFuncIds": [
    "01H9ZQD35JPMBGHH69BT0Q75AA",
    "01H9ZQD35JPMBGHH69BT0Q75BB"
  ],
  "variantId": "01H9ZQD35JPMBGHH69BT0Q79VZ"
}
```

<h3 id="get-the-default-variant-for-a-schema-id-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Schema variant retrieved successfully|[GetSchemaVariantV1Response](#schemagetschemavariantv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Schema variant not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Get a schema variant by schema id and schema variant id

<a id="opIdget_variant"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/variant/{schema_variant_id}`

<h3 id="get-a-schema-variant-by-schema-id-and-schema-variant-id-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|schema_id|path|string|true|Schema identifier|
|schema_variant_id|path|string|true|Schema variant identifier|

> Example responses

> 200 Response

```json
{
  "assetFuncId": "01H9ZQD35JPMBGHH69BT0Q75XY",
  "category": "AWS::EC2",
  "color": "#FF5733",
  "description": "Amazon EC2 Instance resource type",
  "displayName": "AWS EC2 Instance",
  "domainProps": {
    "children": [
      {}
    ],
    "description": "string",
    "name": "string",
    "propId": "string",
    "propType": "string"
  },
  "isDefaultVariant": true,
  "isLocked": false,
  "link": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-instance.html",
  "variantFuncIds": [
    "01H9ZQD35JPMBGHH69BT0Q75AA",
    "01H9ZQD35JPMBGHH69BT0Q75BB"
  ],
  "variantId": "01H9ZQD35JPMBGHH69BT0Q79VZ"
}
```

<h3 id="get-a-schema-variant-by-schema-id-and-schema-variant-id-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Schema variant retrieved successfully|[GetSchemaVariantV1Response](#schemagetschemavariantv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Schema variant not found|None|
|412|[Precondition Failed](https://tools.ietf.org/html/rfc7232#section-4.2)|Schema variant not found for schema|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

# [actions](#system-initiative-api-actions)

Actions management endpoints

## List queued actions

<a id="opIdget_actions"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/actions`

<h3 id="list-queued-actions-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|

> Example responses

> 200 Response

```json
{
  "actions": [
    {
      "id": "01H9ZQD35JPMBGHH69BT0Q79VY",
      "prototypeId": "01H9ZQD35JPMBGHH69BT0Q79AB",
      "componentId": "01H9ZQD35JPMBGHH69BT0Q79CD",
      "name": "Create EC2 Instance",
      "description": "Provisions a new EC2 instance in AWS",
      "kind": "Create",
      "state": "Pending",
      "originatingChangeSetId": "01H9ZQD35JPMBGHH69BT0Q79EF",
      "funcRunId": "01H9ZQD35JPMBGHH69BT0Q79GH"
    }
  ]
}
```

<h3 id="list-queued-actions-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Actions retrieved successfully|[GetActionsV1Response](#schemagetactionsv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Remove queued action

<a id="opIdcancel_action"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/actions/{action_id}/cancel`

<h3 id="remove-queued-action-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|action_id|path|string|true|Action identifier|

> Example responses

> 200 Response

```json
{
  "success": true
}
```

<h3 id="remove-queued-action-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Action cancelled successfully|[CancelActionV1Response](#schemacancelactionv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Action not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Put action on-hold

<a id="opIdput_on_hold"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/actions/{action_id}/put_on_hold`

<h3 id="put-action-on-hold-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|action_id|path|string|true|Action identifier|

> Example responses

> 200 Response

```json
{
  "success": true
}
```

<h3 id="put-action-on-hold-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Action successfully put on hold|[PutOnHoldActionV1Response](#schemaputonholdactionv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Action not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Retry action

<a id="opIdretry_action"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/actions/{action_id}/retry`

<h3 id="retry-action-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|action_id|path|string|true|Action identifier|

> Example responses

> 200 Response

```json
{
  "success": true
}
```

<h3 id="retry-action-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Action successfully requeued|[RetryActionV1Response](#schemaretryactionv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Action not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

# [secrets](#system-initiative-api-secrets)

Secrets management endpoints

## List all secrets

<a id="opIdget_secrets"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/secrets`

<h3 id="list-all-secrets-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|

> Example responses

> 200 Response

```json
{
  "aws_credentials": {
    "definition": {
      "secretDefinition": "aws_credentials",
      "formData": [
        {
          "name": "access_key_id",
          "kind": "string"
        },
        {
          "name": "secret_access_key",
          "kind": "password"
        },
        {
          "name": "region",
          "kind": "string"
        },
        {
          "name": "default_output",
          "kind": "string"
        }
      ]
    },
    "secrets": [
      {
        "id": "01HAXYZF3GC9CYA6ZVSM3E4YHH",
        "name": "Production AWS Key",
        "definition": "aws_credentials",
        "description": "AWS credentials for production environment"
      },
      {
        "id": "01HAXYZF3GC9CYA6ZVSM3E4YHI",
        "name": "Development AWS Key",
        "definition": "aws_credentials",
        "description": "AWS credentials for development environment"
      }
    ]
  },
  "docker_registry": {
    "definition": {
      "secretDefinition": "docker_registry",
      "formData": [
        {
          "name": "username",
          "kind": "string"
        },
        {
          "name": "password",
          "kind": "password"
        },
        {
          "name": "registry_url",
          "kind": "string"
        }
      ]
    },
    "secrets": [
      {
        "id": "01HAXYZF3GC9CYA6ZVSM3E4YHJ",
        "name": "DockerHub Access",
        "definition": "docker_registry",
        "description": "DockerHub registry credentials"
      }
    ]
  }
}
```

> 500 Response

```json
{
  "message": "Invalid request data",
  "statusCode": 422,
  "code": 4001
}
```

<h3 id="list-all-secrets-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Secrets retrieved successfully|[HashMap](#schemahashmap)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Create a secret

<a id="opIdcreate_secret"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/secrets`

> Body parameter

```json
{
  "definitionName": "aws_credentials",
  "description": "AWS credentials for production environment",
  "name": "AWS Access Key",
  "rawData": {
    "access_key_id": "AKIAIOSFODNN7EXAMPLE",
    "secret_access_key": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
    "region": "us-west-2",
    "default_output": "json"
  }
}
```

<h3 id="create-a-secret-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|body|body|[CreateSecretV1Request](#schemacreatesecretv1request)|true|none|

> Example responses

> 200 Response

```json
{
  "secret": {
    "definition": "aws_credentials",
    "description": "AWS credentials for production environment",
    "id": "01HAXYZF3GC9CYA6ZVSM3E4YHH",
    "name": "Production AWS Key"
  }
}
```

<h3 id="create-a-secret-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Secret created successfully|[CreateSecretV1Response](#schemacreatesecretv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|422|[Unprocessable Entity](https://tools.ietf.org/html/rfc2518#section-10.3)|Validation error - Invalid request data|[ApiError](#schemaapierror)|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Update a secret

<a id="opIdupdate_secret"></a>

> Request format

`PUT /v1/w/{workspace_id}/change-sets/{change_set_id}/secrets/{secret_id}`

> Body parameter

```json
{
  "description": "Updated AWS Secret Key for EC2 access",
  "id": "01HAXYZF3GC9CYA6ZVSM3E4YHH",
  "name": "AWS Access Key",
  "rawData": {
    "access_key_id": "AKIAIOSFODNN7EXAMPLE",
    "secret_access_key": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
    "region": "us-west-2",
    "default_output": "json"
  }
}
```

<h3 id="update-a-secret-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|secret_id|path|string|true|Secret identifier|
|body|body|[UpdateSecretV1Request](#schemaupdatesecretv1request)|true|none|

> Example responses

> 200 Response

```json
{
  "secret": {
    "definition": "aws_credentials",
    "description": "AWS credentials for production environment",
    "id": "01HAXYZF3GC9CYA6ZVSM3E4YHH",
    "name": "Production AWS Key"
  }
}
```

<h3 id="update-a-secret-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Secret updated successfully|[UpdateSecretV1Response](#schemaupdatesecretv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Secret not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Delete a secret

<a id="opIddelete_secret"></a>

> Request format

`DELETE /v1/w/{workspace_id}/change-sets/{change_set_id}/secrets/{secret_id}`

<h3 id="delete-a-secret-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|secret_id|path|string|true|Secret identifier|

> Example responses

> 200 Response

```json
{
  "success": true
}
```

<h3 id="delete-a-secret-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Secret deleted successfully|[DeleteSecretV1Response](#schemadeletesecretv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Secret not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

# [funcs](#system-initiative-api-funcs)

Functions management endpoints

## Get func execution run logs

<a id="opIdget_func_run"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/funcs/runs/{func_run_id}`

<h3 id="get-func-execution-run-logs-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|func_run_id|path|string|true|Func run identifier|

> Example responses

> 200 Response

```json
{
  "funcRun": {
    "id": "01JQCJ0AAXGX5M9QY10AVF4GK1",
    "state": "Success",
    "actor": "System",
    "componentId": "01JP8KHZP3DZKGNXRP83Q6WTQ5",
    "attributeValueId": null,
    "componentName": "NAT Gateway IP 1",
    "schemaName": "AWS::EC2::EIP",
    "actionId": "01JQCHZZY99G3R0C1FA3W4AFR6",
    "actionPrototypeId": "01JPNHEE9Z3DFW48XVZ1FX04KA",
    "actionKind": "Destroy",
    "actionDisplayName": "Destroy",
    "actionOriginatingChangeSetId": "01JQCHZZVTAHHZ7DG0ZSCB9RXB",
    "actionOriginatingChangeSetName": "2025-03-27-19:41",
    "actionResultState": "Success",
    "backendKind": "JsAction",
    "backendResponseType": "Action",
    "functionName": "Delete Asset",
    "functionDisplayName": null,
    "functionKind": "Action",
    "functionDescription": null,
    "functionLink": null,
    "functionArgs": {
      "properties": {
        "domain": {
          "Domain": "vpc",
          "Tags": []
        },
        "resource": {
          "payload": {
            "AllocationId": "eipalloc-033720f9556a3b0c1",
            "PublicIp": "3.213.242.163"
          }
        },
        "si": {
          "name": "NAT Gateway IP 1",
          "resourceId": "3.213.242.163|eipalloc-033720f9556a3b0c1",
          "type": "component"
        }
      }
    },
    "resultValue": {
      "error": null,
      "executionId": "01JQCJ0AAXGX5M9QY10AVF4GK1",
      "message": null,
      "payload": null,
      "resourceId": null,
      "status": "ok"
    },
    "logs": {
      "id": "01JQCJ0ABJSCE01GNQDWVY1ZP5",
      "createdAt": "2025-03-27T19:41:58.514416748Z",
      "updatedAt": "2025-03-27T19:41:58.514416748Z",
      "funcRunId": "01JQCJ0AAXGX5M9QY10AVF4GK1",
      "logs": [
        {
          "stream": "stdout",
          "executionId": "",
          "level": "info",
          "group": "log",
          "message": "Running CLI command",
          "timestamp": 1743104518
        },
        {
          "stream": "output",
          "executionId": "01JQCJ0AAXGX5M9QY10AVF4GK1",
          "level": "info",
          "group": "log",
          "message": "Output: {\"status\":\"success\"}",
          "timestamp": 1743104521
        }
      ],
      "finalized": true
    },
    "createdAt": "2025-03-27T19:41:58.493298051Z",
    "updatedAt": "2025-03-27T19:42:02.192033089Z"
  }
}
```

> 500 Response

```json
{
  "message": "Invalid request data",
  "statusCode": 422,
  "code": 4001
}
```

<h3 id="get-func-execution-run-logs-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Func Run retrieved successfully|[GetFuncRunV1Response](#schemagetfuncrunv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Func run not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Get function details

<a id="opIdget_func"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/funcs/{func_id}`

<h3 id="get-function-details-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|func_id|path|string|true|Func identifier|

> Example responses

> 200 Response

```json
[
  {
    "funcId": "01JP8A3S8VDQ1KRQWQRHB1ZEB2",
    "code": "async function main(input: Input): Promise < Output > {\n    if (!input.domain?.region) {\n        return {\n            result: \"failure\",\n            message: \"No Region Name to validate\",\n        };\n    }\n\n    const child = await siExec.waitUntilEnd(\"aws\", [\n        \"ec2\",\n        \"describe-regions\",\n        \"--region-names\",\n        input.domain?.region!,\n        \"--region\",\n        \"us-east-1\",\n    ]);\n\n    if (child.exitCode !== 0) {\n        console.error(child.stderr);\n        return {\n            result: \"failure\",\n            message: \"Error from API\"\n        }\n    }\n\n    const regionDetails = JSON.parse(child.stdout).Regions;\n    if (regionDetails.length === 0 || regionDetails.length > 1) {\n        return {\n            result: \"failure\",\n            message: \"Unable to find Region\"\n        }\n    }\n\n    if (regionDetails[0].OptInStatus === \"not-opted-in\") {\n        return {\n            result: \"failure\",\n            message: \"Region not-opted-in for use\"\n        }\n    }\n\n    return {\n        result: \"success\",\n        message: \"Region is available to use\",\n    };\n}",
    "name": "AWS Region Validator",
    "description": "Validates if an AWS region exists and is available for use",
    "displayName": "Validate Region",
    "kind": "Qualification",
    "isLocked": false,
    "link": "https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeRegions.html"
  }
]
```

> 500 Response

```json
{
  "message": "Invalid request data",
  "statusCode": 422,
  "code": 4001
}
```

<h3 id="get-function-details-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Func retrieved successfully|[GetFuncV1Response](#schemagetfuncv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Func not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

# [management_funcs](#system-initiative-api-management_funcs)

Management functions endpoints

## Get management funcs job state details

<a id="opIdget_management_func_run_state"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/management-funcs/{management_func_job_state_id}`

<h3 id="get-management-funcs-job-state-details-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|management_func_job_state_id|path|string|true|Management Func Job identifier|

> Example responses

> 200 Response

```json
{
  "funcRunId": "01H9ZQD35JPMBGHH69BT0Q79VY",
  "state": "Executing"
}
```

<h3 id="get-management-funcs-job-state-details-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Management Func Job retrieved successfully|[GetManagementFuncJobStateV1Response](#schemagetmanagementfuncjobstatev1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Management Func Job not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

# [Schemas](#schemas)

## [ActionReference](#tocS_ActionReference)

<a id="schemaactionreference"></a>
<a id="schema_ActionReference"></a>
<a id="tocSactionreference"></a>
<a id="tocsactionreference"></a>

```json
{
  "function": "Create Asset"
}

```

Reference to a management function by either name or ID.
This allows clients to use the more human-friendly name approach
or the more precise ID approach when working with actions.

### [Properties](#actionreference-properties)

oneOf

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|object|false|none|none|
|» function|string|true|none|none|

xor

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|object|false|none|none|
|» actionPrototypeId|string|true|none|none|

## [ActionV1RequestPath](#tocS_ActionV1RequestPath)

<a id="schemaactionv1requestpath"></a>
<a id="schema_ActionV1RequestPath"></a>
<a id="tocSactionv1requestpath"></a>
<a id="tocsactionv1requestpath"></a>

```json
{
  "action_id": "string"
}

```

### [Properties](#actionv1requestpath-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|action_id|string|true|none|none|

## [ActionViewV1](#tocS_ActionViewV1)

<a id="schemaactionviewv1"></a>
<a id="schema_ActionViewV1"></a>
<a id="tocSactionviewv1"></a>
<a id="tocsactionviewv1"></a>

```json
{
  "componentId": "string",
  "description": "string",
  "funcRunId": "string",
  "id": "string",
  "kind": "string",
  "name": "string",
  "originatingChangeSetId": "string",
  "prototypeId": "string",
  "state": "string"
}

```

### [Properties](#actionviewv1-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|componentId|string|true|none|none|
|description|string|true|none|none|
|funcRunId|string|true|none|none|
|id|string|true|none|none|
|kind|string|true|none|none|
|name|string|true|none|none|
|originatingChangeSetId|string|true|none|none|
|prototypeId|string|true|none|none|
|state|string|true|none|none|

## [AddActionV1Request](#tocS_AddActionV1Request)

<a id="schemaaddactionv1request"></a>
<a id="schema_AddActionV1Request"></a>
<a id="tocSaddactionv1request"></a>
<a id="tocsaddactionv1request"></a>

```json
{
  "action": {
    "function": "Create Asset"
  }
}

```

### [Properties](#addactionv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|action|[ActionReference](#schemaactionreference)|true|none|Reference to a management function by either name or ID.<br>This allows clients to use the more human-friendly name approach<br>or the more precise ID approach when working with actions.|

## [AddActionV1Response](#tocS_AddActionV1Response)

<a id="schemaaddactionv1response"></a>
<a id="schema_AddActionV1Response"></a>
<a id="tocSaddactionv1response"></a>
<a id="tocsaddactionv1response"></a>

```json
{
  "success": true
}

```

### [Properties](#addactionv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|success|boolean|true|none|none|

## [ApiError](#tocS_ApiError)

<a id="schemaapierror"></a>
<a id="schema_ApiError"></a>
<a id="tocSapierror"></a>
<a id="tocsapierror"></a>

```json
{
  "message": "Invalid request data",
  "statusCode": 422,
  "code": 4001
}

```

Standard error response format for v1 API

### [Properties](#apierror-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|code|integer,null(int32)|false|none|none|
|message|string|true|none|none|
|statusCode|integer(int32)|true|none|none|

## [ApiSuccess_String](#tocS_ApiSuccess_String)

<a id="schemaapisuccess_string"></a>
<a id="schema_ApiSuccess_String"></a>
<a id="tocSapisuccess_string"></a>
<a id="tocsapisuccess_string"></a>

```json
{
  "data": "string"
}

```

Standard success response format for v1 API

### [Properties](#apisuccess_string-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|data|string|true|none|none|

## [CancelActionV1Response](#tocS_CancelActionV1Response)

<a id="schemacancelactionv1response"></a>
<a id="schema_CancelActionV1Response"></a>
<a id="tocScancelactionv1response"></a>
<a id="tocscancelactionv1response"></a>

```json
{
  "success": true
}

```

### [Properties](#cancelactionv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|success|boolean|true|none|none|

## [ChangeSetViewV1](#tocS_ChangeSetViewV1)

<a id="schemachangesetviewv1"></a>
<a id="schema_ChangeSetViewV1"></a>
<a id="tocSchangesetviewv1"></a>
<a id="tocschangesetviewv1"></a>

```json
{
  "id": "string",
  "isHead": true,
  "name": "string",
  "status": "string"
}

```

### [Properties](#changesetviewv1-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|id|string|true|none|none|
|isHead|boolean|true|none|none|
|name|string|true|none|none|
|status|string|true|none|none|

## [ComponentDetailsV1](#tocS_ComponentDetailsV1)

<a id="schemacomponentdetailsv1"></a>
<a id="schema_ComponentDetailsV1"></a>
<a id="tocScomponentdetailsv1"></a>
<a id="tocscomponentdetailsv1"></a>

```json
{
  "componentId": "string",
  "name": "string",
  "schemaName": "string"
}

```

### [Properties](#componentdetailsv1-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|componentId|string|true|none|none|
|name|string|true|none|none|
|schemaName|string|true|none|none|

## [ComponentPropKey](#tocS_ComponentPropKey)

<a id="schemacomponentpropkey"></a>
<a id="schema_ComponentPropKey"></a>
<a id="tocScomponentpropkey"></a>
<a id="tocscomponentpropkey"></a>

```json
"string"

```

### [Properties](#componentpropkey-properties)

oneOf

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|string|false|none|none|

xor

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|[DomainPropPath](#schemadomainproppath)|false|none|A prop path, starting from root/domain, with / instead of PROP_PATH_SEPARATOR as its separator|

## [ComponentPropViewV1](#tocS_ComponentPropViewV1)

<a id="schemacomponentpropviewv1"></a>
<a id="schema_ComponentPropViewV1"></a>
<a id="tocScomponentpropviewv1"></a>
<a id="tocscomponentpropviewv1"></a>

```json
{
  "id": "string",
  "path": "path/to/prop",
  "propId": "string",
  "value": {}
}

```

### [Properties](#componentpropviewv1-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|id|string|true|none|none|
|path|string|true|none|none|
|propId|string|true|none|none|
|value|object|true|none|none|

## [ComponentReference](#tocS_ComponentReference)

<a id="schemacomponentreference"></a>
<a id="schema_ComponentReference"></a>
<a id="tocScomponentreference"></a>
<a id="tocscomponentreference"></a>

```json
{
  "component": "ComponentName"
}

```

### [Properties](#componentreference-properties)

oneOf

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|object|false|none|none|
|» component|string|true|none|none|

xor

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|object|false|none|none|
|» componentId|string|true|none|none|

## [ComponentV1RequestPath](#tocS_ComponentV1RequestPath)

<a id="schemacomponentv1requestpath"></a>
<a id="schema_ComponentV1RequestPath"></a>
<a id="tocScomponentv1requestpath"></a>
<a id="tocscomponentv1requestpath"></a>

```json
{
  "component_id": "string"
}

```

### [Properties](#componentv1requestpath-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|component_id|string|true|none|none|

## [ComponentViewV1](#tocS_ComponentViewV1)

<a id="schemacomponentviewv1"></a>
<a id="schema_ComponentViewV1"></a>
<a id="tocScomponentviewv1"></a>
<a id="tocscomponentviewv1"></a>

```json
{
  "attributes": {
    "/domain/region": "us-east-1",
    "/secrets/credential": {
      "$source": {
        "component": "demo-credential",
        "path": "/secrets/AWS Credential"
      }
    }
  },
  "canBeUpgraded": true,
  "connections": [
    {
      "incoming": {
        "from": "string",
        "fromComponentId": "string",
        "fromComponentName": "string",
        "to": "string"
      }
    }
  ],
  "domainProps": [
    {
      "id": "string",
      "path": "path/to/prop",
      "propId": "string",
      "value": {}
    }
  ],
  "id": "string",
  "name": "string",
  "resourceId": "string",
  "resourceProps": [
    {
      "id": "string",
      "path": "path/to/prop",
      "propId": "string",
      "value": {}
    }
  ],
  "schemaId": "string",
  "schemaVariantId": "string",
  "sockets": [
    {
      "arity": "one",
      "direction": "input",
      "id": "string",
      "name": "string",
      "value": {}
    }
  ],
  "toDelete": true,
  "views": [
    {
      "id": "string",
      "isDefault": true,
      "name": "string"
    }
  ]
}

```

### [Properties](#componentviewv1-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|attributes|object|true|none|none|
|» **additionalProperties**|any|false|none|none|
|canBeUpgraded|boolean|true|none|none|
|connections|[[ConnectionViewV1](#schemaconnectionviewv1)]|true|none|none|
|domainProps|[[ComponentPropViewV1](#schemacomponentpropviewv1)]|true|none|none|
|id|string|true|none|none|
|name|string|true|none|none|
|resourceId|string|true|none|none|
|resourceProps|[[ComponentPropViewV1](#schemacomponentpropviewv1)]|true|none|none|
|schemaId|string|true|none|none|
|schemaVariantId|string|true|none|none|
|sockets|[[SocketViewV1](#schemasocketviewv1)]|true|none|none|
|toDelete|boolean|true|none|none|
|views|[[ViewV1](#schemaviewv1)]|true|none|none|

## [Connection](#tocS_Connection)

<a id="schemaconnection"></a>
<a id="schema_Connection"></a>
<a id="tocSconnection"></a>
<a id="tocsconnection"></a>

```json
{
  "from": {
    "componentId": "01H9ZQD35JPMBGHH69BT0Q79VY",
    "socketName": "output"
  },
  "to": "ThisComponentInputSocketName"
}

```

### [Properties](#connection-properties)

oneOf

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|object|false|none|none|
|» from|[ConnectionPoint](#schemaconnectionpoint)|true|none|none|
|» to|string|true|none|none|

xor

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|object|false|none|none|
|» from|string|true|none|none|
|» to|[ConnectionPoint](#schemaconnectionpoint)|true|none|none|

## [ConnectionDetails](#tocS_ConnectionDetails)

<a id="schemaconnectiondetails"></a>
<a id="schema_ConnectionDetails"></a>
<a id="tocSconnectiondetails"></a>
<a id="tocsconnectiondetails"></a>

```json
{
  "add": {},
  "remove": {}
}

```

### [Properties](#connectiondetails-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|add|[[Connection](#schemaconnection)]|false|none|none|
|remove|[[Connection](#schemaconnection)]|false|none|none|

## [ConnectionPoint](#tocS_ConnectionPoint)

<a id="schemaconnectionpoint"></a>
<a id="schema_ConnectionPoint"></a>
<a id="tocSconnectionpoint"></a>
<a id="tocsconnectionpoint"></a>

```json
{
  "componentId": "01H9ZQD35JPMBGHH69BT0Q79VY",
  "socketName": "OutputSocketName"
}

```

### [Properties](#connectionpoint-properties)

allOf

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|[ComponentReference](#schemacomponentreference)|false|none|none|

and

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|object|false|none|none|
|» socketName|string|true|none|none|

## [ConnectionViewV1](#tocS_ConnectionViewV1)

<a id="schemaconnectionviewv1"></a>
<a id="schema_ConnectionViewV1"></a>
<a id="tocSconnectionviewv1"></a>
<a id="tocsconnectionviewv1"></a>

```json
{
  "incoming": {
    "from": "string",
    "fromComponentId": "string",
    "fromComponentName": "string",
    "to": "string"
  }
}

```

### [Properties](#connectionviewv1-properties)

oneOf

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|object|false|none|none|
|» incoming|[IncomingConnectionViewV1](#schemaincomingconnectionviewv1)|true|none|none|

xor

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|object|false|none|none|
|» outgoing|[OutgoingConnectionViewV1](#schemaoutgoingconnectionviewv1)|true|none|none|

xor

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|object|false|none|none|
|» managing|[ManagingConnectionViewV1](#schemamanagingconnectionviewv1)|true|none|none|

xor

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|object|false|none|none|
|» managedBy|[ManagedByConnectionViewV1](#schemamanagedbyconnectionviewv1)|true|none|none|

## [CreateChangeSetV1Request](#tocS_CreateChangeSetV1Request)

<a id="schemacreatechangesetv1request"></a>
<a id="schema_CreateChangeSetV1Request"></a>
<a id="tocScreatechangesetv1request"></a>
<a id="tocscreatechangesetv1request"></a>

```json
{
  "changeSetName": "My new feature"
}

```

### [Properties](#createchangesetv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|changeSetName|string|true|none|none|

## [CreateChangeSetV1Response](#tocS_CreateChangeSetV1Response)

<a id="schemacreatechangesetv1response"></a>
<a id="schema_CreateChangeSetV1Response"></a>
<a id="tocScreatechangesetv1response"></a>
<a id="tocscreatechangesetv1response"></a>

```json
{
  "changeSet": {
    "id": "string",
    "isHead": true,
    "name": "string",
    "status": "string"
  }
}

```

### [Properties](#createchangesetv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|changeSet|[ChangeSetViewV1](#schemachangesetviewv1)|true|none|none|

## [CreateComponentV1Request](#tocS_CreateComponentV1Request)

<a id="schemacreatecomponentv1request"></a>
<a id="schema_CreateComponentV1Request"></a>
<a id="tocScreatecomponentv1request"></a>
<a id="tocscreatecomponentv1request"></a>

```json
{
  "attributes": {
    "/domain/VpcId": {
      "$source": {
        "component": "01K0WRC69ZPEMD6SMTKC84FBWC",
        "path": "/resource_value/VpcId"
      }
    },
    "/domain/SubnetId": {
      "$source": {
        "component": "01K0WRC69ZPEMD6SMTKC84FBWD",
        "path": "/resource_value/SubnetId"
      }
    },
    "/domain/Version": {
      "$source": null
    }
  },
  "connections": {},
  "domain": {},
  "managedBy": {
    "component": "ComponentName"
  },
  "name": "MyComponentName",
  "resourceId": "i-12345678",
  "schemaName": "AWS::EC2::Instance",
  "secrets": {},
  "subscriptions": {},
  "viewName": "MyView"
}

```

### [Properties](#createcomponentv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|attributes|object|false|none|none|
|» **additionalProperties**|any|false|none|none|
|connections|[[Connection](#schemaconnection)]|false|none|none|
|domain|object|false|none|none|
|» **additionalProperties**|any|false|none|none|
|managedBy|[ComponentReference](#schemacomponentreference)|false|none|none|
|name|string|true|none|none|
|resourceId|string,null|false|none|none|
|schemaName|string|true|none|none|
|secrets|object|false|none|none|
|» **additionalProperties**|any|false|none|none|
|subscriptions|object|false|none|none|
|» **additionalProperties**|[Subscription](#schemasubscription)|false|none|none|
|viewName|string,null|false|none|none|

## [CreateComponentV1Response](#tocS_CreateComponentV1Response)

<a id="schemacreatecomponentv1response"></a>
<a id="schema_CreateComponentV1Response"></a>
<a id="tocScreatecomponentv1response"></a>
<a id="tocscreatecomponentv1response"></a>

```json
{
  "component": {
    "attributes": {
      "/domain/region": "us-east-1",
      "/secrets/credential": {
        "$source": {
          "component": "demo-credential",
          "path": "/secrets/AWS Credential"
        }
      }
    },
    "canBeUpgraded": true,
    "connections": [
      {
        "incoming": {
          "from": "string",
          "fromComponentId": "string",
          "fromComponentName": "string",
          "to": "string"
        }
      }
    ],
    "domainProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": {}
      }
    ],
    "id": "string",
    "name": "string",
    "resourceId": "string",
    "resourceProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": {}
      }
    ],
    "schemaId": "string",
    "schemaVariantId": "string",
    "sockets": [
      {
        "arity": "one",
        "direction": "input",
        "id": "string",
        "name": "string",
        "value": {}
      }
    ],
    "toDelete": true,
    "views": [
      {
        "id": "string",
        "isDefault": true,
        "name": "string"
      }
    ]
  }
}

```

### [Properties](#createcomponentv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|component|[ComponentViewV1](#schemacomponentviewv1)|true|none|none|

## [CreateSecretV1Request](#tocS_CreateSecretV1Request)

<a id="schemacreatesecretv1request"></a>
<a id="schema_CreateSecretV1Request"></a>
<a id="tocScreatesecretv1request"></a>
<a id="tocscreatesecretv1request"></a>

```json
{
  "definitionName": "aws_credentials",
  "description": "AWS credentials for production environment",
  "name": "AWS Access Key",
  "rawData": {
    "access_key_id": "AKIAIOSFODNN7EXAMPLE",
    "secret_access_key": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
    "region": "us-west-2",
    "default_output": "json"
  }
}

```

### [Properties](#createsecretv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|definitionName|string|true|none|none|
|description|string|true|none|none|
|name|string|true|none|none|
|rawData|object|false|none|none|
|» **additionalProperties**|string|false|none|none|

## [CreateSecretV1Response](#tocS_CreateSecretV1Response)

<a id="schemacreatesecretv1response"></a>
<a id="schema_CreateSecretV1Response"></a>
<a id="tocScreatesecretv1response"></a>
<a id="tocscreatesecretv1response"></a>

```json
{
  "secret": {
    "definition": "aws_credentials",
    "description": "AWS credentials for production environment",
    "id": "01HAXYZF3GC9CYA6ZVSM3E4YHH",
    "name": "Production AWS Key"
  }
}

```

### [Properties](#createsecretv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|secret|[SecretV1](#schemasecretv1)|true|none|none|

## [DeleteChangeSetV1Response](#tocS_DeleteChangeSetV1Response)

<a id="schemadeletechangesetv1response"></a>
<a id="schema_DeleteChangeSetV1Response"></a>
<a id="tocSdeletechangesetv1response"></a>
<a id="tocsdeletechangesetv1response"></a>

```json
{
  "success": "true"
}

```

### [Properties](#deletechangesetv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|success|boolean|true|none|none|

## [DeleteComponentV1Response](#tocS_DeleteComponentV1Response)

<a id="schemadeletecomponentv1response"></a>
<a id="schema_DeleteComponentV1Response"></a>
<a id="tocSdeletecomponentv1response"></a>
<a id="tocsdeletecomponentv1response"></a>

```json
{
  "status": "MarkedForDeletion"
}

```

### [Properties](#deletecomponentv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|status|string|true|none|none|

## [DeleteSecretV1Response](#tocS_DeleteSecretV1Response)

<a id="schemadeletesecretv1response"></a>
<a id="schema_DeleteSecretV1Response"></a>
<a id="tocSdeletesecretv1response"></a>
<a id="tocsdeletesecretv1response"></a>

```json
{
  "success": true
}

```

### [Properties](#deletesecretv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|success|boolean|true|none|none|

## [DomainPropPath](#tocS_DomainPropPath)

<a id="schemadomainproppath"></a>
<a id="schema_DomainPropPath"></a>
<a id="tocSdomainproppath"></a>
<a id="tocsdomainproppath"></a>

```json
"string"

```

A prop path, starting from root/domain, with / instead of PROP_PATH_SEPARATOR as its separator

### [Properties](#domainproppath-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|string|false|none|A prop path, starting from root/domain, with / instead of PROP_PATH_SEPARATOR as its separator|

## [DuplicateComponentsV1Request](#tocS_DuplicateComponentsV1Request)

<a id="schemaduplicatecomponentsv1request"></a>
<a id="schema_DuplicateComponentsV1Request"></a>
<a id="tocSduplicatecomponentsv1request"></a>
<a id="tocsduplicatecomponentsv1request"></a>

```json
{
  "components": [
    "01H9ZQD35JPMBGHH69BT0Q79AA",
    "01H9ZQD35JPMBGHH69BT0Q79BB",
    "01H9ZQD35JPMBGHH69BT0Q79CC"
  ],
  "prefix": "copy-of-",
  "viewName": "MyView"
}

```

### [Properties](#duplicatecomponentsv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|components|[array]|true|none|none|
|prefix|string,null|false|none|none|
|viewName|string,null|false|none|none|

## [DuplicateComponentsV1Response](#tocS_DuplicateComponentsV1Response)

<a id="schemaduplicatecomponentsv1response"></a>
<a id="schema_DuplicateComponentsV1Response"></a>
<a id="tocSduplicatecomponentsv1response"></a>
<a id="tocsduplicatecomponentsv1response"></a>

```json
{
  "components": [
    "01H9ZQD35JPMBGHH69BT0Q79AA",
    "01H9ZQD35JPMBGHH69BT0Q79BB",
    "01H9ZQD35JPMBGHH69BT0Q79CC"
  ]
}

```

### [Properties](#duplicatecomponentsv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|components|[array]|true|none|none|

## [ErrorDetail](#tocS_ErrorDetail)

<a id="schemaerrordetail"></a>
<a id="schema_ErrorDetail"></a>
<a id="tocSerrordetail"></a>
<a id="tocserrordetail"></a>

```json
{
  "code": 0,
  "message": "string",
  "status_code": 0
}

```

### [Properties](#errordetail-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|code|integer(int32)|true|none|none|
|message|string|true|none|none|
|status_code|integer(int32)|true|none|none|

## [ErrorResponse](#tocS_ErrorResponse)

<a id="schemaerrorresponse"></a>
<a id="schema_ErrorResponse"></a>
<a id="tocSerrorresponse"></a>
<a id="tocserrorresponse"></a>

```json
{
  "error": {
    "code": 0,
    "message": "string",
    "status_code": 0
  }
}

```

### [Properties](#errorresponse-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|error|[ErrorDetail](#schemaerrordetail)|true|none|none|

## [ExecuteManagementFunctionV1Request](#tocS_ExecuteManagementFunctionV1Request)

<a id="schemaexecutemanagementfunctionv1request"></a>
<a id="schema_ExecuteManagementFunctionV1Request"></a>
<a id="tocSexecutemanagementfunctionv1request"></a>
<a id="tocsexecutemanagementfunctionv1request"></a>

```json
{
  "viewName": "MyViewName",
  "managementFunction": {
    "function": "CreateVpc"
  }
}

```

### [Properties](#executemanagementfunctionv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|managementFunction|[ManagementFunctionReference](#schemamanagementfunctionreference)|true|none|Reference to a management function by either name or ID.<br>This allows clients to use the more human-friendly name approach<br>or the more precise ID approach when working with management functions.|
|viewName|string,null|false|none|none|

## [ExecuteManagementFunctionV1Response](#tocS_ExecuteManagementFunctionV1Response)

<a id="schemaexecutemanagementfunctionv1response"></a>
<a id="schema_ExecuteManagementFunctionV1Response"></a>
<a id="tocSexecutemanagementfunctionv1response"></a>
<a id="tocsexecutemanagementfunctionv1response"></a>

```json
{
  "funcRunId": "string",
  "managementFuncJobStateId": "01H9ZQD35JPMBGHH69BT0Q79VY",
  "message": "enqueued",
  "status": "Ok"
}

```

### [Properties](#executemanagementfunctionv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|funcRunId|string|true|none|none|
|managementFuncJobStateId|string|true|none|none|
|message|string|false|none|none|
|status|string|true|none|none|

## [FindComponentV1Params](#tocS_FindComponentV1Params)

<a id="schemafindcomponentv1params"></a>
<a id="schema_FindComponentV1Params"></a>
<a id="tocSfindcomponentv1params"></a>
<a id="tocsfindcomponentv1params"></a>

```json
{
  "component": "string",
  "componentId": "string"
}

```

### [Properties](#findcomponentv1params-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|component|string,null|false|none|none|
|componentId|string|true|none|none|

## [FindSchemaV1Params](#tocS_FindSchemaV1Params)

<a id="schemafindschemav1params"></a>
<a id="schema_FindSchemaV1Params"></a>
<a id="tocSfindschemav1params"></a>
<a id="tocsfindschemav1params"></a>

```json
{
  "schema": "string",
  "schemaId": "string"
}

```

### [Properties](#findschemav1params-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|schema|string,null|false|none|none|
|schemaId|string|true|none|none|

## [FindSchemaV1Response](#tocS_FindSchemaV1Response)

<a id="schemafindschemav1response"></a>
<a id="schema_FindSchemaV1Response"></a>
<a id="tocSfindschemav1response"></a>
<a id="tocsfindschemav1response"></a>

```json
{
  "category": "string",
  "installed": true,
  "schemaId": "string",
  "schemaName": "string"
}

```

### [Properties](#findschemav1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|category|string|true|none|none|
|installed|boolean|true|none|none|
|schemaId|string|true|none|none|
|schemaName|string|true|none|none|

## [ForceApplyChangeSetV1Response](#tocS_ForceApplyChangeSetV1Response)

<a id="schemaforceapplychangesetv1response"></a>
<a id="schema_ForceApplyChangeSetV1Response"></a>
<a id="tocSforceapplychangesetv1response"></a>
<a id="tocsforceapplychangesetv1response"></a>

```json
{
  "success": "true"
}

```

### [Properties](#forceapplychangesetv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|success|boolean|true|none|none|

## [FuncRunLogViewV1](#tocS_FuncRunLogViewV1)

<a id="schemafuncrunlogviewv1"></a>
<a id="schema_FuncRunLogViewV1"></a>
<a id="tocSfuncrunlogviewv1"></a>
<a id="tocsfuncrunlogviewv1"></a>

```json
{
  "createdAt": "2025-03-27T19:41:58.514416748Z",
  "finalized": true,
  "funcRunId": "01JQCJ0AAXGX5M9QY10AVF4GK1",
  "id": "01JQCJ0ABJSCE01GNQDWVY1ZP5",
  "logs": [
    {
      "stream": "stdout",
      "executionId": "",
      "level": "info",
      "group": "log",
      "message": "Running CLI command: \"aws 'cloudcontrol' 'delete-resource'\"",
      "timestamp": 1743104518
    },
    {
      "stream": "output",
      "executionId": "01JQCJ0AAXGX5M9QY10AVF4GK1",
      "level": "info",
      "group": "log",
      "message": "Output: {\"protocol\":\"result\",\"status\":\"success\"}",
      "timestamp": 1743104521
    }
  ],
  "updatedAt": "2025-03-27T19:41:58.514416748Z"
}

```

### [Properties](#funcrunlogviewv1-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|createdAt|string|true|none|none|
|finalized|boolean|true|none|none|
|funcRunId|string|true|none|none|
|id|string|true|none|none|
|logs|[object]|true|none|none|
|updatedAt|string|true|none|none|

## [FuncRunV1RequestPath](#tocS_FuncRunV1RequestPath)

<a id="schemafuncrunv1requestpath"></a>
<a id="schema_FuncRunV1RequestPath"></a>
<a id="tocSfuncrunv1requestpath"></a>
<a id="tocsfuncrunv1requestpath"></a>

```json
{
  "func_run_id": "string"
}

```

### [Properties](#funcrunv1requestpath-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|func_run_id|string|true|none|none|

## [FuncRunViewV1](#tocS_FuncRunViewV1)

<a id="schemafuncrunviewv1"></a>
<a id="schema_FuncRunViewV1"></a>
<a id="tocSfuncrunviewv1"></a>
<a id="tocsfuncrunviewv1"></a>

```json
{
  "actionDisplayName": "Destroy",
  "actionId": "01JQCHZZY99G3R0C1FA3W4AFR6",
  "actionKind": "Destroy",
  "actionOriginatingChangeSetId": "01JQCHZZVTAHHZ7DG0ZSCB9RXB",
  "actionOriginatingChangeSetName": "2025-03-27-19:41",
  "actionPrototypeId": "01JPNHEE9Z3DFW48XVZ1FX04KA",
  "actionResultState": "Success",
  "attributeValueId": "null",
  "backendKind": "JsAction",
  "backendResponseType": "Action",
  "componentId": "01JP8KHZP3DZKGNXRP83Q6WTQ5",
  "componentName": "NAT Gateway IP 1",
  "createdAt": "2025-03-27T19:41:58.493298051Z",
  "functionArgs": null,
  "functionCodeBase64": "YXN5bmMgZnVuY3Rpb24gbWFpbihjb21wb2...",
  "functionDescription": "null",
  "functionDisplayName": "null",
  "functionKind": "Action",
  "functionLink": "null",
  "functionName": "Delete Asset",
  "id": "01JQCJ0AAXGX5M9QY10AVF4GK1",
  "logs": {},
  "resultValue": null,
  "schemaName": "AWS::EC2::EIP",
  "state": "Success",
  "updatedAt": "2025-03-27T19:42:02.192033089Z"
}

```

### [Properties](#funcrunviewv1-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|actionDisplayName|string|true|none|none|
|actionId|string|true|none|none|
|actionKind|string|true|none|none|
|actionOriginatingChangeSetId|string|true|none|none|
|actionOriginatingChangeSetName|string|true|none|none|
|actionPrototypeId|string|true|none|none|
|actionResultState|string|true|none|none|
|attributeValueId|string|true|none|none|
|backendKind|string|true|none|none|
|backendResponseType|string|true|none|none|
|componentId|string|true|none|none|
|componentName|string|true|none|none|
|createdAt|string|true|none|none|
|functionArgs|any|true|none|none|
|functionCodeBase64|string|true|none|none|
|functionDescription|string|true|none|none|
|functionDisplayName|string|true|none|none|
|functionKind|string|true|none|none|
|functionLink|string|true|none|none|
|functionName|string|true|none|none|
|id|string|true|none|none|
|logs|any|false|none|none|

oneOf

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|» *anonymous*|null|false|none|none|

xor

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|» *anonymous*|[FuncRunLogViewV1](#schemafuncrunlogviewv1)|false|none|none|

continued

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|resultValue|any|false|none|none|
|schemaName|string|true|none|none|
|state|string|true|none|none|
|updatedAt|string|true|none|none|

## [GetActionsV1Response](#tocS_GetActionsV1Response)

<a id="schemagetactionsv1response"></a>
<a id="schema_GetActionsV1Response"></a>
<a id="tocSgetactionsv1response"></a>
<a id="tocsgetactionsv1response"></a>

```json
{
  "actions": [
    {
      "id": "01H9ZQD35JPMBGHH69BT0Q79VY",
      "prototypeId": "01H9ZQD35JPMBGHH69BT0Q79AB",
      "componentId": "01H9ZQD35JPMBGHH69BT0Q79CD",
      "name": "Create EC2 Instance",
      "description": "Provisions a new EC2 instance in AWS",
      "kind": "Create",
      "state": "Pending",
      "originatingChangeSetId": "01H9ZQD35JPMBGHH69BT0Q79EF",
      "funcRunId": "01H9ZQD35JPMBGHH69BT0Q79GH"
    }
  ]
}

```

### [Properties](#getactionsv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|actions|[[ActionViewV1](#schemaactionviewv1)]|true|none|none|

## [GetChangeSetV1Response](#tocS_GetChangeSetV1Response)

<a id="schemagetchangesetv1response"></a>
<a id="schema_GetChangeSetV1Response"></a>
<a id="tocSgetchangesetv1response"></a>
<a id="tocsgetchangesetv1response"></a>

```json
{
  "changeSet": {
    "id": "string",
    "isHead": true,
    "name": "string",
    "status": "string"
  }
}

```

### [Properties](#getchangesetv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|changeSet|[ChangeSetViewV1](#schemachangesetviewv1)|true|none|none|

## [GetComponentV1Response](#tocS_GetComponentV1Response)

<a id="schemagetcomponentv1response"></a>
<a id="schema_GetComponentV1Response"></a>
<a id="tocSgetcomponentv1response"></a>
<a id="tocsgetcomponentv1response"></a>

```json
{
  "actionFunctions": [
    {
      "prototypeId": "01HAXYZF3GC9CYA6ZVSM3E4YGG",
      "funcName": "Terminate Instance"
    }
  ],
  "component": {
    "attributes": {
      "/domain/region": "us-east-1",
      "/secrets/credential": {
        "$source": {
          "component": "demo-credential",
          "path": "/secrets/AWS Credential"
        }
      }
    },
    "canBeUpgraded": true,
    "connections": [
      {
        "incoming": {
          "from": "string",
          "fromComponentId": "string",
          "fromComponentName": "string",
          "to": "string"
        }
      }
    ],
    "domainProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": {}
      }
    ],
    "id": "string",
    "name": "string",
    "resourceId": "string",
    "resourceProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": {}
      }
    ],
    "schemaId": "string",
    "schemaVariantId": "string",
    "sockets": [
      {
        "arity": "one",
        "direction": "input",
        "id": "string",
        "name": "string",
        "value": {}
      }
    ],
    "toDelete": true,
    "views": [
      {
        "id": "string",
        "isDefault": true,
        "name": "string"
      }
    ]
  },
  "managementFunctions": [
    {
      "managementPrototypeId": "01HAXYZF3GC9CYA6ZVSM3E4YFF",
      "funcName": "Start Instance"
    }
  ]
}

```

### [Properties](#getcomponentv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|actionFunctions|[[GetComponentV1ResponseActionFunction](#schemagetcomponentv1responseactionfunction)]|true|none|none|
|component|[ComponentViewV1](#schemacomponentviewv1)|true|none|none|
|managementFunctions|[[GetComponentV1ResponseManagementFunction](#schemagetcomponentv1responsemanagementfunction)]|true|none|none|

## [GetComponentV1ResponseActionFunction](#tocS_GetComponentV1ResponseActionFunction)

<a id="schemagetcomponentv1responseactionfunction"></a>
<a id="schema_GetComponentV1ResponseActionFunction"></a>
<a id="tocSgetcomponentv1responseactionfunction"></a>
<a id="tocsgetcomponentv1responseactionfunction"></a>

```json
{
  "funcName": "Terminate Instance",
  "prototypeId": "01HAXYZF3GC9CYA6ZVSM3E4YGG"
}

```

### [Properties](#getcomponentv1responseactionfunction-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|funcName|string|true|none|none|
|prototypeId|string|true|none|none|

## [GetComponentV1ResponseManagementFunction](#tocS_GetComponentV1ResponseManagementFunction)

<a id="schemagetcomponentv1responsemanagementfunction"></a>
<a id="schema_GetComponentV1ResponseManagementFunction"></a>
<a id="tocSgetcomponentv1responsemanagementfunction"></a>
<a id="tocsgetcomponentv1responsemanagementfunction"></a>

```json
{
  "funcName": "Start Instance",
  "managementPrototypeId": "01HAXYZF3GC9CYA6ZVSM3E4YFF"
}

```

### [Properties](#getcomponentv1responsemanagementfunction-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|funcName|string|true|none|none|
|managementPrototypeId|string|true|none|none|

## [GetFuncRunV1Response](#tocS_GetFuncRunV1Response)

<a id="schemagetfuncrunv1response"></a>
<a id="schema_GetFuncRunV1Response"></a>
<a id="tocSgetfuncrunv1response"></a>
<a id="tocsgetfuncrunv1response"></a>

```json
{
  "funcRun": {
    "actionDisplayName": "Destroy",
    "actionId": "01JQCHZZY99G3R0C1FA3W4AFR6",
    "actionKind": "Destroy",
    "actionOriginatingChangeSetId": "01JQCHZZVTAHHZ7DG0ZSCB9RXB",
    "actionOriginatingChangeSetName": "2025-03-27-19:41",
    "actionPrototypeId": "01JPNHEE9Z3DFW48XVZ1FX04KA",
    "actionResultState": "Success",
    "attributeValueId": "null",
    "backendKind": "JsAction",
    "backendResponseType": "Action",
    "componentId": "01JP8KHZP3DZKGNXRP83Q6WTQ5",
    "componentName": "NAT Gateway IP 1",
    "createdAt": "2025-03-27T19:41:58.493298051Z",
    "functionArgs": null,
    "functionCodeBase64": "YXN5bmMgZnVuY3Rpb24gbWFpbihjb21wb2...",
    "functionDescription": "null",
    "functionDisplayName": "null",
    "functionKind": "Action",
    "functionLink": "null",
    "functionName": "Delete Asset",
    "id": "01JQCJ0AAXGX5M9QY10AVF4GK1",
    "logs": {},
    "resultValue": null,
    "schemaName": "AWS::EC2::EIP",
    "state": "Success",
    "updatedAt": "2025-03-27T19:42:02.192033089Z"
  }
}

```

### [Properties](#getfuncrunv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|funcRun|[FuncRunViewV1](#schemafuncrunviewv1)|true|none|none|

## [GetFuncV1Response](#tocS_GetFuncV1Response)

<a id="schemagetfuncv1response"></a>
<a id="schema_GetFuncV1Response"></a>
<a id="tocSgetfuncv1response"></a>
<a id="tocsgetfuncv1response"></a>

```json
{
  "code": "async function main(input: Input): Promise < Output > {\n    if (!input.domain?.region) {\n        return {\n            result: \"failure\",\n            message: \"No Region Name to validate\",\n        };\n    }\n\n    const child = await siExec.waitUntilEnd(\"aws\", [\n        \"ec2\",\n        \"describe-regions\",\n        \"--region-names\",\n        input.domain?.region!,\n        \"--region\",\n        \"us-east-1\",\n    ]);\n\n    if (child.exitCode !== 0) {\n        console.error(child.stderr);\n        return {\n            result: \"failure\",\n            message: \"Error from API\"\n        }\n    }\n\n    const regionDetails = JSON.parse(child.stdout).Regions;\n    if (regionDetails.length === 0 || regionDetails.length > 1) {\n        return {\n            result: \"failure\",\n            message: \"Unable to find Region\"\n        }\n    }\n\n    if (regionDetails[0].OptInStatus === \"not-opted-in\") {\n        return {\n            result: \"failure\",\n            message: \"Region not-opted-in for use\"\n        }\n    }\n\n    return {\n        result: \"success\",\n        message: \"Region is available to use\",\n    };\n}",
  "description": "Validates if an AWS region exists and is available for use",
  "displayName": "Validate Region",
  "isLocked": false,
  "kind": "Qualification",
  "link": "https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeRegions.html",
  "name": "AWS Region Validator"
}

```

### [Properties](#getfuncv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|code|string|true|none|none|
|description|string|true|none|none|
|displayName|string|true|none|none|
|isLocked|boolean|true|none|none|
|kind|string|true|none|none|
|link|string|true|none|none|
|name|string|true|none|none|

## [GetManagementFuncJobStateV1Response](#tocS_GetManagementFuncJobStateV1Response)

<a id="schemagetmanagementfuncjobstatev1response"></a>
<a id="schema_GetManagementFuncJobStateV1Response"></a>
<a id="tocSgetmanagementfuncjobstatev1response"></a>
<a id="tocsgetmanagementfuncjobstatev1response"></a>

```json
{
  "funcRunId": "01H9ZQD35JPMBGHH69BT0Q79VY",
  "state": "Executing"
}

```

### [Properties](#getmanagementfuncjobstatev1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|funcRunId|string|true|none|none|
|state|string|true|none|none|

## [GetSchemaV1Response](#tocS_GetSchemaV1Response)

<a id="schemagetschemav1response"></a>
<a id="schema_GetSchemaV1Response"></a>
<a id="tocSgetschemav1response"></a>
<a id="tocsgetschemav1response"></a>

```json
{
  "defaultVariantId": "01H9ZQD35JPMBGHH69BT0Q79VZ",
  "name": "AWS::EC2::Instance",
  "variantIds": [
    "01H9ZQD35JPMBGHH69BT0Q79VZ",
    "01H9ZQD35JPMBGHH69BT0Q79VY"
  ]
}

```

### [Properties](#getschemav1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|defaultVariantId|string|true|none|none|
|name|string|true|none|none|
|variantIds|[string]|true|none|none|

## [GetSchemaVariantV1Response](#tocS_GetSchemaVariantV1Response)

<a id="schemagetschemavariantv1response"></a>
<a id="schema_GetSchemaVariantV1Response"></a>
<a id="tocSgetschemavariantv1response"></a>
<a id="tocsgetschemavariantv1response"></a>

```json
{
  "assetFuncId": "01H9ZQD35JPMBGHH69BT0Q75XY",
  "category": "AWS::EC2",
  "color": "#FF5733",
  "description": "Amazon EC2 Instance resource type",
  "displayName": "AWS EC2 Instance",
  "domainProps": {
    "children": [
      {}
    ],
    "description": "string",
    "name": "string",
    "propId": "string",
    "propType": "string"
  },
  "isDefaultVariant": true,
  "isLocked": false,
  "link": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-instance.html",
  "variantFuncIds": [
    "01H9ZQD35JPMBGHH69BT0Q75AA",
    "01H9ZQD35JPMBGHH69BT0Q75BB"
  ],
  "variantId": "01H9ZQD35JPMBGHH69BT0Q79VZ"
}

```

### [Properties](#getschemavariantv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|assetFuncId|string|true|none|none|
|category|string|true|none|none|
|color|string|true|none|none|
|description|string|true|none|none|
|displayName|string|true|none|none|
|domainProps|[PropSchemaV1](#schemapropschemav1)|true|none|none|
|isDefaultVariant|boolean|true|none|none|
|isLocked|boolean|true|none|none|
|link|string|true|none|none|
|variantFuncIds|[string]|true|none|none|
|variantId|string|true|none|none|

## [HashMap](#tocS_HashMap)

<a id="schemahashmap"></a>
<a id="schema_HashMap"></a>
<a id="tocShashmap"></a>
<a id="tocshashmap"></a>

```json
{
  "property1": {
    "definition": {
      "formData": [
        {
          "name": "access_key_id",
          "kind": "string"
        },
        {
          "name": "secret_access_key",
          "kind": "password"
        }
      ],
      "secretDefinition": "aws_credentials"
    },
    "secrets": [
      {
        "id": "01HAXYZF3GC9CYA6ZVSM3E4YHH",
        "name": "Production AWS Key",
        "definition": "aws_credentials",
        "description": "AWS credentials for production environment"
      },
      {
        "id": "01HAXYZF3GC9CYA6ZVSM3E4YHI",
        "name": "Development AWS Key",
        "definition": "aws_credentials",
        "description": "AWS credentials for development environment"
      }
    ]
  },
  "property2": {
    "definition": {
      "formData": [
        {
          "name": "access_key_id",
          "kind": "string"
        },
        {
          "name": "secret_access_key",
          "kind": "password"
        }
      ],
      "secretDefinition": "aws_credentials"
    },
    "secrets": [
      {
        "id": "01HAXYZF3GC9CYA6ZVSM3E4YHH",
        "name": "Production AWS Key",
        "definition": "aws_credentials",
        "description": "AWS credentials for production environment"
      },
      {
        "id": "01HAXYZF3GC9CYA6ZVSM3E4YHI",
        "name": "Development AWS Key",
        "definition": "aws_credentials",
        "description": "AWS credentials for development environment"
      }
    ]
  }
}

```

### [Properties](#hashmap-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|**additionalProperties**|object|false|none|none|
|» definition|[SecretDefinitionV1](#schemasecretdefinitionv1)|true|none|none|
|» secrets|[[SecretV1](#schemasecretv1)]|true|none|none|

## [IncomingConnectionViewV1](#tocS_IncomingConnectionViewV1)

<a id="schemaincomingconnectionviewv1"></a>
<a id="schema_IncomingConnectionViewV1"></a>
<a id="tocSincomingconnectionviewv1"></a>
<a id="tocsincomingconnectionviewv1"></a>

```json
{
  "from": "string",
  "fromComponentId": "string",
  "fromComponentName": "string",
  "to": "string"
}

```

### [Properties](#incomingconnectionviewv1-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|from|string|true|none|none|
|fromComponentId|string|true|none|none|
|fromComponentName|string|true|none|none|
|to|string|true|none|none|

## [ListChangeSetV1Response](#tocS_ListChangeSetV1Response)

<a id="schemalistchangesetv1response"></a>
<a id="schema_ListChangeSetV1Response"></a>
<a id="tocSlistchangesetv1response"></a>
<a id="tocslistchangesetv1response"></a>

```json
{
  "changeSets": "[{\"id\":\"01H9ZQD35JPMBGHH69BT0Q79VY\",\"name\":\"Add new feature\",\"status\":\"Open\",\"isHead\": \"false\"},{\"id\":\"01H9ZQE356JPMBGHH69BT0Q70UO\",\"name\":\"HEAD\",\"status\":\"Open\", \"isHead\": \"true\"}]"
}

```

### [Properties](#listchangesetv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|changeSets|[object]|true|none|none|

## [ListComponentsV1Response](#tocS_ListComponentsV1Response)

<a id="schemalistcomponentsv1response"></a>
<a id="schema_ListComponentsV1Response"></a>
<a id="tocSlistcomponentsv1response"></a>
<a id="tocslistcomponentsv1response"></a>

```json
{
  "componentDetails": [
    {
      "component_id": "01H9ZQD35JPMBGHH69BT0Q79AA",
      "name": "my-vpc",
      "schema_name": "AWS::EC2::VPC"
    },
    {
      "component_id": "01H9ZQD35JPMBGHH69BT0Q79BB",
      "name": "Public 1",
      "schema_name": "AWS::EC2::Subnet"
    }
  ],
  "components": [
    "01H9ZQD35JPMBGHH69BT0Q79AA",
    "01H9ZQD35JPMBGHH69BT0Q79BB",
    "01H9ZQD35JPMBGHH69BT0Q79CC"
  ],
  "nextCursor": "string"
}

```

### [Properties](#listcomponentsv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|componentDetails|[[ComponentDetailsV1](#schemacomponentdetailsv1)]|true|none|none|
|components|[array]|true|none|none|
|nextCursor|string,null|false|none|none|

## [ListSchemaV1Response](#tocS_ListSchemaV1Response)

<a id="schemalistschemav1response"></a>
<a id="schema_ListSchemaV1Response"></a>
<a id="tocSlistschemav1response"></a>
<a id="tocslistschemav1response"></a>

```json
{
  "nextCursor": "string",
  "schemas": [
    {
      "category": "AWS::EC2",
      "installed": "false",
      "schemaId": "01H9ZQD35JPMBGHH69BT0Q79VY",
      "schemaName": "AWS::EC2::Instance"
    }
  ]
}

```

### [Properties](#listschemav1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|nextCursor|string,null|false|none|none|
|schemas|[[SchemaResponse](#schemaschemaresponse)]|true|none|none|

## [ManageComponentV1Request](#tocS_ManageComponentV1Request)

<a id="schemamanagecomponentv1request"></a>
<a id="schema_ManageComponentV1Request"></a>
<a id="tocSmanagecomponentv1request"></a>
<a id="tocsmanagecomponentv1request"></a>

```json
{
  "componentId": "string"
}

```

### [Properties](#managecomponentv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|componentId|string|true|none|none|

## [ManageComponentV1Response](#tocS_ManageComponentV1Response)

<a id="schemamanagecomponentv1response"></a>
<a id="schema_ManageComponentV1Response"></a>
<a id="tocSmanagecomponentv1response"></a>
<a id="tocsmanagecomponentv1response"></a>

```json
{
  "component": {
    "attributes": {
      "/domain/region": "us-east-1",
      "/secrets/credential": {
        "$source": {
          "component": "demo-credential",
          "path": "/secrets/AWS Credential"
        }
      }
    },
    "canBeUpgraded": true,
    "connections": [
      {
        "incoming": {
          "from": "string",
          "fromComponentId": "string",
          "fromComponentName": "string",
          "to": "string"
        }
      }
    ],
    "domainProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": {}
      }
    ],
    "id": "string",
    "name": "string",
    "resourceId": "string",
    "resourceProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": {}
      }
    ],
    "schemaId": "string",
    "schemaVariantId": "string",
    "sockets": [
      {
        "arity": "one",
        "direction": "input",
        "id": "string",
        "name": "string",
        "value": {}
      }
    ],
    "toDelete": true,
    "views": [
      {
        "id": "string",
        "isDefault": true,
        "name": "string"
      }
    ]
  }
}

```

### [Properties](#managecomponentv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|component|[ComponentViewV1](#schemacomponentviewv1)|true|none|none|

## [ManagedByConnectionViewV1](#tocS_ManagedByConnectionViewV1)

<a id="schemamanagedbyconnectionviewv1"></a>
<a id="schema_ManagedByConnectionViewV1"></a>
<a id="tocSmanagedbyconnectionviewv1"></a>
<a id="tocsmanagedbyconnectionviewv1"></a>

```json
{
  "componentId": "string",
  "componentName": "string"
}

```

### [Properties](#managedbyconnectionviewv1-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|componentId|string|true|none|none|
|componentName|string|true|none|none|

## [ManagementFuncJobStateV1RequestPath](#tocS_ManagementFuncJobStateV1RequestPath)

<a id="schemamanagementfuncjobstatev1requestpath"></a>
<a id="schema_ManagementFuncJobStateV1RequestPath"></a>
<a id="tocSmanagementfuncjobstatev1requestpath"></a>
<a id="tocsmanagementfuncjobstatev1requestpath"></a>

```json
{
  "management_func_job_state_id": "string"
}

```

### [Properties](#managementfuncjobstatev1requestpath-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|management_func_job_state_id|string|true|none|none|

## [ManagementFunctionReference](#tocS_ManagementFunctionReference)

<a id="schemamanagementfunctionreference"></a>
<a id="schema_ManagementFunctionReference"></a>
<a id="tocSmanagementfunctionreference"></a>
<a id="tocsmanagementfunctionreference"></a>

```json
{
  "function": "CreateVpc"
}

```

Reference to a management function by either name or ID.
This allows clients to use the more human-friendly name approach
or the more precise ID approach when working with management functions.

### [Properties](#managementfunctionreference-properties)

oneOf

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|object|false|none|none|
|» function|string|true|none|none|

xor

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|object|false|none|none|
|» managementPrototypeId|string|true|none|none|

## [ManagingConnectionViewV1](#tocS_ManagingConnectionViewV1)

<a id="schemamanagingconnectionviewv1"></a>
<a id="schema_ManagingConnectionViewV1"></a>
<a id="tocSmanagingconnectionviewv1"></a>
<a id="tocsmanagingconnectionviewv1"></a>

```json
{
  "componentId": "string",
  "componentName": "string"
}

```

### [Properties](#managingconnectionviewv1-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|componentId|string|true|none|none|
|componentName|string|true|none|none|

## [MergeStatusV1Response](#tocS_MergeStatusV1Response)

<a id="schemamergestatusv1response"></a>
<a id="schema_MergeStatusV1Response"></a>
<a id="tocSmergestatusv1response"></a>
<a id="tocsmergestatusv1response"></a>

```json
{
  "changeSet": {
    "id": "01FXNV4P306V3KGZ73YSVN8A60",
    "name": "My feature",
    "status": "Ready"
  },
  "actions": [
    {
      "id": "01H9ZQD35JPMBGHH69BT0Q79VY",
      "component": {
        "id": "01H9ZQD35JPMBGHH69BT0Q79AB",
        "name": "my-ec2-instance"
      },
      "state": "Pending",
      "kind": "Create",
      "name": "Create EC2 Instance"
    }
  ]
}

```

Response for merge status

### [Properties](#mergestatusv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|actions|[[MergeStatusV1ResponseAction](#schemamergestatusv1responseaction)]|true|none|[Action item in merge status response]|
|changeSet|object|true|none|none|

## [MergeStatusV1ResponseAction](#tocS_MergeStatusV1ResponseAction)

<a id="schemamergestatusv1responseaction"></a>
<a id="schema_MergeStatusV1ResponseAction"></a>
<a id="tocSmergestatusv1responseaction"></a>
<a id="tocsmergestatusv1responseaction"></a>

```json
{
  "id": "01H9ZQD35JPMBGHH69BT0Q79VY",
  "component": {
    "id": "01H9ZQD35JPMBGHH69BT0Q79AB",
    "name": "my-ec2-instance"
  },
  "state": "Pending",
  "kind": "Create",
  "name": "Create EC2 Instance"
}

```

Action item in merge status response

### [Properties](#mergestatusv1responseaction-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|component|any|false|none|none|

oneOf

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|» *anonymous*|null|false|none|none|

xor

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|» *anonymous*|[MergeStatusV1ResponseActionComponent](#schemamergestatusv1responseactioncomponent)|false|none|Component details in action response|

continued

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|id|string|true|none|none|
|kind|string|true|none|none|
|name|string|true|none|none|
|state|string|true|none|none|

## [MergeStatusV1ResponseActionComponent](#tocS_MergeStatusV1ResponseActionComponent)

<a id="schemamergestatusv1responseactioncomponent"></a>
<a id="schema_MergeStatusV1ResponseActionComponent"></a>
<a id="tocSmergestatusv1responseactioncomponent"></a>
<a id="tocsmergestatusv1responseactioncomponent"></a>

```json
{
  "id": "01H9ZQD35JPMBGHH69BT0Q79AB",
  "name": "my-ec2-instance"
}

```

Component details in action response

### [Properties](#mergestatusv1responseactioncomponent-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|id|string|true|none|none|
|name|string|true|none|none|

## [OutgoingConnectionViewV1](#tocS_OutgoingConnectionViewV1)

<a id="schemaoutgoingconnectionviewv1"></a>
<a id="schema_OutgoingConnectionViewV1"></a>
<a id="tocSoutgoingconnectionviewv1"></a>
<a id="tocsoutgoingconnectionviewv1"></a>

```json
{
  "from": "string",
  "toComponentId": "string",
  "toComponentName": "string"
}

```

### [Properties](#outgoingconnectionviewv1-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|from|string|true|none|none|
|toComponentId|string|true|none|none|
|toComponentName|string|true|none|none|

## [OutputLineViewV1](#tocS_OutputLineViewV1)

<a id="schemaoutputlineviewv1"></a>
<a id="schema_OutputLineViewV1"></a>
<a id="tocSoutputlineviewv1"></a>
<a id="tocsoutputlineviewv1"></a>

```json
{
  "executionId": "01JQCJ0AAXGX5M9QY10AVF4GK1",
  "group": "log",
  "level": "info",
  "message": "Running CLI command: \"aws 'cloudcontrol' 'delete-resource'\"",
  "stream": "stdout",
  "timestamp": 1743104518
}

```

### [Properties](#outputlineviewv1-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|executionId|string|true|none|none|
|group|string,null|false|none|none|
|level|string|true|none|none|
|message|string|true|none|none|
|stream|string|true|none|none|
|timestamp|integer(int64)|true|none|none|

## [PropSchemaV1](#tocS_PropSchemaV1)

<a id="schemapropschemav1"></a>
<a id="schema_PropSchemaV1"></a>
<a id="tocSpropschemav1"></a>
<a id="tocspropschemav1"></a>

```json
{
  "children": [
    {
      "children": [],
      "description": "string",
      "name": "string",
      "propId": "string",
      "propType": "string"
    }
  ],
  "description": "string",
  "name": "string",
  "propId": "string",
  "propType": "string"
}

```

### [Properties](#propschemav1-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|children|[[PropSchemaV1](#schemapropschemav1)]|true|none|none|
|description|string|true|none|none|
|name|string|true|none|none|
|propId|string|true|none|none|
|propType|string|true|none|none|

## [PurgeOpenChangeSetsV1Response](#tocS_PurgeOpenChangeSetsV1Response)

<a id="schemapurgeopenchangesetsv1response"></a>
<a id="schema_PurgeOpenChangeSetsV1Response"></a>
<a id="tocSpurgeopenchangesetsv1response"></a>
<a id="tocspurgeopenchangesetsv1response"></a>

```json
{
  "success": {
    "success": "true"
  }
}

```

### [Properties](#purgeopenchangesetsv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|success|boolean|true|none|none|

## [PutOnHoldActionV1Response](#tocS_PutOnHoldActionV1Response)

<a id="schemaputonholdactionv1response"></a>
<a id="schema_PutOnHoldActionV1Response"></a>
<a id="tocSputonholdactionv1response"></a>
<a id="tocsputonholdactionv1response"></a>

```json
{
  "success": true
}

```

### [Properties](#putonholdactionv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|success|boolean|true|none|none|

## [RequestApprovalChangeSetV1Response](#tocS_RequestApprovalChangeSetV1Response)

<a id="schemarequestapprovalchangesetv1response"></a>
<a id="schema_RequestApprovalChangeSetV1Response"></a>
<a id="tocSrequestapprovalchangesetv1response"></a>
<a id="tocsrequestapprovalchangesetv1response"></a>

```json
{
  "success": "true"
}

```

### [Properties](#requestapprovalchangesetv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|success|boolean|true|none|none|

## [RetryActionV1Response](#tocS_RetryActionV1Response)

<a id="schemaretryactionv1response"></a>
<a id="schema_RetryActionV1Response"></a>
<a id="tocSretryactionv1response"></a>
<a id="tocsretryactionv1response"></a>

```json
{
  "success": true
}

```

### [Properties](#retryactionv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|success|boolean|true|none|none|

## [SchemaResponse](#tocS_SchemaResponse)

<a id="schemaschemaresponse"></a>
<a id="schema_SchemaResponse"></a>
<a id="tocSschemaresponse"></a>
<a id="tocsschemaresponse"></a>

```json
{
  "category": "AWS::EC2",
  "installed": "false",
  "schemaId": "01H9ZQD35JPMBGHH69BT0Q79VY",
  "schemaName": "AWS::EC2::Instance"
}

```

### [Properties](#schemaresponse-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|category|string,null|false|none|none|
|installed|boolean|true|none|none|
|schemaId|string|true|none|none|
|schemaName|string|true|none|none|

## [SchemaV1RequestPath](#tocS_SchemaV1RequestPath)

<a id="schemaschemav1requestpath"></a>
<a id="schema_SchemaV1RequestPath"></a>
<a id="tocSschemav1requestpath"></a>
<a id="tocsschemav1requestpath"></a>

```json
{
  "schema_id": "string"
}

```

### [Properties](#schemav1requestpath-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|schema_id|string|true|none|none|

## [SchemaVariantV1RequestPath](#tocS_SchemaVariantV1RequestPath)

<a id="schemaschemavariantv1requestpath"></a>
<a id="schema_SchemaVariantV1RequestPath"></a>
<a id="tocSschemavariantv1requestpath"></a>
<a id="tocsschemavariantv1requestpath"></a>

```json
{
  "schema_id": "string",
  "schema_variant_id": "string"
}

```

### [Properties](#schemavariantv1requestpath-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|schema_id|string|true|none|none|
|schema_variant_id|string|true|none|none|

## [SearchComponentsV1Request](#tocS_SearchComponentsV1Request)

<a id="schemasearchcomponentsv1request"></a>
<a id="schema_SearchComponentsV1Request"></a>
<a id="tocSsearchcomponentsv1request"></a>
<a id="tocssearchcomponentsv1request"></a>

```json
{
  "schemaName": "AWS::EC2::Instance"
}

```

### [Properties](#searchcomponentsv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|schemaName|string,null|false|none|none|

## [SearchComponentsV1Response](#tocS_SearchComponentsV1Response)

<a id="schemasearchcomponentsv1response"></a>
<a id="schema_SearchComponentsV1Response"></a>
<a id="tocSsearchcomponentsv1response"></a>
<a id="tocssearchcomponentsv1response"></a>

```json
{
  "components": [
    "01H9ZQD35JPMBGHH69BT0Q79AA",
    "01H9ZQD35JPMBGHH69BT0Q79BB",
    "01H9ZQD35JPMBGHH69BT0Q79CC"
  ]
}

```

### [Properties](#searchcomponentsv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|components|[array]|true|none|none|

## [SecretDefinitionV1](#tocS_SecretDefinitionV1)

<a id="schemasecretdefinitionv1"></a>
<a id="schema_SecretDefinitionV1"></a>
<a id="tocSsecretdefinitionv1"></a>
<a id="tocssecretdefinitionv1"></a>

```json
{
  "formData": [
    {
      "name": "access_key_id",
      "kind": "string"
    },
    {
      "name": "secret_access_key",
      "kind": "password"
    }
  ],
  "secretDefinition": "aws_credentials"
}

```

### [Properties](#secretdefinitionv1-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|formData|[[SecretFormDataV1](#schemasecretformdatav1)]|true|none|none|
|secretDefinition|string|true|none|none|

## [SecretFormDataV1](#tocS_SecretFormDataV1)

<a id="schemasecretformdatav1"></a>
<a id="schema_SecretFormDataV1"></a>
<a id="tocSsecretformdatav1"></a>
<a id="tocssecretformdatav1"></a>

```json
{
  "kind": "string",
  "name": "access_key_id"
}

```

### [Properties](#secretformdatav1-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|kind|string|true|none|none|
|name|string|true|none|none|

## [SecretPropKey](#tocS_SecretPropKey)

<a id="schemasecretpropkey"></a>
<a id="schema_SecretPropKey"></a>
<a id="tocSsecretpropkey"></a>
<a id="tocssecretpropkey"></a>

```json
"string"

```

### [Properties](#secretpropkey-properties)

oneOf

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|string|false|none|none|

xor

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|[SecretPropPath](#schemasecretproppath)|false|none|none|

## [SecretPropPath](#tocS_SecretPropPath)

<a id="schemasecretproppath"></a>
<a id="schema_SecretPropPath"></a>
<a id="tocSsecretproppath"></a>
<a id="tocssecretproppath"></a>

```json
"string"

```

### [Properties](#secretproppath-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|string|false|none|none|

## [SecretV1](#tocS_SecretV1)

<a id="schemasecretv1"></a>
<a id="schema_SecretV1"></a>
<a id="tocSsecretv1"></a>
<a id="tocssecretv1"></a>

```json
{
  "definition": "aws_credentials",
  "description": "AWS credentials for production environment",
  "id": "01HAXYZF3GC9CYA6ZVSM3E4YHH",
  "name": "Production AWS Key"
}

```

### [Properties](#secretv1-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|definition|string|true|none|none|
|description|string|true|none|none|
|id|string|true|none|none|
|name|string|true|none|none|

## [SocketDirection](#tocS_SocketDirection)

<a id="schemasocketdirection"></a>
<a id="schema_SocketDirection"></a>
<a id="tocSsocketdirection"></a>
<a id="tocssocketdirection"></a>

```json
"input"

```

### [Properties](#socketdirection-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|string|false|none|none|

#### [Enumerated Values](#socketdirection-enumerated-values)

|Property|Value|
|---|---|
|*anonymous*|input|
|*anonymous*|output|

## [SocketViewV1](#tocS_SocketViewV1)

<a id="schemasocketviewv1"></a>
<a id="schema_SocketViewV1"></a>
<a id="tocSsocketviewv1"></a>
<a id="tocssocketviewv1"></a>

```json
{
  "arity": "one",
  "direction": "input",
  "id": "string",
  "name": "string",
  "value": {}
}

```

### [Properties](#socketviewv1-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|arity|string|true|none|none|
|direction|[SocketDirection](#schemasocketdirection)|true|none|none|
|id|string|true|none|none|
|name|string|true|none|none|
|value|object|true|none|none|

## [SourceViewV1](#tocS_SourceViewV1)

<a id="schemasourceviewv1"></a>
<a id="schema_SourceViewV1"></a>
<a id="tocSsourceviewv1"></a>
<a id="tocssourceviewv1"></a>

```json
{
  "component": "string",
  "propPath": "string"
}

```

### [Properties](#sourceviewv1-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|component|string|true|none|none|
|propPath|string|true|none|none|

## [Subscription](#tocS_Subscription)

<a id="schemasubscription"></a>
<a id="schema_Subscription"></a>
<a id="tocSsubscription"></a>
<a id="tocssubscription"></a>

```json
{
  "component": "ComponentName",
  "function": "string",
  "keepExistingSubscriptions": true,
  "propPath": "string"
}

```

### [Properties](#subscription-properties)

allOf

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|[ComponentReference](#schemacomponentreference)|false|none|none|

and

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|object|false|none|none|
|» function|string|false|none|none|
|» keepExistingSubscriptions|boolean,null|false|none|none|
|» propPath|string|true|none|none|

## [SystemStatusResponse](#tocS_SystemStatusResponse)

<a id="schemasystemstatusresponse"></a>
<a id="schema_SystemStatusResponse"></a>
<a id="tocSsystemstatusresponse"></a>
<a id="tocssystemstatusresponse"></a>

```json
{
  "API Documentation": "Available at /swagger-ui"
}

```

### [Properties](#systemstatusresponse-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|API Documentation|string|true|none|none|

## [UpdateComponentV1Request](#tocS_UpdateComponentV1Request)

<a id="schemaupdatecomponentv1request"></a>
<a id="schema_UpdateComponentV1Request"></a>
<a id="tocSupdatecomponentv1request"></a>
<a id="tocsupdatecomponentv1request"></a>

```json
{
  "attributes": {
    "/domain/VpcId": {
      "$source": {
        "component": "01K0WRC69ZPEMD6SMTKC84FBWC",
        "path": "/resource_value/VpcId"
      }
    },
    "/domain/SubnetId": {
      "$source": {
        "component": "01K0WRC69ZPEMD6SMTKC84FBWD",
        "path": "/resource_value/SubnetId"
      }
    },
    "/domain/Version": {
      "$source": null
    }
  },
  "connectionChanges": {
    "add": {},
    "remove": {}
  },
  "domain": {},
  "name": "MyUpdatedComponentName",
  "resourceId": "i-12345678",
  "secrets": {},
  "subscriptions": {},
  "unset": {}
}

```

### [Properties](#updatecomponentv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|attributes|object|false|none|none|
|» **additionalProperties**|any|false|none|none|
|connectionChanges|[ConnectionDetails](#schemaconnectiondetails)|false|none|none|
|domain|object|false|none|none|
|» **additionalProperties**|any|false|none|none|
|name|string,null|false|none|none|
|resourceId|string,null|false|none|none|
|secrets|object|false|none|none|
|» **additionalProperties**|any|false|none|none|
|subscriptions|object|false|none|none|
|» **additionalProperties**|[Subscription](#schemasubscription)|false|none|none|
|unset|[[ComponentPropKey](#schemacomponentpropkey)]|false|none|none|

## [UpdateComponentV1Response](#tocS_UpdateComponentV1Response)

<a id="schemaupdatecomponentv1response"></a>
<a id="schema_UpdateComponentV1Response"></a>
<a id="tocSupdatecomponentv1response"></a>
<a id="tocsupdatecomponentv1response"></a>

```json
{
  "component": {
    "attributes": {
      "/domain/region": "us-east-1",
      "/secrets/credential": {
        "$source": {
          "component": "demo-credential",
          "path": "/secrets/AWS Credential"
        }
      }
    },
    "canBeUpgraded": true,
    "connections": [
      {
        "incoming": {
          "from": "string",
          "fromComponentId": "string",
          "fromComponentName": "string",
          "to": "string"
        }
      }
    ],
    "domainProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": {}
      }
    ],
    "id": "string",
    "name": "string",
    "resourceId": "string",
    "resourceProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": {}
      }
    ],
    "schemaId": "string",
    "schemaVariantId": "string",
    "sockets": [
      {
        "arity": "one",
        "direction": "input",
        "id": "string",
        "name": "string",
        "value": {}
      }
    ],
    "toDelete": true,
    "views": [
      {
        "id": "string",
        "isDefault": true,
        "name": "string"
      }
    ]
  }
}

```

### [Properties](#updatecomponentv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|component|[ComponentViewV1](#schemacomponentviewv1)|true|none|none|

## [UpdateSecretV1Request](#tocS_UpdateSecretV1Request)

<a id="schemaupdatesecretv1request"></a>
<a id="schema_UpdateSecretV1Request"></a>
<a id="tocSupdatesecretv1request"></a>
<a id="tocsupdatesecretv1request"></a>

```json
{
  "description": "Updated AWS Secret Key for EC2 access",
  "id": "01HAXYZF3GC9CYA6ZVSM3E4YHH",
  "name": "AWS Access Key",
  "rawData": {
    "access_key_id": "AKIAIOSFODNN7EXAMPLE",
    "secret_access_key": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
    "region": "us-west-2",
    "default_output": "json"
  }
}

```

### [Properties](#updatesecretv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|description|string|true|none|none|
|id|string|true|none|none|
|name|string|true|none|none|
|rawData|object|false|none|none|
|» **additionalProperties**|string|false|none|none|

## [UpdateSecretV1Response](#tocS_UpdateSecretV1Response)

<a id="schemaupdatesecretv1response"></a>
<a id="schema_UpdateSecretV1Response"></a>
<a id="tocSupdatesecretv1response"></a>
<a id="tocsupdatesecretv1response"></a>

```json
{
  "secret": {
    "definition": "aws_credentials",
    "description": "AWS credentials for production environment",
    "id": "01HAXYZF3GC9CYA6ZVSM3E4YHH",
    "name": "Production AWS Key"
  }
}

```

### [Properties](#updatesecretv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|secret|[SecretV1](#schemasecretv1)|true|none|none|

## [UpgradeComponentV1Response](#tocS_UpgradeComponentV1Response)

<a id="schemaupgradecomponentv1response"></a>
<a id="schema_UpgradeComponentV1Response"></a>
<a id="tocSupgradecomponentv1response"></a>
<a id="tocsupgradecomponentv1response"></a>

```json
{
  "component": {
    "attributes": {
      "/domain/region": "us-east-1",
      "/secrets/credential": {
        "$source": {
          "component": "demo-credential",
          "path": "/secrets/AWS Credential"
        }
      }
    },
    "canBeUpgraded": true,
    "connections": [
      {
        "incoming": {
          "from": "string",
          "fromComponentId": "string",
          "fromComponentName": "string",
          "to": "string"
        }
      }
    ],
    "domainProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": {}
      }
    ],
    "id": "string",
    "name": "string",
    "resourceId": "string",
    "resourceProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": {}
      }
    ],
    "schemaId": "string",
    "schemaVariantId": "string",
    "sockets": [
      {
        "arity": "one",
        "direction": "input",
        "id": "string",
        "name": "string",
        "value": {}
      }
    ],
    "toDelete": true,
    "views": [
      {
        "id": "string",
        "isDefault": true,
        "name": "string"
      }
    ]
  }
}

```

### [Properties](#upgradecomponentv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|component|[ComponentViewV1](#schemacomponentviewv1)|true|none|none|

## [ViewV1](#tocS_ViewV1)

<a id="schemaviewv1"></a>
<a id="schema_ViewV1"></a>
<a id="tocSviewv1"></a>
<a id="tocsviewv1"></a>

```json
{
  "id": "string",
  "isDefault": true,
  "name": "string"
}

```

### [Properties](#viewv1-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|id|string|true|none|none|
|isDefault|boolean|true|none|none|
|name|string|true|none|none|

## [WhoamiResponse](#tocS_WhoamiResponse)

<a id="schemawhoamiresponse"></a>
<a id="schema_WhoamiResponse"></a>
<a id="tocSwhoamiresponse"></a>
<a id="tocswhoamiresponse"></a>

```json
{
  "token": {},
  "userEmail": "user@example.com",
  "userId": "01H9ZQCBJ3E7HBTRN3J58JQX8K",
  "workspaceId": "01H9ZQD35JPMBGHH69BT0Q79VY"
}

```

### [Properties](#whoamiresponse-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|token|object|true|none|none|
|userEmail|string|true|none|none|
|userId|string|true|none|none|
|workspaceId|string|true|none|none|

