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
|428|[Precondition Required](https://tools.ietf.org/html/rfc6585#section-3)|DVU Roots still exist, apply must be tried again later.|None|
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
|includeCodegen|query|boolean|false|Allow returning the codegen for the cloudformation template for the component (if it exists)|

> Example responses

> 200 Response

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
  "managedBy": {
    "component": "ComponentName"
  },
  "name": "MyComponentName",
  "resourceId": "i-12345678",
  "schemaName": "AWS::EC2::Instance",
  "useWorkingCopy": true,
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
        "managing": {
          "componentId": "string",
          "componentName": "string"
        }
      }
    ],
    "domainProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": null
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
        "value": null
      }
    ],
    "schemaId": "string",
    "schemaVariantId": "string",
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

## Add components to a view

<a id="opIdadd_to_view"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/components/add_to_view`

Adds multiple components to a view by name. If the view doesn't exist, it will be created automatically.

> Body parameter

```json
{
  "componentIds": [
    "string"
  ],
  "viewName": "string"
}
```

<h3 id="add-components-to-a-view-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|body|body|[AddToViewV1Request](#schemaaddtoviewv1request)|true|none|

> Example responses

> 409 Response

```json
{
  "message": "Invalid request data",
  "statusCode": 422,
  "code": 4001
}
```

<h3 id="add-components-to-a-view-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|204|[No Content](https://tools.ietf.org/html/rfc7231#section-6.3.5)|Components added to view successfully|None|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|409|[Conflict](https://tools.ietf.org/html/rfc7231#section-6.5.8)|Conflict - Changes not permitted on HEAD change set|[ApiError](#schemaapierror)|
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
        "managing": {
          "componentId": "string",
          "componentName": "string"
        }
      }
    ],
    "domainProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": null
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
        "value": null
      }
    ],
    "schemaId": "string",
    "schemaVariantId": "string",
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

## Generate a template

<a id="opIdgenerate_template"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/components/generate_template`

> Body parameter

```json
{
  "assetName": "My Cool Template",
  "category": "Templates",
  "componentIds": [
    "01H9ZQD35JPMBGHH69BT0Q79AA",
    "01H9ZQD35JPMBGHH69BT0Q79BB",
    "01H9ZQD35JPMBGHH69BT0Q79CC"
  ],
  "funcName": "Generate My Template"
}
```

<h3 id="generate-a-template-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|body|body|[GenerateTemplateV1Request](#schemageneratetemplatev1request)|true|none|

> Example responses

> 200 Response

```json
{
  "funcId": "01H9ZQD35JPMBGHH69BT0Q79CC",
  "schemaId": "01H9ZQD35JPMBGHH69BT0Q79AA",
  "schemaVariantId": "01H9ZQD35JPMBGHH69BT0Q79BB"
}
```

<h3 id="generate-a-template-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Template generated successfully|[GenerateTemplateV1Response](#schemageneratetemplatev1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Complex search for components

<a id="opIdsearch_components"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/components/search`

> Body parameter

```json
{
  "queryString": "string",
  "schemaCategory": "AWS::EC2",
  "schemaName": "AWS::EC2::Instance",
  "upgradable": true
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
        "managing": {
          "componentId": "string",
          "componentName": "string"
        }
      }
    ],
    "domainProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": null
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
        "value": null
      }
    ],
    "schemaId": "string",
    "schemaVariantId": "string",
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
  "name": "MyUpdatedComponentName",
  "resourceId": "i-12345678",
  "secrets": {
    "secretDefinitionName": "secretName"
  }
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
        "managing": {
          "componentId": "string",
          "componentName": "string"
        }
      }
    ],
    "domainProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": null
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
        "value": null
      }
    ],
    "schemaId": "string",
    "schemaVariantId": "string",
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

## Erase a component without queuing a delete action

<a id="opIderase_component"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}/erase`

<h3 id="erase-a-component-without-queuing-a-delete-action-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|component_id|path|string|true|Component identifier|

> Example responses

> 200 Response

```json
{
  "status": "true"
}
```

<h3 id="erase-a-component-without-queuing-a-delete-action-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Component erased successfully|[EraseComponentV1Response](#schemaerasecomponentv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Component not found|None|
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
        "managing": {
          "componentId": "string",
          "componentName": "string"
        }
      }
    ],
    "domainProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": null
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
        "value": null
      }
    ],
    "schemaId": "string",
    "schemaVariantId": "string",
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

## Get a component resource by component Id

<a id="opIdget_component_resource"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}/resource`

<h3 id="get-a-component-resource-by-component-id-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|component_id|path|string|true|Component identifier|

> Example responses

> 200 Response

```json
{
  "last_synced": "2024-01-15T12:30:00Z",
  "payload": null,
  "status": "Ok"
}
```

<h3 id="get-a-component-resource-by-component-id-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Component resource retrieved successfully|[GetComponentResourceDataV1Response](#schemagetcomponentresourcedatav1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|412|[Precondition Failed](https://tools.ietf.org/html/rfc7232#section-4.2)|Component has no associated resource|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Restore a component that is marked for deletion

<a id="opIdrestore_component"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}/restore`

<h3 id="restore-a-component-that-is-marked-for-deletion-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|component_id|path|string|true|Component identifier|

> Example responses

> 200 Response

```json
{
  "status": "true"
}
```

<h3 id="restore-a-component-that-is-marked-for-deletion-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Component restored successfully|[RestoreComponentV1Response](#schemarestorecomponentv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Component not found|None|
|412|[Precondition Failed](https://tools.ietf.org/html/rfc7232#section-4.2)|Component not marked for deletion|None|
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
        "managing": {
          "componentId": "string",
          "componentName": "string"
        }
      }
    ],
    "domainProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": null
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
        "value": null
      }
    ],
    "schemaId": "string",
    "schemaVariantId": "string",
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

## Create a schema and it's default variant

<a id="opIdcreate_schema"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/schemas`

> Body parameter

```json
{
  "category": "string",
  "code": "string",
  "color": "string",
  "description": "string",
  "link": "string",
  "name": "string"
}
```

<h3 id="create-a-schema-and-it's-default-variant-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|body|body|[CreateSchemaV1Request](#schemacreateschemav1request)|true|none|

> Example responses

> 200 Response

```json
{
  "defaultVariantId": "01H9ZQD35JPMBGHH69BT0Q79VZ",
  "name": "AWS::EC2::Instance",
  "schemaId": "string",
  "variantIds": [
    "01H9ZQD35JPMBGHH69BT0Q79VZ",
    "01H9ZQD35JPMBGHH69BT0Q79VY"
  ]
}
```

<h3 id="create-a-schema-and-it's-default-variant-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Schema created successfully|[GetSchemaV1Response](#schemagetschemav1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|422|[Unprocessable Entity](https://tools.ietf.org/html/rfc2518#section-10.3)|Validation error - Invalid request data|[ApiError](#schemaapierror)|
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

## Complex search for schemas

<a id="opIdsearch_schemas"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/search`

> Body parameter

```json
{
  "category": "AWS::EC2"
}
```

<h3 id="complex-search-for-schemas-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|body|body|[SearchSchemasV1Request](#schemasearchschemasv1request)|true|none|

> Example responses

> 200 Response

```json
{
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

<h3 id="complex-search-for-schemas-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Schemas retrieved successfully|[SearchSchemasV1Response](#schemasearchschemasv1response)|
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
  "schemaId": "string",
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
|202|[Accepted](https://tools.ietf.org/html/rfc7231#section-6.3.3)|Schema data is being generated from cached modules|[BuildingResponseV1](#schemabuildingresponsev1)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Schema not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Installs a schema - if there's an installed schema, it will return that schema detail

<a id="opIdinstall_schema"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/install`

<h3 id="installs-a-schema---if-there's-an-installed-schema,-it-will-return-that-schema-detail-parameters">Parameters</h3>

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
  "schemaId": "string",
  "variantIds": [
    "01H9ZQD35JPMBGHH69BT0Q79VZ",
    "01H9ZQD35JPMBGHH69BT0Q79VY"
  ]
}
```

<h3 id="installs-a-schema---if-there's-an-installed-schema,-it-will-return-that-schema-detail-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Schema installed successfully|[GetSchemaV1Response](#schemagetschemav1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|422|[Unprocessable Entity](https://tools.ietf.org/html/rfc2518#section-10.3)|Validation error - Invalid request data|[ApiError](#schemaapierror)|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Unlocks a schema - if there's already an unlocked variant, then we return that

<a id="opIdunlock_schema"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/unlock`

<h3 id="unlocks-a-schema---if-there's-already-an-unlocked-variant,-then-we-return-that-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|schema_id|path|string|true|Schema identifier|

> Example responses

> 200 Response

```json
{
  "schemaId": "01H9ZQD35JPMBGHH69BT0Q79VZ",
  "unlockedVariant": {
    "assetFuncId": "01H9ZQD35JPMBGHH69BT0Q75XY",
    "category": "AWS::EC2",
    "color": "#FF5733",
    "description": "Amazon EC2 Instance resource type",
    "displayName": "AWS EC2 Instance",
    "domainProps": {},
    "installedFromUpstream": false,
    "isDefaultVariant": true,
    "isLocked": false,
    "link": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-instance.html",
    "variantFuncIds": [
      "01H9ZQD35JPMBGHH69BT0Q75AA",
      "01H9ZQD35JPMBGHH69BT0Q75BB"
    ],
    "variantFuncs": [
      {
        "funcKind": {
          "actionKind": "Create",
          "kind": "action"
        },
        "id": "01H9ZQD35JPMBGHH69BT0Q79VZ",
        "isOverlay": true
      }
    ],
    "variantId": "01H9ZQD35JPMBGHH69BT0Q79VZ"
  },
  "unlockedVariantId": "01H9ZQD35JPMBGHH69BT0Q75XY"
}
```

<h3 id="unlocks-a-schema---if-there's-already-an-unlocked-variant,-then-we-return-that-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Schema unlocked successfully|[UnlockedSchemaV1Response](#schemaunlockedschemav1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|422|[Unprocessable Entity](https://tools.ietf.org/html/rfc2518#section-10.3)|Validation error - Invalid request data|[ApiError](#schemaapierror)|
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
  "domainProps": {},
  "installedFromUpstream": false,
  "isDefaultVariant": true,
  "isLocked": false,
  "link": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-instance.html",
  "variantFuncIds": [
    "01H9ZQD35JPMBGHH69BT0Q75AA",
    "01H9ZQD35JPMBGHH69BT0Q75BB"
  ],
  "variantFuncs": [
    {
      "funcKind": {
        "actionKind": "Create",
        "kind": "action"
      },
      "id": "01H9ZQD35JPMBGHH69BT0Q79VZ",
      "isOverlay": true
    }
  ],
  "variantId": "01H9ZQD35JPMBGHH69BT0Q79VZ"
}
```

<h3 id="get-the-default-variant-for-a-schema-id-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Schema variant retrieved successfully|[GetSchemaVariantV1Response](#schemagetschemavariantv1response)|
|202|[Accepted](https://tools.ietf.org/html/rfc7231#section-6.3.3)|Schema variant building, try again later|[BuildingResponseV1](#schemabuildingresponsev1)|
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
  "domainProps": {},
  "installedFromUpstream": false,
  "isDefaultVariant": true,
  "isLocked": false,
  "link": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-instance.html",
  "variantFuncIds": [
    "01H9ZQD35JPMBGHH69BT0Q75AA",
    "01H9ZQD35JPMBGHH69BT0Q75BB"
  ],
  "variantFuncs": [
    {
      "funcKind": {
        "actionKind": "Create",
        "kind": "action"
      },
      "id": "01H9ZQD35JPMBGHH69BT0Q79VZ",
      "isOverlay": true
    }
  ],
  "variantId": "01H9ZQD35JPMBGHH69BT0Q79VZ"
}
```

<h3 id="get-a-schema-variant-by-schema-id-and-schema-variant-id-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Schema variant retrieved successfully|[GetSchemaVariantV1Response](#schemagetschemavariantv1response)|
|202|[Accepted](https://tools.ietf.org/html/rfc7231#section-6.3.3)|Schema variant building, try again later|[BuildingResponseV1](#schemabuildingresponsev1)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Schema variant not found|None|
|412|[Precondition Failed](https://tools.ietf.org/html/rfc7232#section-4.2)|Schema variant not found for schema|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Update the schema variant and regenerate

<a id="opIdupdate_schema_variant"></a>

> Request format

`PUT /v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/variant/{schema_variant_id}`

> Body parameter

```json
{
  "category": "AWS::EC2",
  "code": "async function main(input: Input): Promise < Output > {\n    if (!input.domain?.region) {\n        return {\n            result: \"failure\",\n            message: \"No Region Name to validate\",\n        };\n    }\n\n    const child = await siExec.waitUntilEnd(\"aws\", [\n        \"ec2\",\n        \"describe-regions\",\n        \"--region-names\",\n        input.domain?.region!,\n        \"--region\",\n        \"us-east-1\",\n    ]);\n\n    if (child.exitCode !== 0) {\n        console.error(child.stderr);\n        return {\n            result: \"failure\",\n            message: \"Error from API\"\n        }\n    }\n\n    const regionDetails = JSON.parse(child.stdout).Regions;\n    if (regionDetails.length === 0 || regionDetails.length > 1) {\n        return {\n            result: \"failure\",\n            message: \"Unable to find Region\"\n        }\n    }\n\n    if (regionDetails[0].OptInStatus === \"not-opted-in\") {\n        return {\n            result: \"failure\",\n            message: \"Region not-opted-in for use\"\n        }\n    }\n\n    return {\n        result: \"success\",\n        message: \"Region is available to use\",\n    };\n}",
  "color": "#FF5733",
  "description": "Validates if an AWS region exists and is available for use",
  "link": "https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeRegions.html",
  "name": "AWS Region Validator"
}
```

<h3 id="update-the-schema-variant-and-regenerate-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|schema_id|path|string|true|Schema identifier|
|schema_variant_id|path|string|true|Schema variant identifier|
|body|body|[UpdateSchemaVariantV1Request](#schemaupdateschemavariantv1request)|true|none|

> Example responses

> 200 Response

```json
{
  "assetFuncId": "01H9ZQD35JPMBGHH69BT0Q75XY",
  "category": "AWS::EC2",
  "color": "#FF5733",
  "description": "Amazon EC2 Instance resource type",
  "displayName": "AWS EC2 Instance",
  "domainProps": {},
  "installedFromUpstream": false,
  "isDefaultVariant": true,
  "isLocked": false,
  "link": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-instance.html",
  "variantFuncIds": [
    "01H9ZQD35JPMBGHH69BT0Q75AA",
    "01H9ZQD35JPMBGHH69BT0Q75BB"
  ],
  "variantFuncs": [
    {
      "funcKind": {
        "actionKind": "Create",
        "kind": "action"
      },
      "id": "01H9ZQD35JPMBGHH69BT0Q79VZ",
      "isOverlay": true
    }
  ],
  "variantId": "01H9ZQD35JPMBGHH69BT0Q79VZ"
}
```

<h3 id="update-the-schema-variant-and-regenerate-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Schema variant successfully updated|[GetSchemaVariantV1Response](#schemagetschemavariantv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Schema variant not found|None|
|412|[Precondition Failed](https://tools.ietf.org/html/rfc7232#section-4.2)|Schema variant not found for schema|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Create an action function and attach to a schema variant

<a id="opIdcreate_variant_action"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/variant/{schema_variant_id}/funcs/action`

> Body parameter

```json
{
  "code": "<!-- String escaped Typescript code here -->",
  "description": "Creates an EC2 Instance",
  "displayName": "Create EC2 Instance",
  "kind": "Create",
  "name": "awsEC2InstanceCreate",
  "skipOverlay": false
}
```

<h3 id="create-an-action-function-and-attach-to-a-schema-variant-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|schema_id|path|string|true|Schema identifier|
|schema_variant_id|path|string|true|Schema variant identifier|
|body|body|[CreateVariantActionFuncV1Request](#schemacreatevariantactionfuncv1request)|true|none|

> Example responses

> 200 Response

```json
{
  "funcId": "01H9ZQD35JPMBGHH69BT0Q79VZ"
}
```

<h3 id="create-an-action-function-and-attach-to-a-schema-variant-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Action function successfully created and attached to the variant|[CreateVariantActionFuncV1Response](#schemacreatevariantactionfuncv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Schema variant not found|None|
|412|[Precondition Failed](https://tools.ietf.org/html/rfc7232#section-4.2)|Schema variant not found for schema|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Delete the binding between an action func and the schema variant

<a id="opIddetach_action_func_binding"></a>

> Request format

`DELETE /v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/variant/{schema_variant_id}/funcs/action/{func_id}`

<h3 id="delete-the-binding-between-an-action-func-and-the-schema-variant-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|schema_id|path|string|true|Schema identifier|
|schema_variant_id|path|string|true|Schema variant identifier|
|func_id|path|string|true|Func identifier|

> Example responses

> 200 Response

```json
{
  "success": true
}
```

<h3 id="delete-the-binding-between-an-action-func-and-the-schema-variant-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Action function successfully deteched from the variant|[DetachFuncBindingV1Response](#schemadetachfuncbindingv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Func not found|None|
|412|[Precondition Failed](https://tools.ietf.org/html/rfc7232#section-4.2)|Schema variant not found for schema|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Create an attribute function and attach to a schema variant

<a id="opIdcreate_variant_attribute"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/variant/{schema_variant_id}/funcs/attribute`

> Body parameter

```json
{
  "argumentBindings": [
    {
      "elementKind": "String",
      "kind": "String",
      "name": "instanceType",
      "propId": "01H9ZQD35JPMBGHH69BT0Q79VZ",
      "staticValue": null
    }
  ],
  "code": "async function main(instanceType: Input): Promise<Output> { return instanceType; }",
  "componentId": "01H9ZQD35JPMBGHH69BT0Q79VZ",
  "description": "Sets the instance type for an EC2 Instance",
  "displayName": "Set Instance Type",
  "name": "awsEC2SetInstanceType",
  "propId": "01H9ZQD35JPMBGHH69BT0Q79VZ",
  "skipOverlay": false
}
```

<h3 id="create-an-attribute-function-and-attach-to-a-schema-variant-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|schema_id|path|string|true|Schema identifier|
|schema_variant_id|path|string|true|Schema variant identifier|
|body|body|[CreateVariantAttributeFuncV1Request](#schemacreatevariantattributefuncv1request)|true|none|

> Example responses

> 200 Response

```json
{
  "attributePrototypeId": "01H9ZQD35JPMBGHH69BT0Q79VZ",
  "funcId": "01H9ZQD35JPMBGHH69BT0Q79VZ"
}
```

<h3 id="create-an-attribute-function-and-attach-to-a-schema-variant-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Attribute function successfully created and attached to the variant|[CreateVariantAttributeFuncV1Response](#schemacreatevariantattributefuncv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Schema variant not found|None|
|412|[Precondition Failed](https://tools.ietf.org/html/rfc7232#section-4.2)|Schema variant not found for schema|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Delete the binding between an attribute func and the schema variant

<a id="opIddetach_attribute_func_binding"></a>

> Request format

`DELETE /v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/variant/{schema_variant_id}/funcs/attribute/{func_id}`

<h3 id="delete-the-binding-between-an-attribute-func-and-the-schema-variant-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|schema_id|path|string|true|Schema identifier|
|schema_variant_id|path|string|true|Schema variant identifier|
|func_id|path|string|true|Func identifier|

> Example responses

> 200 Response

```json
{
  "success": true
}
```

<h3 id="delete-the-binding-between-an-attribute-func-and-the-schema-variant-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Attribute function successfully detached from the variant|[DetachFuncBindingV1Response](#schemadetachfuncbindingv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Func not found|None|
|412|[Precondition Failed](https://tools.ietf.org/html/rfc7232#section-4.2)|Schema variant not found for schema|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Create an authentication function and attach to a schema variant

<a id="opIdcreate_variant_authentication"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/variant/{schema_variant_id}/funcs/authentication`

> Body parameter

```json
{
  "code": "<!-- String escaped Typescript code here -->",
  "description": "Function to manage AWS Credentials",
  "displayName": "Set AWS credentials",
  "name": "awsSetCredentials"
}
```

<h3 id="create-an-authentication-function-and-attach-to-a-schema-variant-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|schema_id|path|string|true|Schema identifier|
|schema_variant_id|path|string|true|Schema variant identifier|
|body|body|[CreateVariantAuthenticationFuncV1Request](#schemacreatevariantauthenticationfuncv1request)|true|none|

> Example responses

> 200 Response

```json
{
  "funcId": "01H9ZQD35JPMBGHH69BT0Q79VZ"
}
```

<h3 id="create-an-authentication-function-and-attach-to-a-schema-variant-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Authentication function successfully created and attached to the variant|[CreateVariantAuthenticationFuncV1Response](#schemacreatevariantauthenticationfuncv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Schema variant not found|None|
|412|[Precondition Failed](https://tools.ietf.org/html/rfc7232#section-4.2)|Schema variant not found for schema|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Delete the binding between an authentication func and the schema variant

<a id="opIddetach_authentication_func_binding"></a>

> Request format

`DELETE /v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/variant/{schema_variant_id}/funcs/authentication/{func_id}`

<h3 id="delete-the-binding-between-an-authentication-func-and-the-schema-variant-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|schema_id|path|string|true|Schema identifier|
|schema_variant_id|path|string|true|Schema variant identifier|
|func_id|path|string|true|Func identifier|

> Example responses

> 200 Response

```json
{
  "success": true
}
```

<h3 id="delete-the-binding-between-an-authentication-func-and-the-schema-variant-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Authentication function successfully detached from the variant|[DetachFuncBindingV1Response](#schemadetachfuncbindingv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Func not found|None|
|412|[Precondition Failed](https://tools.ietf.org/html/rfc7232#section-4.2)|Schema variant not found for schema|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Create a codegen function and attach to a schema variant

<a id="opIdcreate_variant_codegen"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/variant/{schema_variant_id}/funcs/codegen`

> Body parameter

```json
{
  "code": "<!-- String escaped Typescript code here -->",
  "description": "Generates the payload required for creating an EC2 instance",
  "displayName": "Generate EC2 Instance Create Payload",
  "name": "awsEC2InstanceGenerateCode",
  "skipOverlay": false
}
```

<h3 id="create-a-codegen-function-and-attach-to-a-schema-variant-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|schema_id|path|string|true|Schema identifier|
|schema_variant_id|path|string|true|Schema variant identifier|
|body|body|[CreateVariantCodegenFuncV1Request](#schemacreatevariantcodegenfuncv1request)|true|none|

> Example responses

> 200 Response

```json
{
  "funcId": "01H9ZQD35JPMBGHH69BT0Q79VZ"
}
```

<h3 id="create-a-codegen-function-and-attach-to-a-schema-variant-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Codegen function successfully created and attached to the variant|[CreateVariantCodegenFuncV1Response](#schemacreatevariantcodegenfuncv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Schema variant not found|None|
|412|[Precondition Failed](https://tools.ietf.org/html/rfc7232#section-4.2)|Schema variant not found for schema|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Delete the binding between a codegen func and the schema variant

<a id="opIddetach_codegen_func_binding"></a>

> Request format

`DELETE /v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/variant/{schema_variant_id}/funcs/codegen/{func_id}`

<h3 id="delete-the-binding-between-a-codegen-func-and-the-schema-variant-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|schema_id|path|string|true|Schema identifier|
|schema_variant_id|path|string|true|Schema variant identifier|
|func_id|path|string|true|Func identifier|

> Example responses

> 200 Response

```json
{
  "success": true
}
```

<h3 id="delete-the-binding-between-a-codegen-func-and-the-schema-variant-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Codegen function successfully deteched from the variant|[DetachFuncBindingV1Response](#schemadetachfuncbindingv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Func not found|None|
|412|[Precondition Failed](https://tools.ietf.org/html/rfc7232#section-4.2)|Schema variant not found for schema|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Create a management function and attach to a schema variant

<a id="opIdcreate_variant_management"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/variant/{schema_variant_id}/funcs/management`

> Body parameter

```json
{
  "code": "<!-- String escaped Typescript code here -->",
  "description": "Manages a collection of VPC components and their relationships",
  "displayName": "Manage my VPC Components",
  "name": "awsCreateMyVpc",
  "skipOverlay": false
}
```

<h3 id="create-a-management-function-and-attach-to-a-schema-variant-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|schema_id|path|string|true|Schema identifier|
|schema_variant_id|path|string|true|Schema variant identifier|
|body|body|[CreateVariantManagementFuncV1Request](#schemacreatevariantmanagementfuncv1request)|true|none|

> Example responses

> 200 Response

```json
{
  "funcId": "01H9ZQD35JPMBGHH69BT0Q79VZ"
}
```

<h3 id="create-a-management-function-and-attach-to-a-schema-variant-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Management function successfully created and attached to the variant|[CreateVariantManagementFuncV1Response](#schemacreatevariantmanagementfuncv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Schema variant not found|None|
|412|[Precondition Failed](https://tools.ietf.org/html/rfc7232#section-4.2)|Schema variant not found for schema|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Delete the binding between a management func and the schema variant

<a id="opIddetach_management_func_binding"></a>

> Request format

`DELETE /v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/variant/{schema_variant_id}/funcs/management/{func_id}`

<h3 id="delete-the-binding-between-a-management-func-and-the-schema-variant-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|schema_id|path|string|true|Schema identifier|
|schema_variant_id|path|string|true|Schema variant identifier|
|func_id|path|string|true|Func identifier|

> Example responses

> 200 Response

```json
{
  "success": true
}
```

<h3 id="delete-the-binding-between-a-management-func-and-the-schema-variant-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Management function successfully deteched from the variant|[DetachFuncBindingV1Response](#schemadetachfuncbindingv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Func not found|None|
|412|[Precondition Failed](https://tools.ietf.org/html/rfc7232#section-4.2)|Schema variant not found for schema|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Create a qualification and attach to a schema variant

<a id="opIdcreate_variant_qualification"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/variant/{schema_variant_id}/funcs/qualification`

> Body parameter

```json
{
  "code": "<!-- String escaped Typescript code here -->",
  "description": "Creates an EC2 Instance",
  "displayName": "Create EC2 Instance",
  "name": "awsEC2InstanceCreate",
  "skipOverlay": false
}
```

<h3 id="create-a-qualification-and-attach-to-a-schema-variant-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|schema_id|path|string|true|Schema identifier|
|schema_variant_id|path|string|true|Schema variant identifier|
|body|body|[CreateVariantQualificationFuncV1Request](#schemacreatevariantqualificationfuncv1request)|true|none|

> Example responses

> 200 Response

```json
{
  "funcId": "01H9ZQD35JPMBGHH69BT0Q79VZ"
}
```

<h3 id="create-a-qualification-and-attach-to-a-schema-variant-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Qualification successfully created and attached to the variant|[CreateVariantQualificationFuncV1Response](#schemacreatevariantqualificationfuncv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Schema variant not found|None|
|412|[Precondition Failed](https://tools.ietf.org/html/rfc7232#section-4.2)|Schema variant not found for schema|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Delete the binding between a qualification func and the schema variant

<a id="opIddetach_qualification_func_binding"></a>

> Request format

`DELETE /v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/variant/{schema_variant_id}/funcs/qualification/{func_id}`

<h3 id="delete-the-binding-between-a-qualification-func-and-the-schema-variant-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|schema_id|path|string|true|Schema identifier|
|schema_variant_id|path|string|true|Schema variant identifier|
|func_id|path|string|true|Func identifier|

> Example responses

> 200 Response

```json
{
  "success": true
}
```

<h3 id="delete-the-binding-between-a-qualification-func-and-the-schema-variant-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Qualification function successfully deteched from the variant|[DetachFuncBindingV1Response](#schemadetachfuncbindingv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Func not found|None|
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

## Create a transformation function

<a id="opIdcreate_transformation"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/funcs/transformation`

> Body parameter

```json
{
  "code": "<!-- String escaped Typescript code here -->",
  "description": "A custom transformation function",
  "displayName": "My Transformation",
  "name": "myTransformation"
}
```

<h3 id="create-a-transformation-function-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|body|body|[CreateTransformationFuncV1Request](#schemacreatetransformationfuncv1request)|true|none|

> Example responses

> 200 Response

```json
{
  "funcId": "01H9ZQD35JPMBGHH69BT0Q79VZ"
}
```

<h3 id="create-a-transformation-function-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Transformation function successfully created|[CreateTransformationFuncV1Response](#schemacreatetransformationfuncv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
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

## Update a func

<a id="opIdupdate_func"></a>

> Request format

`PUT /v1/w/{workspace_id}/change-sets/{change_set_id}/funcs/{func_id}`

> Body parameter

```json
{
  "code": "<!-- String escaped Typescript code here -->",
  "description": "Updated Description",
  "displayName": "Updated Display Name"
}
```

<h3 id="update-a-func-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|func_id|path|string|true|Func identifier|
|body|body|[UpdateFuncV1Request](#schemaupdatefuncv1request)|true|none|

> Example responses

> 200 Response

```json
{
  "success": true
}
```

<h3 id="update-a-func-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Function successfully updated|[UpdateFuncV1Response](#schemaupdatefuncv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Function not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Unlocks a func - if there's already an unlocked function, then we return that

<a id="opIdunlock_func"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/funcs/{func_id}/unlock`

> Body parameter

```json
{
  "schemaVariantId": "01H9ZQD35JPMBGHH69BT0Q75XY"
}
```

<h3 id="unlocks-a-func---if-there's-already-an-unlocked-function,-then-we-return-that-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|func_id|path|string|true|Func identifier|
|body|body|[UnlockFuncV1Request](#schemaunlockfuncv1request)|true|none|

> Example responses

> 200 Response

```json
{
  "unlockedFuncId": "01H9ZQD35JPMBGHH69BT0Q75XY"
}
```

<h3 id="unlocks-a-func---if-there's-already-an-unlocked-function,-then-we-return-that-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Function unlocked successfully|[UnlockFuncV1Response](#schemaunlockfuncv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|422|[Unprocessable Entity](https://tools.ietf.org/html/rfc2518#section-10.3)|Validation error - Invalid request data|[ApiError](#schemaapierror)|
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

# [debug_funcs](#system-initiative-api-debug_funcs)

Debug function endpoints

## Execute a debug function in the context of a component

<a id="opIdexec_debug_func"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/debug-funcs`

> Body parameter

```json
{
  "code": "async function main() { return 'Hello World'; }",
  "componentId": "01H9ZQD35JPMBGHH69BT0Q79VZ",
  "debugInput": null,
  "handler": "main",
  "name": "getAmiIdsForRegion"
}
```

<h3 id="execute-a-debug-function-in-the-context-of-a-component-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|body|body|[ExecDebugFuncV1Request](#schemaexecdebugfuncv1request)|true|none|

> Example responses

> 200 Response

```json
{
  "debugFuncJobStateId": "01H9ZQD35JPMBGHH69BT0Q79VZ"
}
```

<h3 id="execute-a-debug-function-in-the-context-of-a-component-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Debug function execution started|[ExecDebugFuncV1Response](#schemaexecdebugfuncv1response)|
|400|[Bad Request](https://tools.ietf.org/html/rfc7231#section-6.5.1)|Bad request - Invalid input|None|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Component not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Get debug funcs job state details

<a id="opIdget_debug_func_state"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/debug-funcs/{debug_func_job_state_id}`

<h3 id="get-debug-funcs-job-state-details-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|debug_func_job_state_id|path|string|true|Debug Func Job identifier|

> Example responses

> 200 Response

```json
{
  "failure": "Could not execute function",
  "funcRunId": "01H9ZQD35JPMBGHH69BT0Q79VY",
  "id": "01H9ZQD35JPMBGHH69BT0Q79VY",
  "result": null,
  "state": "pending"
}
```

<h3 id="get-debug-funcs-job-state-details-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Debug Function Job retrieved successfully|[GetDebugFuncJobStateV1Response](#schemagetdebugfuncjobstatev1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Debug Function Job not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

# [workspace_management](#system-initiative-api-workspace_management)

Workspace management endpoints

## List workspaces

<a id="opIdlist_workspaces"></a>

> Request format

`GET /management/workspaces`

> Example responses

> 200 Response

```json
[
  {
    "approvalsEnabled": true,
    "creatorUser": {},
    "creatorUserId": "string",
    "description": "string",
    "displayName": "string",
    "externalId": "string",
    "id": "string",
    "initialApiToken": {},
    "instanceEnvType": "string",
    "instanceUrl": "string",
    "isDefault": true,
    "quarantinedAt": "string",
    "role": "string"
  }
]
```

<h3 id="list-workspaces-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Workspaces Listed successfully|Inline|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

<h3 id="list-workspaces-responseschema">Response Schema</h3>

Status Code **200**

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|[[Workspace](#schemaworkspace)]|false|none|none|
|» approvalsEnabled|boolean|true|none|none|
|» creatorUser|any|false|none|none|

*oneOf*

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|»» *anonymous*|null|false|none|none|

*xor*

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|»» *anonymous*|[CreatorUser](#schemacreatoruser)|false|none|none|
|»»» firstName|string,null|false|none|none|
|»»» lastName|string,null|false|none|none|

*continued*

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|» creatorUserId|string|true|none|none|
|» description|string,null|false|none|none|
|» displayName|string|true|none|none|
|» externalId|string,null|false|none|none|
|» id|string|true|none|none|
|» initialApiToken|any|false|none|none|

*oneOf*

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|»» *anonymous*|null|false|none|none|

*xor*

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|»» *anonymous*|[InitialApiToken](#schemainitialapitoken)|false|none|none|
|»»» expiresAt|string,null|false|none|none|
|»»» token|string|true|none|none|

*continued*

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|» instanceEnvType|string|true|none|none|
|» instanceUrl|string,null|false|none|none|
|» isDefault|boolean|true|none|none|
|» quarantinedAt|string,null|false|none|none|
|» role|string,null|false|none|none|

## Create a new workspace

<a id="opIdcreate_workspace"></a>

> Request format

`POST /management/workspaces`

> Body parameter

```json
{
  "description": "Production environment for customer deployments",
  "displayName": "My Production Workspace",
  "instanceUrl": "https://app.systeminit.com",
  "isDefault": false
}
```

<h3 id="create-a-new-workspace-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|body|body|[CreateWorkspaceRequest](#schemacreateworkspacerequest)|true|none|

> Example responses

> 200 Response

```json
{
  "approvalsEnabled": true,
  "creatorUser": {},
  "creatorUserId": "string",
  "description": "string",
  "displayName": "string",
  "externalId": "string",
  "id": "string",
  "initialApiToken": {},
  "instanceEnvType": "string",
  "instanceUrl": "string",
  "isDefault": true,
  "quarantinedAt": "string",
  "role": "string"
}
```

<h3 id="create-a-new-workspace-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Workspace successfully created|[Workspace](#schemaworkspace)|
|400|[Bad Request](https://tools.ietf.org/html/rfc7231#section-6.5.1)|Bad Request - Validation error (invalid URL, display name, or description format)|None|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Get the details of a workspace

<a id="opIdget_workspace"></a>

> Request format

`GET /management/workspaces/{workspace_id}`

<h3 id="get-the-details-of-a-workspace-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|

> Example responses

> 200 Response

```json
{
  "approvalsEnabled": true,
  "creatorUser": {},
  "creatorUserId": "string",
  "description": "string",
  "displayName": "string",
  "externalId": "string",
  "id": "string",
  "initialApiToken": {},
  "instanceEnvType": "string",
  "instanceUrl": "string",
  "isDefault": true,
  "quarantinedAt": "string",
  "role": "string"
}
```

<h3 id="get-the-details-of-a-workspace-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Workspace retrieved successfully|[Workspace](#schemaworkspace)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|403|[Forbidden](https://tools.ietf.org/html/rfc7231#section-6.5.3)|Forbidden - User is not a member of this workspace|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Workspace not found or has been deleted|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Delete a workspace - please note, this is a soft delete and workspaces can be recovered

<a id="opIddelete_workspace"></a>

> Request format

`DELETE /management/workspaces/{workspace_id}`

<h3 id="delete-a-workspace---please-note,-this-is-a-soft-delete-and-workspaces-can-be-recovered-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|

> Example responses

> 500 Response

```json
{
  "message": "Invalid request data",
  "statusCode": 422,
  "code": 4001
}
```

<h3 id="delete-a-workspace---please-note,-this-is-a-soft-delete-and-workspaces-can-be-recovered-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|204|[No Content](https://tools.ietf.org/html/rfc7231#section-6.3.5)|Workspace deleted successfully|None|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|403|[Forbidden](https://tools.ietf.org/html/rfc7231#section-6.5.3)|Forbidden - User must be workspace owner to delete workspace|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Workspace not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## Update the details of a workspace

<a id="opIdupdate_workspace"></a>

> Request format

`PATCH /management/workspaces/{workspace_id}`

> Body parameter

```json
{
  "description": "Updated description for the workspace",
  "displayName": "Updated Workspace Name",
  "instanceUrl": "https://app.systeminit.com"
}
```

<h3 id="update-the-details-of-a-workspace-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|body|body|[UpdateWorkspaceRequest](#schemaupdateworkspacerequest)|true|none|

> Example responses

> 200 Response

```json
{
  "approvalsEnabled": true,
  "creatorUser": {},
  "creatorUserId": "string",
  "description": "string",
  "displayName": "string",
  "externalId": "string",
  "id": "string",
  "initialApiToken": {},
  "instanceEnvType": "string",
  "instanceUrl": "string",
  "isDefault": true,
  "quarantinedAt": "string",
  "role": "string"
}
```

<h3 id="update-the-details-of-a-workspace-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Workspace successfully updated|[Workspace](#schemaworkspace)|
|400|[Bad Request](https://tools.ietf.org/html/rfc7231#section-6.5.1)|Bad Request - Validation error (invalid URL, display name, or description format)|None|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|403|[Forbidden](https://tools.ietf.org/html/rfc7231#section-6.5.3)|Forbidden - User must be workspace owner to update workspace|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Workspace not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## List all members of a workspace

<a id="opIdlist_members"></a>

> Request format

`GET /management/workspaces/{workspace_id}/members`

<h3 id="list-all-members-of-a-workspace-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|

> Example responses

> 200 Response

```json
[
  {
    "email": "user@example.com",
    "nickname": "John Doe",
    "role": "OWNER",
    "signupAt": "string",
    "userId": "01GW0KXH4YJBWC7BTBAZ6ZR7EA"
  }
]
```

<h3 id="list-all-members-of-a-workspace-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Members listed successfully|Inline|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|403|[Forbidden](https://tools.ietf.org/html/rfc7231#section-6.5.3)|Forbidden - User is not a member of this workspace|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Workspace not found or has been deleted|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

<h3 id="list-all-members-of-a-workspace-responseschema">Response Schema</h3>

Status Code **200**

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|[[Member](#schemamember)]|false|none|none|
|» email|string|true|none|none|
|» nickname|string|true|none|none|
|» role|string|true|none|none|
|» signupAt|string,null|false|none|none|
|» userId|string|true|none|none|

## Invite a new member to the workspace

<a id="opIdinvite_member"></a>

> Request format

`POST /management/workspaces/{workspace_id}/members`

> Body parameter

```json
{
  "email": "newuser@example.com"
}
```

<h3 id="invite-a-new-member-to-the-workspace-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|body|body|[InviteMemberRequest](#schemainvitememberrequest)|true|none|

> Example responses

> 200 Response

```json
[
  {
    "email": "user@example.com",
    "nickname": "John Doe",
    "role": "OWNER",
    "signupAt": "string",
    "userId": "01GW0KXH4YJBWC7BTBAZ6ZR7EA"
  }
]
```

<h3 id="invite-a-new-member-to-the-workspace-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Member invited successfully|Inline|
|400|[Bad Request](https://tools.ietf.org/html/rfc7231#section-6.5.1)|Bad Request - Invalid email format|None|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|403|[Forbidden](https://tools.ietf.org/html/rfc7231#section-6.5.3)|Forbidden - User must be workspace owner or approver to invite members|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Workspace not found or has been deleted|None|
|409|[Conflict](https://tools.ietf.org/html/rfc7231#section-6.5.8)|Conflict - User already invited, suspended, or other conflict|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

<h3 id="invite-a-new-member-to-the-workspace-responseschema">Response Schema</h3>

Status Code **200**

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|[[Member](#schemamember)]|false|none|none|
|» email|string|true|none|none|
|» nickname|string|true|none|none|
|» role|string|true|none|none|
|» signupAt|string,null|false|none|none|
|» userId|string|true|none|none|

## Remove a member from the workspace

<a id="opIdremove_member"></a>

> Request format

`DELETE /management/workspaces/{workspace_id}/members`

> Body parameter

```json
{
  "email": "user@example.com"
}
```

<h3 id="remove-a-member-from-the-workspace-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|body|body|[RemoveMemberRequest](#schemaremovememberrequest)|true|none|

> Example responses

> 200 Response

```json
[
  {
    "email": "user@example.com",
    "nickname": "John Doe",
    "role": "OWNER",
    "signupAt": "string",
    "userId": "01GW0KXH4YJBWC7BTBAZ6ZR7EA"
  }
]
```

<h3 id="remove-a-member-from-the-workspace-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Member removed successfully|Inline|
|400|[Bad Request](https://tools.ietf.org/html/rfc7231#section-6.5.1)|Bad Request - Invalid email format|None|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|403|[Forbidden](https://tools.ietf.org/html/rfc7231#section-6.5.3)|Forbidden - User must be workspace owner or approver to remove members|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Workspace not found, has been deleted, or user not found in workspace|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

<h3 id="remove-a-member-from-the-workspace-responseschema">Response Schema</h3>

Status Code **200**

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|[[Member](#schemamember)]|false|none|none|
|» email|string|true|none|none|
|» nickname|string|true|none|none|
|» role|string|true|none|none|
|» signupAt|string,null|false|none|none|
|» userId|string|true|none|none|

## Update a member's role in the workspace

<a id="opIdupdate_member_role"></a>

> Request format

`POST /management/workspaces/{workspace_id}/update_member_access`

> Body parameter

```json
{
  "role": "EDITOR",
  "userId": "01GW0KXH4YJBWC7BTBAZ6ZR7EA"
}
```

<h3 id="update-a-member's-role-in-the-workspace-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|body|body|[UpdateMemberRoleRequest](#schemaupdatememberrolerequest)|true|none|

> Example responses

> 200 Response

```json
[
  {
    "email": "user@example.com",
    "nickname": "John Doe",
    "role": "OWNER",
    "signupAt": "string",
    "userId": "01GW0KXH4YJBWC7BTBAZ6ZR7EA"
  }
]
```

<h3 id="update-a-member's-role-in-the-workspace-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Member role updated successfully|Inline|
|400|[Bad Request](https://tools.ietf.org/html/rfc7231#section-6.5.1)|Bad Request - Invalid userId or role format|None|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|403|[Forbidden](https://tools.ietf.org/html/rfc7231#section-6.5.3)|Forbidden - User must be workspace owner to update member roles|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Workspace not found or has been deleted|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

<h3 id="update-a-member's-role-in-the-workspace-responseschema">Response Schema</h3>

Status Code **200**

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|[[Member](#schemamember)]|false|none|none|
|» email|string|true|none|none|
|» nickname|string|true|none|none|
|» role|string|true|none|none|
|» signupAt|string,null|false|none|none|
|» userId|string|true|none|none|

# [search](#system-initiative-api-search)

## Complex search for components

<a id="opIdsearch"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/search`

<h3 id="complex-search-for-components-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|string|true|Workspace identifier|
|change_set_id|path|string|true|Change Set identifier|
|q|query|string|true|Query string. See https://docs.systeminit.com/explanation/search-syntax for details.|

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
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Components retrieved successfully|[SearchV1Response](#schemasearchv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
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
|componentId|string,null|false|none|none|
|description|string,null|false|none|none|
|funcRunId|string,null|false|none|none|
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

## [AddToViewV1Request](#tocS_AddToViewV1Request)

<a id="schemaaddtoviewv1request"></a>
<a id="schema_AddToViewV1Request"></a>
<a id="tocSaddtoviewv1request"></a>
<a id="tocsaddtoviewv1request"></a>

```json
{
  "componentIds": [
    "string"
  ],
  "viewName": "string"
}

```

### [Properties](#addtoviewv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|componentIds|[string]|true|none|none|
|viewName|string|true|none|none|

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

## [AttributeArgumentBindingRequest](#tocS_AttributeArgumentBindingRequest)

<a id="schemaattributeargumentbindingrequest"></a>
<a id="schema_AttributeArgumentBindingRequest"></a>
<a id="tocSattributeargumentbindingrequest"></a>
<a id="tocsattributeargumentbindingrequest"></a>

```json
{
  "elementKind": "String",
  "kind": "String",
  "name": "instanceType",
  "propId": "01H9ZQD35JPMBGHH69BT0Q79VZ",
  "staticValue": null
}

```

### [Properties](#attributeargumentbindingrequest-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|elementKind|string,null|false|none|Element type for Array arguments. Required when kind is 'Array'. Specifies the type of array elements.|
|kind|string|true|none|Type of the argument. Valid values: "Any", "Array", "Boolean", "Float", "Integer", "Json", "Map", "Object", "String". Use 'Array' with element_kind for typed arrays.|
|name|string|true|none|Name of the function argument (e.g., "instanceType", "region", "tags")|
|propId|string,null|false|none|Prop ID to bind this argument to. Either prop_id or static_value must be provided.|
|staticValue|any|false|none|Static value for this argument. Can be string, number, boolean, array, or object. Either prop_id or static_value must be provided.|

## [BuildingResponseV1](#tocS_BuildingResponseV1)

<a id="schemabuildingresponsev1"></a>
<a id="schema_BuildingResponseV1"></a>
<a id="tocSbuildingresponsev1"></a>
<a id="tocsbuildingresponsev1"></a>

```json
{
  "estimatedCompletionSeconds": 10,
  "message": "Schema data is being generated, please retry shortly",
  "retryAfterSeconds": 2,
  "status": "building"
}

```

### [Properties](#buildingresponsev1-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|estimatedCompletionSeconds|integer(int64)|true|none|none|
|message|string|true|none|none|
|retryAfterSeconds|integer(int64)|true|none|none|
|status|string|true|none|none|

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
  "codegen": null,
  "componentId": "string",
  "name": "string",
  "schemaName": "string"
}

```

### [Properties](#componentdetailsv1-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|codegen|any|false|none|none|
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
  "value": null
}

```

### [Properties](#componentpropviewv1-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|id|string|true|none|none|
|path|string|true|none|none|
|propId|string|true|none|none|
|value|any|false|none|none|

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

## [ComponentSearchResult](#tocS_ComponentSearchResult)

<a id="schemacomponentsearchresult"></a>
<a id="schema_ComponentSearchResult"></a>
<a id="tocScomponentsearchresult"></a>
<a id="tocscomponentsearchresult"></a>

```json
{
  "id": "01H9ZQD35JPMBGHH69BT0Q79AA",
  "name": "MyInstance",
  "schema": {
    "name": "AWS::EC2::Instance"
  }
}

```

Component data in search results.

### [Properties](#componentsearchresult-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|id|string|true|none|none|
|name|string|true|none|none|
|schema|[ComponentSearchResultSchema](#schemacomponentsearchresultschema)|true|none|The schema for a component in search results.|

## [ComponentSearchResultSchema](#tocS_ComponentSearchResultSchema)

<a id="schemacomponentsearchresultschema"></a>
<a id="schema_ComponentSearchResultSchema"></a>
<a id="tocScomponentsearchresultschema"></a>
<a id="tocscomponentsearchresultschema"></a>

```json
{
  "name": "AWS::EC2::Instance"
}

```

The schema for a component in search results.

### [Properties](#componentsearchresultschema-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|name|string|true|none|none|

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
      "managing": {
        "componentId": "string",
        "componentName": "string"
      }
    }
  ],
  "domainProps": [
    {
      "id": "string",
      "path": "path/to/prop",
      "propId": "string",
      "value": null
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
      "value": null
    }
  ],
  "schemaId": "string",
  "schemaVariantId": "string",
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
|toDelete|boolean|true|none|none|
|views|[[ViewV1](#schemaviewv1)]|true|none|none|

## [ConnectionViewV1](#tocS_ConnectionViewV1)

<a id="schemaconnectionviewv1"></a>
<a id="schema_ConnectionViewV1"></a>
<a id="tocSconnectionviewv1"></a>
<a id="tocsconnectionviewv1"></a>

```json
{
  "managing": {
    "componentId": "string",
    "componentName": "string"
  }
}

```

### [Properties](#connectionviewv1-properties)

oneOf

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
  "managedBy": {
    "component": "ComponentName"
  },
  "name": "MyComponentName",
  "resourceId": "i-12345678",
  "schemaName": "AWS::EC2::Instance",
  "useWorkingCopy": true,
  "viewName": "MyView"
}

```

### [Properties](#createcomponentv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|attributes|object|false|none|none|
|» **additionalProperties**|any|false|none|none|
|managedBy|[ComponentReference](#schemacomponentreference)|false|none|none|
|name|string|true|none|none|
|resourceId|string,null|false|none|none|
|schemaName|string|true|none|none|
|useWorkingCopy|boolean,null|false|none|none|
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
        "managing": {
          "componentId": "string",
          "componentName": "string"
        }
      }
    ],
    "domainProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": null
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
        "value": null
      }
    ],
    "schemaId": "string",
    "schemaVariantId": "string",
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

## [CreateSchemaV1Request](#tocS_CreateSchemaV1Request)

<a id="schemacreateschemav1request"></a>
<a id="schema_CreateSchemaV1Request"></a>
<a id="tocScreateschemav1request"></a>
<a id="tocscreateschemav1request"></a>

```json
{
  "category": "string",
  "code": "string",
  "color": "string",
  "description": "string",
  "link": "string",
  "name": "string"
}

```

### [Properties](#createschemav1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|category|string,null|false|none|none|
|code|string|true|none|none|
|color|string,null|false|none|none|
|description|string,null|false|none|none|
|link|string,null|false|none|none|
|name|string|true|none|none|

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
|description|string,null|false|none|none|
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

## [CreateTransformationFuncV1Request](#tocS_CreateTransformationFuncV1Request)

<a id="schemacreatetransformationfuncv1request"></a>
<a id="schema_CreateTransformationFuncV1Request"></a>
<a id="tocScreatetransformationfuncv1request"></a>
<a id="tocscreatetransformationfuncv1request"></a>

```json
{
  "code": "<!-- String escaped Typescript code here -->",
  "description": "A custom transformation function",
  "displayName": "My Transformation",
  "name": "myTransformation"
}

```

### [Properties](#createtransformationfuncv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|code|string|true|none|none|
|description|string,null|false|none|none|
|displayName|string,null|false|none|none|
|name|string|true|none|none|

## [CreateTransformationFuncV1Response](#tocS_CreateTransformationFuncV1Response)

<a id="schemacreatetransformationfuncv1response"></a>
<a id="schema_CreateTransformationFuncV1Response"></a>
<a id="tocScreatetransformationfuncv1response"></a>
<a id="tocscreatetransformationfuncv1response"></a>

```json
{
  "funcId": "01H9ZQD35JPMBGHH69BT0Q79VZ"
}

```

### [Properties](#createtransformationfuncv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|funcId|string|true|none|none|

## [CreateVariantActionFuncV1Request](#tocS_CreateVariantActionFuncV1Request)

<a id="schemacreatevariantactionfuncv1request"></a>
<a id="schema_CreateVariantActionFuncV1Request"></a>
<a id="tocScreatevariantactionfuncv1request"></a>
<a id="tocscreatevariantactionfuncv1request"></a>

```json
{
  "code": "<!-- String escaped Typescript code here -->",
  "description": "Creates an EC2 Instance",
  "displayName": "Create EC2 Instance",
  "kind": "Create",
  "name": "awsEC2InstanceCreate",
  "skipOverlay": false
}

```

### [Properties](#createvariantactionfuncv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|code|string|true|none|none|
|description|string,null|false|none|none|
|displayName|string,null|false|none|none|
|kind|string|true|none|none|
|name|string|true|none|none|
|skipOverlay|boolean,null|false|none|none|

## [CreateVariantActionFuncV1Response](#tocS_CreateVariantActionFuncV1Response)

<a id="schemacreatevariantactionfuncv1response"></a>
<a id="schema_CreateVariantActionFuncV1Response"></a>
<a id="tocScreatevariantactionfuncv1response"></a>
<a id="tocscreatevariantactionfuncv1response"></a>

```json
{
  "funcId": "01H9ZQD35JPMBGHH69BT0Q79VZ"
}

```

### [Properties](#createvariantactionfuncv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|funcId|string|true|none|none|

## [CreateVariantAttributeFuncV1Request](#tocS_CreateVariantAttributeFuncV1Request)

<a id="schemacreatevariantattributefuncv1request"></a>
<a id="schema_CreateVariantAttributeFuncV1Request"></a>
<a id="tocScreatevariantattributefuncv1request"></a>
<a id="tocscreatevariantattributefuncv1request"></a>

```json
{
  "argumentBindings": [
    {
      "elementKind": "String",
      "kind": "String",
      "name": "instanceType",
      "propId": "01H9ZQD35JPMBGHH69BT0Q79VZ",
      "staticValue": null
    }
  ],
  "code": "async function main(instanceType: Input): Promise<Output> { return instanceType; }",
  "componentId": "01H9ZQD35JPMBGHH69BT0Q79VZ",
  "description": "Sets the instance type for an EC2 Instance",
  "displayName": "Set Instance Type",
  "name": "awsEC2SetInstanceType",
  "propId": "01H9ZQD35JPMBGHH69BT0Q79VZ",
  "skipOverlay": false
}

```

### [Properties](#createvariantattributefuncv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|argumentBindings|[[AttributeArgumentBindingRequest](#schemaattributeargumentbindingrequest)]|true|none|Function arguments with their bindings (input sources). Each argument defines its type and where its value comes from.|
|code|string|true|none|TypeScript code for the function. Should export a main function that takes arguments and returns a value.|
|componentId|string,null|false|none|Optional component ID for component-level bindings. If not provided, creates a schema variant-level binding.|
|description|string,null|false|none|Description of what the function does|
|displayName|string,null|false|none|Human-readable display name|
|name|string|true|none|Unique name for the function (e.g., "awsEC2SetInstanceType")|
|propId|string|true|none|Prop ID where the function output will be written (required)|
|skipOverlay|boolean,null|false|none|none|

## [CreateVariantAttributeFuncV1Response](#tocS_CreateVariantAttributeFuncV1Response)

<a id="schemacreatevariantattributefuncv1response"></a>
<a id="schema_CreateVariantAttributeFuncV1Response"></a>
<a id="tocScreatevariantattributefuncv1response"></a>
<a id="tocscreatevariantattributefuncv1response"></a>

```json
{
  "attributePrototypeId": "01H9ZQD35JPMBGHH69BT0Q79VZ",
  "funcId": "01H9ZQD35JPMBGHH69BT0Q79VZ"
}

```

### [Properties](#createvariantattributefuncv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|attributePrototypeId|string|true|none|none|
|funcId|string|true|none|none|

## [CreateVariantAuthenticationFuncV1Request](#tocS_CreateVariantAuthenticationFuncV1Request)

<a id="schemacreatevariantauthenticationfuncv1request"></a>
<a id="schema_CreateVariantAuthenticationFuncV1Request"></a>
<a id="tocScreatevariantauthenticationfuncv1request"></a>
<a id="tocscreatevariantauthenticationfuncv1request"></a>

```json
{
  "code": "<!-- String escaped Typescript code here -->",
  "description": "Function to manage AWS Credentials",
  "displayName": "Set AWS credentials",
  "name": "awsSetCredentials"
}

```

### [Properties](#createvariantauthenticationfuncv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|code|string|true|none|none|
|description|string,null|false|none|none|
|displayName|string,null|false|none|none|
|name|string|true|none|none|

## [CreateVariantAuthenticationFuncV1Response](#tocS_CreateVariantAuthenticationFuncV1Response)

<a id="schemacreatevariantauthenticationfuncv1response"></a>
<a id="schema_CreateVariantAuthenticationFuncV1Response"></a>
<a id="tocScreatevariantauthenticationfuncv1response"></a>
<a id="tocscreatevariantauthenticationfuncv1response"></a>

```json
{
  "funcId": "01H9ZQD35JPMBGHH69BT0Q79VZ"
}

```

### [Properties](#createvariantauthenticationfuncv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|funcId|string|true|none|none|

## [CreateVariantCodegenFuncV1Request](#tocS_CreateVariantCodegenFuncV1Request)

<a id="schemacreatevariantcodegenfuncv1request"></a>
<a id="schema_CreateVariantCodegenFuncV1Request"></a>
<a id="tocScreatevariantcodegenfuncv1request"></a>
<a id="tocscreatevariantcodegenfuncv1request"></a>

```json
{
  "code": "<!-- String escaped Typescript code here -->",
  "description": "Generates the payload required for creating an EC2 instance",
  "displayName": "Generate EC2 Instance Create Payload",
  "name": "awsEC2InstanceGenerateCode",
  "skipOverlay": false
}

```

### [Properties](#createvariantcodegenfuncv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|code|string|true|none|none|
|description|string,null|false|none|none|
|displayName|string,null|false|none|none|
|name|string|true|none|none|
|skipOverlay|boolean,null|false|none|none|

## [CreateVariantCodegenFuncV1Response](#tocS_CreateVariantCodegenFuncV1Response)

<a id="schemacreatevariantcodegenfuncv1response"></a>
<a id="schema_CreateVariantCodegenFuncV1Response"></a>
<a id="tocScreatevariantcodegenfuncv1response"></a>
<a id="tocscreatevariantcodegenfuncv1response"></a>

```json
{
  "funcId": "01H9ZQD35JPMBGHH69BT0Q79VZ"
}

```

### [Properties](#createvariantcodegenfuncv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|funcId|string|true|none|none|

## [CreateVariantManagementFuncV1Request](#tocS_CreateVariantManagementFuncV1Request)

<a id="schemacreatevariantmanagementfuncv1request"></a>
<a id="schema_CreateVariantManagementFuncV1Request"></a>
<a id="tocScreatevariantmanagementfuncv1request"></a>
<a id="tocscreatevariantmanagementfuncv1request"></a>

```json
{
  "code": "<!-- String escaped Typescript code here -->",
  "description": "Manages a collection of VPC components and their relationships",
  "displayName": "Manage my VPC Components",
  "name": "awsCreateMyVpc",
  "skipOverlay": false
}

```

### [Properties](#createvariantmanagementfuncv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|code|string|true|none|none|
|description|string,null|false|none|none|
|displayName|string,null|false|none|none|
|name|string|true|none|none|
|skipOverlay|boolean,null|false|none|none|

## [CreateVariantManagementFuncV1Response](#tocS_CreateVariantManagementFuncV1Response)

<a id="schemacreatevariantmanagementfuncv1response"></a>
<a id="schema_CreateVariantManagementFuncV1Response"></a>
<a id="tocScreatevariantmanagementfuncv1response"></a>
<a id="tocscreatevariantmanagementfuncv1response"></a>

```json
{
  "funcId": "01H9ZQD35JPMBGHH69BT0Q79VZ"
}

```

### [Properties](#createvariantmanagementfuncv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|funcId|string|true|none|none|

## [CreateVariantQualificationFuncV1Request](#tocS_CreateVariantQualificationFuncV1Request)

<a id="schemacreatevariantqualificationfuncv1request"></a>
<a id="schema_CreateVariantQualificationFuncV1Request"></a>
<a id="tocScreatevariantqualificationfuncv1request"></a>
<a id="tocscreatevariantqualificationfuncv1request"></a>

```json
{
  "code": "<!-- String escaped Typescript code here -->",
  "description": "Creates an EC2 Instance",
  "displayName": "Create EC2 Instance",
  "name": "awsEC2InstanceCreate",
  "skipOverlay": false
}

```

### [Properties](#createvariantqualificationfuncv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|code|string|true|none|none|
|description|string,null|false|none|none|
|displayName|string,null|false|none|none|
|name|string|true|none|none|
|skipOverlay|boolean,null|false|none|none|

## [CreateVariantQualificationFuncV1Response](#tocS_CreateVariantQualificationFuncV1Response)

<a id="schemacreatevariantqualificationfuncv1response"></a>
<a id="schema_CreateVariantQualificationFuncV1Response"></a>
<a id="tocScreatevariantqualificationfuncv1response"></a>
<a id="tocscreatevariantqualificationfuncv1response"></a>

```json
{
  "funcId": "01H9ZQD35JPMBGHH69BT0Q79VZ"
}

```

### [Properties](#createvariantqualificationfuncv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|funcId|string|true|none|none|

## [CreateWorkspaceRequest](#tocS_CreateWorkspaceRequest)

<a id="schemacreateworkspacerequest"></a>
<a id="schema_CreateWorkspaceRequest"></a>
<a id="tocScreateworkspacerequest"></a>
<a id="tocscreateworkspacerequest"></a>

```json
{
  "description": "Production environment for customer deployments",
  "displayName": "My Production Workspace",
  "instanceUrl": "https://app.systeminit.com",
  "isDefault": false
}

```

### [Properties](#createworkspacerequest-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|description|string|true|none|none|
|displayName|string|true|none|none|
|instanceUrl|string|true|none|none|
|isDefault|boolean|false|none|none|

## [CreatorUser](#tocS_CreatorUser)

<a id="schemacreatoruser"></a>
<a id="schema_CreatorUser"></a>
<a id="tocScreatoruser"></a>
<a id="tocscreatoruser"></a>

```json
{
  "firstName": "string",
  "lastName": "string"
}

```

### [Properties](#creatoruser-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|firstName|string,null|false|none|none|
|lastName|string,null|false|none|none|

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

## [DetachFuncBindingV1Response](#tocS_DetachFuncBindingV1Response)

<a id="schemadetachfuncbindingv1response"></a>
<a id="schema_DetachFuncBindingV1Response"></a>
<a id="tocSdetachfuncbindingv1response"></a>
<a id="tocsdetachfuncbindingv1response"></a>

```json
{
  "success": true
}

```

### [Properties](#detachfuncbindingv1response-properties)

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

## [EraseComponentV1Response](#tocS_EraseComponentV1Response)

<a id="schemaerasecomponentv1response"></a>
<a id="schema_EraseComponentV1Response"></a>
<a id="tocSerasecomponentv1response"></a>
<a id="tocserasecomponentv1response"></a>

```json
{
  "status": "true"
}

```

### [Properties](#erasecomponentv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|status|boolean|true|none|none|

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

## [ExecDebugFuncV1Request](#tocS_ExecDebugFuncV1Request)

<a id="schemaexecdebugfuncv1request"></a>
<a id="schema_ExecDebugFuncV1Request"></a>
<a id="tocSexecdebugfuncv1request"></a>
<a id="tocsexecdebugfuncv1request"></a>

```json
{
  "code": "async function main() { return 'Hello World'; }",
  "componentId": "01H9ZQD35JPMBGHH69BT0Q79VZ",
  "debugInput": null,
  "handler": "main",
  "name": "getAmiIdsForRegion"
}

```

### [Properties](#execdebugfuncv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|code|string|true|none|none|
|componentId|string|true|none|none|
|debugInput|any|false|none|none|
|handler|string|true|none|none|
|name|string|true|none|none|

## [ExecDebugFuncV1Response](#tocS_ExecDebugFuncV1Response)

<a id="schemaexecdebugfuncv1response"></a>
<a id="schema_ExecDebugFuncV1Response"></a>
<a id="tocSexecdebugfuncv1response"></a>
<a id="tocsexecdebugfuncv1response"></a>

```json
{
  "debugFuncJobStateId": "01H9ZQD35JPMBGHH69BT0Q79VZ"
}

```

### [Properties](#execdebugfuncv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|debugFuncJobStateId|string|true|none|none|

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
|funcRunId|string,null|false|none|none|
|managementFuncJobStateId|string|true|none|none|
|message|string,null|false|none|none|
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
|componentId|string,null|false|none|none|

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
|schemaId|string,null|false|none|none|

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
|category|string,null|false|none|none|
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
|actionDisplayName|string,null|false|none|none|
|actionId|string,null|false|none|none|
|actionKind|string,null|false|none|none|
|actionOriginatingChangeSetId|string,null|false|none|none|
|actionOriginatingChangeSetName|string,null|false|none|none|
|actionPrototypeId|string,null|false|none|none|
|actionResultState|string,null|false|none|none|
|attributeValueId|string,null|false|none|none|
|backendKind|string|true|none|none|
|backendResponseType|string|true|none|none|
|componentId|string,null|false|none|none|
|componentName|string,null|false|none|none|
|createdAt|string|true|none|none|
|functionArgs|any|true|none|none|
|functionCodeBase64|string|true|none|none|
|functionDescription|string,null|false|none|none|
|functionDisplayName|string,null|false|none|none|
|functionKind|string|true|none|none|
|functionLink|string,null|false|none|none|
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
|schemaName|string,null|false|none|none|
|state|string|true|none|none|
|updatedAt|string|true|none|none|

## [GenerateTemplateV1Request](#tocS_GenerateTemplateV1Request)

<a id="schemageneratetemplatev1request"></a>
<a id="schema_GenerateTemplateV1Request"></a>
<a id="tocSgeneratetemplatev1request"></a>
<a id="tocsgeneratetemplatev1request"></a>

```json
{
  "assetName": "My Cool Template",
  "category": "Templates",
  "componentIds": [
    "01H9ZQD35JPMBGHH69BT0Q79AA",
    "01H9ZQD35JPMBGHH69BT0Q79BB",
    "01H9ZQD35JPMBGHH69BT0Q79CC"
  ],
  "funcName": "Generate My Template"
}

```

### [Properties](#generatetemplatev1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|assetName|string|true|none|none|
|category|string,null|false|none|none|
|componentIds|[string]|true|none|none|
|funcName|string|true|none|none|

## [GenerateTemplateV1Response](#tocS_GenerateTemplateV1Response)

<a id="schemageneratetemplatev1response"></a>
<a id="schema_GenerateTemplateV1Response"></a>
<a id="tocSgeneratetemplatev1response"></a>
<a id="tocsgeneratetemplatev1response"></a>

```json
{
  "funcId": "01H9ZQD35JPMBGHH69BT0Q79CC",
  "schemaId": "01H9ZQD35JPMBGHH69BT0Q79AA",
  "schemaVariantId": "01H9ZQD35JPMBGHH69BT0Q79BB"
}

```

### [Properties](#generatetemplatev1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|funcId|string|true|none|none|
|schemaId|string|true|none|none|
|schemaVariantId|string|true|none|none|

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

## [GetComponentResourceDataV1Response](#tocS_GetComponentResourceDataV1Response)

<a id="schemagetcomponentresourcedatav1response"></a>
<a id="schema_GetComponentResourceDataV1Response"></a>
<a id="tocSgetcomponentresourcedatav1response"></a>
<a id="tocsgetcomponentresourcedatav1response"></a>

```json
{
  "last_synced": "2024-01-15T12:30:00Z",
  "payload": null,
  "status": "Ok"
}

```

### [Properties](#getcomponentresourcedatav1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|last_synced|string(date-time)|true|none|none|
|payload|any|false|none|none|
|status|string|true|none|none|

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
        "managing": {
          "componentId": "string",
          "componentName": "string"
        }
      }
    ],
    "domainProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": null
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
        "value": null
      }
    ],
    "schemaId": "string",
    "schemaVariantId": "string",
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

## [GetDebugFuncJobStateV1Response](#tocS_GetDebugFuncJobStateV1Response)

<a id="schemagetdebugfuncjobstatev1response"></a>
<a id="schema_GetDebugFuncJobStateV1Response"></a>
<a id="tocSgetdebugfuncjobstatev1response"></a>
<a id="tocsgetdebugfuncjobstatev1response"></a>

```json
{
  "failure": "Could not execute function",
  "funcRunId": "01H9ZQD35JPMBGHH69BT0Q79VY",
  "id": "01H9ZQD35JPMBGHH69BT0Q79VY",
  "result": null,
  "state": "pending"
}

```

### [Properties](#getdebugfuncjobstatev1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|failure|string,null|false|none|none|
|funcRunId|string,null|false|none|none|
|id|string|true|none|none|
|result|any|false|none|none|
|state|string|true|none|none|

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
|description|string,null|false|none|none|
|displayName|string,null|false|none|none|
|isLocked|boolean|true|none|none|
|kind|string|true|none|none|
|link|string,null|false|none|none|
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
|funcRunId|string,null|false|none|none|
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
  "schemaId": "string",
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
|schemaId|string|true|none|none|
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
  "domainProps": {},
  "installedFromUpstream": false,
  "isDefaultVariant": true,
  "isLocked": false,
  "link": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-instance.html",
  "variantFuncIds": [
    "01H9ZQD35JPMBGHH69BT0Q75AA",
    "01H9ZQD35JPMBGHH69BT0Q75BB"
  ],
  "variantFuncs": [
    {
      "funcKind": {
        "actionKind": "Create",
        "kind": "action"
      },
      "id": "01H9ZQD35JPMBGHH69BT0Q79VZ",
      "isOverlay": true
    }
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
|description|string,null|false|none|none|
|displayName|string|true|none|none|
|domainProps|any|false|none|none|

oneOf

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|» *anonymous*|null|false|none|none|

xor

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|» *anonymous*|[PropSchemaV1](#schemapropschemav1)|false|none|none|

continued

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|installedFromUpstream|boolean|true|none|none|
|isDefaultVariant|boolean|true|none|none|
|isLocked|boolean|true|none|none|
|link|string,null|false|none|none|
|variantFuncIds|[string]|true|none|none|
|variantFuncs|[[SchemaVariantFunc](#schemaschemavariantfunc)]|true|none|none|
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

## [InitialApiToken](#tocS_InitialApiToken)

<a id="schemainitialapitoken"></a>
<a id="schema_InitialApiToken"></a>
<a id="tocSinitialapitoken"></a>
<a id="tocsinitialapitoken"></a>

```json
{
  "expiresAt": "string",
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}

```

### [Properties](#initialapitoken-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|expiresAt|string,null|false|none|none|
|token|string|true|none|none|

## [InviteMemberRequest](#tocS_InviteMemberRequest)

<a id="schemainvitememberrequest"></a>
<a id="schema_InviteMemberRequest"></a>
<a id="tocSinvitememberrequest"></a>
<a id="tocsinvitememberrequest"></a>

```json
{
  "email": "newuser@example.com"
}

```

### [Properties](#invitememberrequest-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|email|string|true|none|none|

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
  "nextCursor": "string"
}

```

### [Properties](#listcomponentsv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|componentDetails|[[ComponentDetailsV1](#schemacomponentdetailsv1)]|true|none|none|
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
        "managing": {
          "componentId": "string",
          "componentName": "string"
        }
      }
    ],
    "domainProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": null
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
        "value": null
      }
    ],
    "schemaId": "string",
    "schemaVariantId": "string",
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

## [Member](#tocS_Member)

<a id="schemamember"></a>
<a id="schema_Member"></a>
<a id="tocSmember"></a>
<a id="tocsmember"></a>

```json
{
  "email": "user@example.com",
  "nickname": "John Doe",
  "role": "OWNER",
  "signupAt": "string",
  "userId": "01GW0KXH4YJBWC7BTBAZ6ZR7EA"
}

```

### [Properties](#member-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|email|string|true|none|none|
|nickname|string|true|none|none|
|role|string|true|none|none|
|signupAt|string,null|false|none|none|
|userId|string|true|none|none|

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
      "defaultValue": null,
      "description": "string",
      "docLink": "string",
      "hidden": true,
      "name": "string",
      "propId": "string",
      "propType": "string",
      "validationFormat": "string"
    }
  ],
  "defaultValue": null,
  "description": "string",
  "docLink": "string",
  "hidden": true,
  "name": "string",
  "propId": "string",
  "propType": "string",
  "validationFormat": "string"
}

```

### [Properties](#propschemav1-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|children|array,null|false|none|none|
|defaultValue|any|false|none|none|
|description|string,null|false|none|none|
|docLink|string,null|false|none|none|
|hidden|boolean,null|false|none|none|
|name|string|true|none|none|
|propId|string|true|none|none|
|propType|string|true|none|none|
|validationFormat|string,null|false|none|none|

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

## [RemoveMemberRequest](#tocS_RemoveMemberRequest)

<a id="schemaremovememberrequest"></a>
<a id="schema_RemoveMemberRequest"></a>
<a id="tocSremovememberrequest"></a>
<a id="tocsremovememberrequest"></a>

```json
{
  "email": "user@example.com"
}

```

### [Properties](#removememberrequest-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|email|string|true|none|none|

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

## [RestoreComponentV1Response](#tocS_RestoreComponentV1Response)

<a id="schemarestorecomponentv1response"></a>
<a id="schema_RestoreComponentV1Response"></a>
<a id="tocSrestorecomponentv1response"></a>
<a id="tocsrestorecomponentv1response"></a>

```json
{
  "status": "true"
}

```

### [Properties](#restorecomponentv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|status|boolean|true|none|none|

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

## [SchemaVariantFunc](#tocS_SchemaVariantFunc)

<a id="schemaschemavariantfunc"></a>
<a id="schema_SchemaVariantFunc"></a>
<a id="tocSschemavariantfunc"></a>
<a id="tocsschemavariantfunc"></a>

```json
{
  "funcKind": {
    "actionKind": "Create",
    "kind": "action"
  },
  "id": "01H9ZQD35JPMBGHH69BT0Q79VZ",
  "isOverlay": true
}

```

### [Properties](#schemavariantfunc-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|funcKind|[SchemaVariantFuncKind](#schemaschemavariantfunckind)|true|none|none|
|id|string|true|none|none|
|isOverlay|boolean|true|none|none|

## [SchemaVariantFuncKind](#tocS_SchemaVariantFuncKind)

<a id="schemaschemavariantfunckind"></a>
<a id="schema_SchemaVariantFuncKind"></a>
<a id="tocSschemavariantfunckind"></a>
<a id="tocsschemavariantfunckind"></a>

```json
{
  "actionKind": "Create",
  "kind": "action"
}

```

### [Properties](#schemavariantfunckind-properties)

oneOf

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|object|false|none|Action function; carries the specific `ActionKind`.|
|» actionKind|string|true|none|Specific action kind|
|» kind|string|true|none|none|

xor

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|object|false|none|Management function; carries the specific `ManagementFuncKind`.|
|» kind|string|true|none|none|
|» managementFuncKind|string|true|none|Specific management function kind|

xor

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|object|false|none|Any other function; exposes the raw `FuncKind` category.|
|» funcKind|string|true|none|none|
|» kind|string|true|none|none|

#### [Enumerated Values](#schemavariantfunckind-enumerated-values)

|Property|Value|
|---|---|
|kind|action|
|kind|management|
|kind|other|

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
  "queryString": "string",
  "schemaCategory": "AWS::EC2",
  "schemaName": "AWS::EC2::Instance",
  "upgradable": true
}

```

### [Properties](#searchcomponentsv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|queryString|string,null|false|none|none|
|schemaCategory|string,null|false|none|none|
|schemaName|string,null|false|none|none|
|upgradable|boolean,null|false|none|none|

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

## [SearchSchemasV1Request](#tocS_SearchSchemasV1Request)

<a id="schemasearchschemasv1request"></a>
<a id="schema_SearchSchemasV1Request"></a>
<a id="tocSsearchschemasv1request"></a>
<a id="tocssearchschemasv1request"></a>

```json
{
  "category": "AWS::EC2"
}

```

### [Properties](#searchschemasv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|category|string,null|false|none|none|

## [SearchSchemasV1Response](#tocS_SearchSchemasV1Response)

<a id="schemasearchschemasv1response"></a>
<a id="schema_SearchSchemasV1Response"></a>
<a id="tocSsearchschemasv1response"></a>
<a id="tocssearchschemasv1response"></a>

```json
{
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

### [Properties](#searchschemasv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|schemas|[[SchemaResponse](#schemaschemaresponse)]|true|none|none|

## [SearchV1Request](#tocS_SearchV1Request)

<a id="schemasearchv1request"></a>
<a id="schema_SearchV1Request"></a>
<a id="tocSsearchv1request"></a>
<a id="tocssearchv1request"></a>

```json
{
  "q": "AWS::EC2::Instance region:us-east-1"
}

```

### [Properties](#searchv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|q|string|true|none|none|

## [SearchV1Response](#tocS_SearchV1Response)

<a id="schemasearchv1response"></a>
<a id="schema_SearchV1Response"></a>
<a id="tocSsearchv1response"></a>
<a id="tocssearchv1response"></a>

```json
{
  "components": [
    "01H9ZQD35JPMBGHH69BT0Q79AA",
    "01H9ZQD35JPMBGHH69BT0Q79BB",
    "01H9ZQD35JPMBGHH69BT0Q79CC"
  ]
}

```

### [Properties](#searchv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|components|[[ComponentSearchResult](#schemacomponentsearchresult)]|true|none|[Component data in search results.]|

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
|description|string,null|false|none|none|
|id|string|true|none|none|
|name|string|true|none|none|

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

## [UnlockFuncV1Request](#tocS_UnlockFuncV1Request)

<a id="schemaunlockfuncv1request"></a>
<a id="schema_UnlockFuncV1Request"></a>
<a id="tocSunlockfuncv1request"></a>
<a id="tocsunlockfuncv1request"></a>

```json
{
  "schemaVariantId": "01H9ZQD35JPMBGHH69BT0Q75XY"
}

```

### [Properties](#unlockfuncv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|schemaVariantId|string|true|none|none|

## [UnlockFuncV1Response](#tocS_UnlockFuncV1Response)

<a id="schemaunlockfuncv1response"></a>
<a id="schema_UnlockFuncV1Response"></a>
<a id="tocSunlockfuncv1response"></a>
<a id="tocsunlockfuncv1response"></a>

```json
{
  "unlockedFuncId": "01H9ZQD35JPMBGHH69BT0Q75XY"
}

```

### [Properties](#unlockfuncv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|unlockedFuncId|string|true|none|none|

## [UnlockedSchemaV1Response](#tocS_UnlockedSchemaV1Response)

<a id="schemaunlockedschemav1response"></a>
<a id="schema_UnlockedSchemaV1Response"></a>
<a id="tocSunlockedschemav1response"></a>
<a id="tocsunlockedschemav1response"></a>

```json
{
  "schemaId": "01H9ZQD35JPMBGHH69BT0Q79VZ",
  "unlockedVariant": {
    "assetFuncId": "01H9ZQD35JPMBGHH69BT0Q75XY",
    "category": "AWS::EC2",
    "color": "#FF5733",
    "description": "Amazon EC2 Instance resource type",
    "displayName": "AWS EC2 Instance",
    "domainProps": {},
    "installedFromUpstream": false,
    "isDefaultVariant": true,
    "isLocked": false,
    "link": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-instance.html",
    "variantFuncIds": [
      "01H9ZQD35JPMBGHH69BT0Q75AA",
      "01H9ZQD35JPMBGHH69BT0Q75BB"
    ],
    "variantFuncs": [
      {
        "funcKind": {
          "actionKind": "Create",
          "kind": "action"
        },
        "id": "01H9ZQD35JPMBGHH69BT0Q79VZ",
        "isOverlay": true
      }
    ],
    "variantId": "01H9ZQD35JPMBGHH69BT0Q79VZ"
  },
  "unlockedVariantId": "01H9ZQD35JPMBGHH69BT0Q75XY"
}

```

### [Properties](#unlockedschemav1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|schemaId|string|true|none|none|
|unlockedVariant|[GetSchemaVariantV1Response](#schemagetschemavariantv1response)|true|none|none|
|unlockedVariantId|string|true|none|none|

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
  "name": "MyUpdatedComponentName",
  "resourceId": "i-12345678",
  "secrets": {
    "secretDefinitionName": "secretName"
  }
}

```

### [Properties](#updatecomponentv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|attributes|object|false|none|none|
|» **additionalProperties**|any|false|none|none|
|name|string,null|false|none|none|
|resourceId|string,null|false|none|none|
|secrets|object|false|none|none|
|» **additionalProperties**|any|false|none|none|

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
        "managing": {
          "componentId": "string",
          "componentName": "string"
        }
      }
    ],
    "domainProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": null
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
        "value": null
      }
    ],
    "schemaId": "string",
    "schemaVariantId": "string",
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

## [UpdateFuncV1Request](#tocS_UpdateFuncV1Request)

<a id="schemaupdatefuncv1request"></a>
<a id="schema_UpdateFuncV1Request"></a>
<a id="tocSupdatefuncv1request"></a>
<a id="tocsupdatefuncv1request"></a>

```json
{
  "code": "<!-- String escaped Typescript code here -->",
  "description": "Updated Description",
  "displayName": "Updated Display Name"
}

```

### [Properties](#updatefuncv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|code|string|true|none|none|
|description|string,null|false|none|none|
|displayName|string,null|false|none|none|

## [UpdateFuncV1Response](#tocS_UpdateFuncV1Response)

<a id="schemaupdatefuncv1response"></a>
<a id="schema_UpdateFuncV1Response"></a>
<a id="tocSupdatefuncv1response"></a>
<a id="tocsupdatefuncv1response"></a>

```json
{
  "success": true
}

```

### [Properties](#updatefuncv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|success|boolean|true|none|none|

## [UpdateMemberRoleRequest](#tocS_UpdateMemberRoleRequest)

<a id="schemaupdatememberrolerequest"></a>
<a id="schema_UpdateMemberRoleRequest"></a>
<a id="tocSupdatememberrolerequest"></a>
<a id="tocsupdatememberrolerequest"></a>

```json
{
  "role": "EDITOR",
  "userId": "01GW0KXH4YJBWC7BTBAZ6ZR7EA"
}

```

### [Properties](#updatememberrolerequest-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|role|string|true|none|none|
|userId|string|true|none|none|

## [UpdateSchemaVariantV1Request](#tocS_UpdateSchemaVariantV1Request)

<a id="schemaupdateschemavariantv1request"></a>
<a id="schema_UpdateSchemaVariantV1Request"></a>
<a id="tocSupdateschemavariantv1request"></a>
<a id="tocsupdateschemavariantv1request"></a>

```json
{
  "category": "AWS::EC2",
  "code": "async function main(input: Input): Promise < Output > {\n    if (!input.domain?.region) {\n        return {\n            result: \"failure\",\n            message: \"No Region Name to validate\",\n        };\n    }\n\n    const child = await siExec.waitUntilEnd(\"aws\", [\n        \"ec2\",\n        \"describe-regions\",\n        \"--region-names\",\n        input.domain?.region!,\n        \"--region\",\n        \"us-east-1\",\n    ]);\n\n    if (child.exitCode !== 0) {\n        console.error(child.stderr);\n        return {\n            result: \"failure\",\n            message: \"Error from API\"\n        }\n    }\n\n    const regionDetails = JSON.parse(child.stdout).Regions;\n    if (regionDetails.length === 0 || regionDetails.length > 1) {\n        return {\n            result: \"failure\",\n            message: \"Unable to find Region\"\n        }\n    }\n\n    if (regionDetails[0].OptInStatus === \"not-opted-in\") {\n        return {\n            result: \"failure\",\n            message: \"Region not-opted-in for use\"\n        }\n    }\n\n    return {\n        result: \"success\",\n        message: \"Region is available to use\",\n    };\n}",
  "color": "#FF5733",
  "description": "Validates if an AWS region exists and is available for use",
  "link": "https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeRegions.html",
  "name": "AWS Region Validator"
}

```

### [Properties](#updateschemavariantv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|category|string|true|none|none|
|code|string|true|none|none|
|color|string,null|false|none|none|
|description|string,null|false|none|none|
|link|string,null|false|none|none|
|name|string|true|none|none|

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
|description|string,null|false|none|none|
|id|string|true|none|none|
|name|string|true|none|none|
|rawData|object,null|false|none|none|
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

## [UpdateWorkspaceRequest](#tocS_UpdateWorkspaceRequest)

<a id="schemaupdateworkspacerequest"></a>
<a id="schema_UpdateWorkspaceRequest"></a>
<a id="tocSupdateworkspacerequest"></a>
<a id="tocsupdateworkspacerequest"></a>

```json
{
  "description": "Updated description for the workspace",
  "displayName": "Updated Workspace Name",
  "instanceUrl": "https://app.systeminit.com"
}

```

### [Properties](#updateworkspacerequest-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|description|string,null|false|none|none|
|displayName|string,null|false|none|none|
|instanceUrl|string,null|false|none|none|

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
        "managing": {
          "componentId": "string",
          "componentName": "string"
        }
      }
    ],
    "domainProps": [
      {
        "id": "string",
        "path": "path/to/prop",
        "propId": "string",
        "value": null
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
        "value": null
      }
    ],
    "schemaId": "string",
    "schemaVariantId": "string",
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

## [Workspace](#tocS_Workspace)

<a id="schemaworkspace"></a>
<a id="schema_Workspace"></a>
<a id="tocSworkspace"></a>
<a id="tocsworkspace"></a>

```json
{
  "approvalsEnabled": true,
  "creatorUser": {},
  "creatorUserId": "string",
  "description": "string",
  "displayName": "string",
  "externalId": "string",
  "id": "string",
  "initialApiToken": {},
  "instanceEnvType": "string",
  "instanceUrl": "string",
  "isDefault": true,
  "quarantinedAt": "string",
  "role": "string"
}

```

### [Properties](#workspace-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|approvalsEnabled|boolean|true|none|none|
|creatorUser|any|false|none|none|

oneOf

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|» *anonymous*|null|false|none|none|

xor

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|» *anonymous*|[CreatorUser](#schemacreatoruser)|false|none|none|

continued

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|creatorUserId|string|true|none|none|
|description|string,null|false|none|none|
|displayName|string|true|none|none|
|externalId|string,null|false|none|none|
|id|string|true|none|none|
|initialApiToken|any|false|none|none|

oneOf

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|» *anonymous*|null|false|none|none|

xor

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|» *anonymous*|[InitialApiToken](#schemainitialapitoken)|false|none|none|

continued

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|instanceEnvType|string|true|none|none|
|instanceUrl|string,null|false|none|none|
|isDefault|boolean|true|none|none|
|quarantinedAt|string,null|false|none|none|
|role|string,null|false|none|none|

## [WorkspaceManagementRequestPath](#tocS_WorkspaceManagementRequestPath)

<a id="schemaworkspacemanagementrequestpath"></a>
<a id="schema_WorkspaceManagementRequestPath"></a>
<a id="tocSworkspacemanagementrequestpath"></a>
<a id="tocsworkspacemanagementrequestpath"></a>

```json
{
  "workspace_id": "string"
}

```

### [Properties](#workspacemanagementrequestpath-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|workspace_id|string|true|none|none|

