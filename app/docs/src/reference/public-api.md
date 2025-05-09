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
  "API Documentation": "Available at /swagger-ui",
  "What is this?": "I am luminork, the new System Initiative External API application"
}
```

<h3 id="system_status_route-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|System status information|[SystemStatusResponse](#schemasystemstatusresponse)|
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
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
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

Change set management endpoints

## list_change_sets

<a id="opIdlist_change_sets"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets`

<h3 id="list_change_sets-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|

> Example responses

> 200 Response

```json
{
  "changeSets": "[{\"id\":\"01H9ZQD35JPMBGHH69BT0Q79VY\",\"name\":\"Add new feature\",\"status\":\"Draft\"}]"
}
```

<h3 id="list_change_sets-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Change sets listed successfully|[ListChangeSetV1Response](#schemalistchangesetv1response)|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## create_change_set

<a id="opIdcreate_change_set"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets`

> Body parameter

```json
{
  "changeSetName": "My new feature"
}
```

<h3 id="create_change_set-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|
|body|body|[CreateChangeSetV1Request](#schemacreatechangesetv1request)|true|none|

> Example responses

> 200 Response

```json
{
  "changeSet": {}
}
```

<h3 id="create_change_set-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Change set created successfully|[CreateChangeSetV1Response](#schemacreatechangesetv1response)|
|422|[Unprocessable Entity](https://tools.ietf.org/html/rfc2518#section-10.3)|Validation error - Invalid request data|[ApiError](#schemaapierror)|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## get_change_set

<a id="opIdget_change_set"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}`

<h3 id="get_change_set-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|
|change_set_id|path|undefined|true|Change set identifier|

> Example responses

> 200 Response

```json
{
  "changeSet": {}
}
```

<h3 id="get_change_set-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Change sets listed successfully|[GetChangeSetV1Response](#schemagetchangesetv1response)|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## abandon_change_set

<a id="opIdabandon_change_set"></a>

> Request format

`DELETE /v1/w/{workspace_id}/change-sets/{change_set_id}`

<h3 id="abandon_change_set-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|
|change_set_id|path|undefined|true|Change set identifier|

> Example responses

> 200 Response

```json
{
  "success": "true"
}
```

<h3 id="abandon_change_set-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Change set deleted successfully|[DeleteChangeSetV1Response](#schemadeletechangesetv1response)|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## force_apply

<a id="opIdforce_apply"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/force_apply`

<h3 id="force_apply-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|
|change_set_id|path|undefined|true|Change set identifier|

> Example responses

> 500 Response

```json
{
  "code": 0,
  "message": "string",
  "statusCode": 0
}
```

<h3 id="force_apply-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Change set force applied successfully|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## merge_status

<a id="opIdmerge_status"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/merge_status`

<h3 id="merge_status-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|
|change_set_id|path|undefined|true|Change set identifier|

> Example responses

> 200 Response

```json
{
  "actions": [
    {
      "component": {},
      "id": "string",
      "kind": "string",
      "name": "string",
      "state": "string"
    }
  ],
  "changeSet": {}
}
```

<h3 id="merge_status-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Change set merge status retrieved successfully|[MergeStatusV1Response](#schemamergestatusv1response)|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## request_approval

<a id="opIdrequest_approval"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/request_approval`

<h3 id="request_approval-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|
|change_set_id|path|undefined|true|Change set identifier|

> Example responses

> 500 Response

```json
{
  "code": 0,
  "message": "string",
  "statusCode": 0
}
```

<h3 id="request_approval-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Change set approval requested successfully|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

# [components](#system-initiative-api-components)

Component management endpoints

## list_components

<a id="opIdlist_components"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/components`

<h3 id="list_components-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|
|change_set_id|path|undefined|true|Change set identifier|

> Example responses

> 500 Response

```json
{
  "code": 0,
  "message": "string",
  "statusCode": 0
}
```

<h3 id="list_components-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Components retrieved successfully|None|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## create_component

<a id="opIdcreate_component"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/components`

> Body parameter

```json
{
  "connections": [
    {
      "from": {
        "component": "OtherComponentName",
        "socketName": "SocketName"
      },
      "to": "ThisComponentInputSocketName"
    },
    {
      "from": {
        "componentId": "01H9ZQD35JPMBGHH69BT0Q79VY",
        "socketName": "SocketName"
      },
      "to": "ThisComponentInputSocketName"
    },
    {
      "from": "ThisComponentOutputSocketName",
      "to": {
        "component": "OtherComponentName",
        "socketName": "InputSocketName"
      }
    },
    {
      "from": "ThisComponentOutputSocketName",
      "to": {
        "componentId": "01H9ZQD35JPMBGHH69BT0Q79VY",
        "socketName": "InputSocketName"
      }
    }
  ],
  "domain": {
    "propId1": "value1",
    "path/to/prop": "value2"
  },
  "name": "MyComponentName",
  "schemaName": "AWS::EC2::Instance",
  "secrets": {
    "secretDefinitionName": "secretName"
  },
  "viewName": "MyView"
}
```

<h3 id="create_component-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|
|change_set_id|path|undefined|true|Change set identifier|
|body|body|[CreateComponentV1Request](#schemacreatecomponentv1request)|true|none|

> Example responses

> 200 Response

```json
{
  "component": {
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

<h3 id="create_component-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Component created successfully|[CreateComponentV1Response](#schemacreatecomponentv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Component not found|None|
|412|[Precondition Failed](https://tools.ietf.org/html/rfc7232#section-4.2)|Precondition Failed - View not found|[ApiError](#schemaapierror)|
|422|[Unprocessable Entity](https://tools.ietf.org/html/rfc2518#section-10.3)|Validation error - Invalid request data|[ApiError](#schemaapierror)|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## find_component

<a id="opIdfind_component"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/components/find`

<h3 id="find_component-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|
|change_set_id|path|undefined|true|Change set identifier|
|component|query|string,null|false|none|
|componentId|query|string,null|false|none|

> Example responses

> 200 Response

```json
{
  "actionFunctions": [
    {
      "funcName": "string",
      "prototypeId": "string"
    }
  ],
  "component": {
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
      "funcName": "string",
      "managementPrototypeId": "string"
    }
  ]
}
```

<h3 id="find_component-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Component retrieved successfully|[GetComponentV1Response](#schemagetcomponentv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Component not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## get_component

<a id="opIdget_component"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}`

<h3 id="get_component-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|
|change_set_id|path|undefined|true|Change set identifier|
|component_id|path|undefined|true|Component identifier|

> Example responses

> 200 Response

```json
{
  "actionFunctions": [
    {
      "funcName": "string",
      "prototypeId": "string"
    }
  ],
  "component": {
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
      "funcName": "string",
      "managementPrototypeId": "string"
    }
  ]
}
```

<h3 id="get_component-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Component retrieved successfully|[GetComponentV1Response](#schemagetcomponentv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Component not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## update_component

<a id="opIdupdate_component"></a>

> Request format

`PUT /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}`

> Body parameter

```json
{
  "connectionChanges": {
    "add": [
      {
        "from": {
          "component": "OtherComponentName",
          "socketName": "output"
        },
        "to": "ThisComponentInputSocketName"
      },
      {
        "from": "ThisComponentOutputSocketName",
        "to": {
          "componentId": "01H9ZQD35JPMBGHH69BT0Q79VY",
          "socketName": "InputSocketName"
        }
      }
    ],
    "remove": [
      {
        "from": {
          "componentId": "01H9ZQD35JPMBGHH69BT0Q79VY",
          "socketName": "output"
        },
        "to": "ThisComponentInputSocketName"
      },
      {
        "from": "ThisComponentOutputSocketName",
        "to": {
          "component": "OtherComponentName",
          "socketName": "InputSocketName"
        }
      }
    ]
  },
  "domain": {
    "propId1": "value1",
    "path/to/prop": "value2"
  },
  "name": "MyUpdatedComponentName",
  "secrets": {
    "secretDefinitionName": "secretName"
  },
  "unset": [
    "propId1",
    "path/to/prop"
  ]
}
```

<h3 id="update_component-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|
|change_set_id|path|undefined|true|Change set identifier|
|component_id|path|undefined|true|Component identifier|
|body|body|[UpdateComponentV1Request](#schemaupdatecomponentv1request)|true|none|

> Example responses

> 200 Response

```json
{
  "component": {
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

<h3 id="update_component-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Component updated successfully|[UpdateComponentV1Response](#schemaupdatecomponentv1response)|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Component not found|None|
|412|[Precondition Failed](https://tools.ietf.org/html/rfc7232#section-4.2)|Precondition failed - Duplicate component name|None|
|422|[Unprocessable Entity](https://tools.ietf.org/html/rfc2518#section-10.3)|Validation error - Invalid request data|[ApiError](#schemaapierror)|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## delete_component

<a id="opIddelete_component"></a>

> Request format

`DELETE /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}`

<h3 id="delete_component-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|
|change_set_id|path|undefined|true|Change set identifier|
|component_id|path|undefined|true|Component identifier|

> Example responses

> 200 Response

```json
{
  "status": "MarkedForDeletion"
}
```

<h3 id="delete_component-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Component deleted successfully|[DeleteComponentV1Response](#schemadeletecomponentv1response)|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Component not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## add_action

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

<h3 id="add_action-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|
|change_set_id|path|undefined|true|Change set identifier|
|component_id|path|undefined|true|Component identifier|
|body|body|[AddActionV1Request](#schemaaddactionv1request)|true|none|

> Example responses

> 200 Response

```json
{}
```

<h3 id="add_action-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Action successfully queued|[AddActionV1Response](#schemaaddactionv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Component or function not found|None|
|409|[Conflict](https://tools.ietf.org/html/rfc7231#section-6.5.8)|action already enqueued|[ApiError](#schemaapierror)|
|412|[Precondition Failed](https://tools.ietf.org/html/rfc7232#section-4.2)|Precondition Failed - View not found or duplicate function name|[ApiError](#schemaapierror)|
|422|[Unprocessable Entity](https://tools.ietf.org/html/rfc2518#section-10.3)|Validation error - Invalid request data|[ApiError](#schemaapierror)|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## execute_management_function

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

<h3 id="execute_management_function-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|
|change_set_id|path|undefined|true|Change set identifier|
|component_id|path|undefined|true|Component identifier|
|body|body|[ExecuteManagementFunctionV1Request](#schemaexecutemanagementfunctionv1request)|true|none|

> Example responses

> 200 Response

```json
{
  "funcRunId": "01H9ZQD35JPMBGHH69BT0Q79VY"
}
```

<h3 id="execute_management_function-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Function successfully dispatched|[ExecuteManagementFunctionV1Response](#schemaexecutemanagementfunctionv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Component or function not found|None|
|412|[Precondition Failed](https://tools.ietf.org/html/rfc7232#section-4.2)|Precondition Failed - View not found or duplicate function name|[ApiError](#schemaapierror)|
|422|[Unprocessable Entity](https://tools.ietf.org/html/rfc2518#section-10.3)|Validation error - Invalid request data|[ApiError](#schemaapierror)|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

# [funcs](#system-initiative-api-funcs)

Functions management endpoints

## get_func_run

<a id="opIdget_func_run"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/funcs/runs/{func_run_id}`

<h3 id="get_func_run-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|
|change_set_id|path|undefined|true|Change set identifier|
|func_run_id|path|undefined|true|Func run identifier|

> Example responses

> 200 Response

```json
{
  "funcRun": {
    "actionDisplayName": "string",
    "actionId": "string",
    "actionKind": "string",
    "actionOriginatingChangeSetId": "string",
    "actionOriginatingChangeSetName": "string",
    "actionPrototypeId": "string",
    "actionResultState": "string",
    "attributeValueId": "string",
    "backendKind": "string",
    "backendResponseType": "string",
    "componentId": "string",
    "componentName": "string",
    "createdAt": "string",
    "functionArgs": {},
    "functionCodeBase64": {},
    "functionDescription": "string",
    "functionDisplayName": "string",
    "functionKind": "string",
    "functionLink": "string",
    "functionName": "string",
    "id": "string",
    "logs": {},
    "resultValue": {},
    "schemaName": "string",
    "state": "string",
    "updatedAt": "string"
  }
}
```

<h3 id="get_func_run-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Func Run retrieved successfully|[GetFuncRunV1Response](#schemagetfuncrunv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Func run not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## get_func

<a id="opIdget_func"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/funcs/{func_id}`

<h3 id="get_func-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|
|change_set_id|path|undefined|true|Change set identifier|
|func_id|path|undefined|true|Func identifier|

> Example responses

> 200 Response

```json
{
  "code": "string",
  "description": "string",
  "displayName": "string",
  "isLocked": true,
  "kind": "string",
  "link": "string",
  "name": "string"
}
```

<h3 id="get_func-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Func retrieved successfully|[GetFuncV1Response](#schemagetfuncv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Func not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

# [actions](#system-initiative-api-actions)

## get_actions

<a id="opIdget_actions"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/actions/`

<h3 id="get_actions-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|
|change_set_id|path|undefined|true|Change set identifier|

> Example responses

> 200 Response

```json
{
  "actions": [
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
  ]
}
```

<h3 id="get_actions-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Actions retrieved successfully|[GetActionsV1Response](#schemagetactionsv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## cancel_action

<a id="opIdcancel_action"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/actions/{action_id}/cancel`

<h3 id="cancel_action-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|
|change_set_id|path|undefined|true|Change set identifier|
|action_id|path|undefined|true|Func identifier|

> Example responses

> 200 Response

```json
{
  "success": true
}
```

<h3 id="cancel_action-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Action cancelled successfully|[CancelActionV1Response](#schemacancelactionv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Action not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## put_on_hold

<a id="opIdput_on_hold"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/actions/{action_id}/put_on_hold`

<h3 id="put_on_hold-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|
|change_set_id|path|undefined|true|Change set identifier|
|action_id|path|undefined|true|Func identifier|

> Example responses

> 200 Response

```json
{
  "success": true
}
```

<h3 id="put_on_hold-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Action successfully put on hold|[PutOnHoldActionV1Response](#schemaputonholdactionv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Action not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## retry_action

<a id="opIdretry_action"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/actions/{action_id}/retry`

<h3 id="retry_action-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|
|change_set_id|path|undefined|true|Change set identifier|
|action_id|path|undefined|true|Func identifier|

> Example responses

> 200 Response

```json
{
  "success": true
}
```

<h3 id="retry_action-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Action successfully requeued|[RetryActionV1Response](#schemaretryactionv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Action not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

# [schemas](#system-initiative-api-schemas)

## list_schemas

<a id="opIdlist_schemas"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/schema`

<h3 id="list_schemas-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|
|change_set_id|path|undefined|true|Change set identifier|

> Example responses

> 200 Response

```json
{
  "schemas": "[{\"schemaId\":\"01H9ZQD35JPMBGHH69BT0Q79VY\",\"schemaName\":\"AWS::EC2::Instance\",\"category\":\"AWS::EC2\",\"installed\": \"true\"}]"
}
```

<h3 id="list_schemas-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Schemas listed successfully|[ListSchemaV1Response](#schemalistschemav1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## find_schema

<a id="opIdfind_schema"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/schema/find`

<h3 id="find_schema-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|
|change_set_id|path|undefined|true|Change set identifier|
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

<h3 id="find_schema-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Schema retrieved successfully|[FindSchemaV1Response](#schemafindschemav1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Schema not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## get_schema

<a id="opIdget_schema"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/schema/{schema_id}`

<h3 id="get_schema-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|
|change_set_id|path|undefined|true|Change set identifier|
|schema_id|path|undefined|true|Schema identifier|

> Example responses

> 200 Response

```json
{
  "defaultVariantId": "string",
  "name": "string",
  "variantIds": [
    "string"
  ]
}
```

<h3 id="get_schema-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Schema retrieved successfully|[GetSchemaV1Response](#schemagetschemav1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Schema not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## get_default_variant

<a id="opIdget_default_variant"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/schema/{schema_id}/variant/default`

<h3 id="get_default_variant-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|
|change_set_id|path|undefined|true|Change set identifier|
|schema_id|path|undefined|true|Schema identifier|

> Example responses

> 200 Response

```json
{
  "assetFuncId": "string",
  "category": "string",
  "color": "string",
  "description": "string",
  "displayName": "string",
  "isDefaultVariant": true,
  "isLocked": true,
  "link": "string",
  "variantFuncIds": [
    "string"
  ],
  "variantId": "string"
}
```

<h3 id="get_default_variant-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Schema variant retrieved successfully|[GetSchemaVariantV1Response](#schemagetschemavariantv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Schema variant not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## get_variant

<a id="opIdget_variant"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/schema/{schema_id}/variant/{schema_variant_id}`

<h3 id="get_variant-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|
|change_set_id|path|undefined|true|Change set identifier|
|schema_id|path|undefined|true|Schema identifier|
|schema_variant_id|path|undefined|true|Schema variant identifier|

> Example responses

> 200 Response

```json
{
  "assetFuncId": "string",
  "category": "string",
  "color": "string",
  "description": "string",
  "displayName": "string",
  "isDefaultVariant": true,
  "isLocked": true,
  "link": "string",
  "variantFuncIds": [
    "string"
  ],
  "variantId": "string"
}
```

<h3 id="get_variant-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Schema variant retrieved successfully|[GetSchemaVariantV1Response](#schemagetschemavariantv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Schema variant not found|None|
|412|[Precondition Failed](https://tools.ietf.org/html/rfc7232#section-4.2)|Schema variant not found for schema|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

# [secrets](#system-initiative-api-secrets)

## get_secrets

<a id="opIdget_secrets"></a>

> Request format

`GET /v1/w/{workspace_id}/change-sets/{change_set_id}/secrets`

<h3 id="get_secrets-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|
|change_set_id|path|undefined|true|Change set identifier|

> Example responses

> 500 Response

```json
{
  "code": 0,
  "message": "string",
  "statusCode": 0
}
```

<h3 id="get_secrets-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Secrets retrieved successfully|None|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## create_secret

<a id="opIdcreate_secret"></a>

> Request format

`POST /v1/w/{workspace_id}/change-sets/{change_set_id}/secrets`

> Body parameter

```json
{
  "definitionName": "string",
  "description": "string",
  "name": "string",
  "rawData": {}
}
```

<h3 id="create_secret-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|
|change_set_id|path|undefined|true|Change set identifier|
|body|body|[CreateSecretV1Request](#schemacreatesecretv1request)|true|none|

> Example responses

> 200 Response

```json
{
  "secret": {
    "definition": "string",
    "description": "string",
    "id": "string",
    "name": "string"
  }
}
```

<h3 id="create_secret-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Secret created successfully|[CreateSecretV1Response](#schemacreatesecretv1response)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|Unauthorized - Invalid or missing token|None|
|422|[Unprocessable Entity](https://tools.ietf.org/html/rfc2518#section-10.3)|Validation error - Invalid request data|[ApiError](#schemaapierror)|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## update_secret

<a id="opIdupdate_secret"></a>

> Request format

`PUT /v1/w/{workspace_id}/change-sets/{change_set_id}/secrets/{secret_id}`

> Body parameter

```json
{
  "description": "string",
  "id": "string",
  "name": "string",
  "rawData": {}
}
```

<h3 id="update_secret-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|
|change_set_id|path|undefined|true|Change set identifier|
|secret_id|path|undefined|true|Secret identifier|
|body|body|[UpdateSecretV1Request](#schemaupdatesecretv1request)|true|none|

> Example responses

> 200 Response

```json
{
  "secret": {
    "definition": "string",
    "description": "string",
    "id": "string",
    "name": "string"
  }
}
```

<h3 id="update_secret-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Secret updated successfully|[UpdateSecretV1Response](#schemaupdatesecretv1response)|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Secret not found|None|
|500|[Internal Server Error](https://tools.ietf.org/html/rfc7231#section-6.6.1)|Internal server error|[ApiError](#schemaapierror)|

## delete_secret

<a id="opIddelete_secret"></a>

> Request format

`DELETE /v1/w/{workspace_id}/change-sets/{change_set_id}/secrets/{secret_id}`

<h3 id="delete_secret-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|workspace_id|path|undefined|true|Workspace identifier|
|change_set_id|path|undefined|true|Change set identifier|
|secret_id|path|undefined|true|Secret identifier|

> Example responses

> 200 Response

```json
{
  "success": true
}
```

<h3 id="delete_secret-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Secret deleted successfully|[DeleteSecretV1Response](#schemadeletesecretv1response)|
|404|[Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4)|Secret not found|None|
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
{}

```

### [Properties](#addactionv1response-properties)

*None*

## [ApiError](#tocS_ApiError)

<a id="schemaapierror"></a>
<a id="schema_ApiError"></a>
<a id="tocSapierror"></a>
<a id="tocsapierror"></a>

```json
{
  "code": 0,
  "message": "string",
  "statusCode": 0
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
  "componentId": "01H9ZQD35JPMBGHH69BT0Q79VY"
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
  "add": [
    {
      "from": {
        "component": "OtherComponentName",
        "socketName": "output"
      },
      "to": "ThisComponentInputSocketName"
    },
    {
      "from": "ThisComponentOutputSocketName",
      "to": {
        "componentId": "01H9ZQD35JPMBGHH69BT0Q79VY",
        "socketName": "InputSocketName"
      }
    }
  ],
  "remove": [
    {
      "from": {
        "componentId": "01H9ZQD35JPMBGHH69BT0Q79VY",
        "socketName": "output"
      },
      "to": "ThisComponentInputSocketName"
    },
    {
      "from": "ThisComponentOutputSocketName",
      "to": {
        "component": "OtherComponentName",
        "socketName": "InputSocketName"
      }
    }
  ]
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
  "changeSet": {}
}

```

### [Properties](#createchangesetv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|changeSet|object|true|none|none|

## [CreateComponentV1Request](#tocS_CreateComponentV1Request)

<a id="schemacreatecomponentv1request"></a>
<a id="schema_CreateComponentV1Request"></a>
<a id="tocScreatecomponentv1request"></a>
<a id="tocscreatecomponentv1request"></a>

```json
{
  "connections": [
    {
      "from": {
        "component": "OtherComponentName",
        "socketName": "SocketName"
      },
      "to": "ThisComponentInputSocketName"
    },
    {
      "from": {
        "componentId": "01H9ZQD35JPMBGHH69BT0Q79VY",
        "socketName": "SocketName"
      },
      "to": "ThisComponentInputSocketName"
    },
    {
      "from": "ThisComponentOutputSocketName",
      "to": {
        "component": "OtherComponentName",
        "socketName": "InputSocketName"
      }
    },
    {
      "from": "ThisComponentOutputSocketName",
      "to": {
        "componentId": "01H9ZQD35JPMBGHH69BT0Q79VY",
        "socketName": "InputSocketName"
      }
    }
  ],
  "domain": {
    "propId1": "value1",
    "path/to/prop": "value2"
  },
  "name": "MyComponentName",
  "schemaName": "AWS::EC2::Instance",
  "secrets": {
    "secretDefinitionName": "secretName"
  },
  "viewName": "MyView"
}

```

### [Properties](#createcomponentv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|connections|[[Connection](#schemaconnection)]|false|none|none|
|domain|object|false|none|none|
|» **additionalProperties**|any|false|none|none|
|name|string|true|none|none|
|schemaName|string|true|none|none|
|secrets|object|false|none|none|
|» **additionalProperties**|any|false|none|none|
|viewName|string,null|false|none|none|

## [CreateComponentV1Response](#tocS_CreateComponentV1Response)

<a id="schemacreatecomponentv1response"></a>
<a id="schema_CreateComponentV1Response"></a>
<a id="tocScreatecomponentv1response"></a>
<a id="tocscreatecomponentv1response"></a>

```json
{
  "component": {
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
  "definitionName": "string",
  "description": "string",
  "name": "string",
  "rawData": {}
}

```

### [Properties](#createsecretv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|definitionName|string|true|none|none|
|description|string|true|none|none|
|name|string|true|none|none|
|rawData|object|true|none|none|

## [CreateSecretV1Response](#tocS_CreateSecretV1Response)

<a id="schemacreatesecretv1response"></a>
<a id="schema_CreateSecretV1Response"></a>
<a id="tocScreatesecretv1response"></a>
<a id="tocscreatesecretv1response"></a>

```json
{
  "secret": {
    "definition": "string",
    "description": "string",
    "id": "string",
    "name": "string"
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
  "funcRunId": "01H9ZQD35JPMBGHH69BT0Q79VY"
}

```

### [Properties](#executemanagementfunctionv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|funcRunId|string|true|none|none|

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

## [FuncRunLogViewV1](#tocS_FuncRunLogViewV1)

<a id="schemafuncrunlogviewv1"></a>
<a id="schema_FuncRunLogViewV1"></a>
<a id="tocSfuncrunlogviewv1"></a>
<a id="tocsfuncrunlogviewv1"></a>

```json
{
  "createdAt": "string",
  "finalized": true,
  "funcRunId": "string",
  "id": "string",
  "logs": [
    {}
  ],
  "updatedAt": "string"
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
  "actionDisplayName": "string",
  "actionId": "string",
  "actionKind": "string",
  "actionOriginatingChangeSetId": "string",
  "actionOriginatingChangeSetName": "string",
  "actionPrototypeId": "string",
  "actionResultState": "string",
  "attributeValueId": "string",
  "backendKind": "string",
  "backendResponseType": "string",
  "componentId": "string",
  "componentName": "string",
  "createdAt": "string",
  "functionArgs": {},
  "functionCodeBase64": {},
  "functionDescription": "string",
  "functionDisplayName": "string",
  "functionKind": "string",
  "functionLink": "string",
  "functionName": "string",
  "id": "string",
  "logs": {},
  "resultValue": {},
  "schemaName": "string",
  "state": "string",
  "updatedAt": "string"
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
|functionArgs|object|true|none|none|
|functionCodeBase64|object|true|none|none|
|functionDescription|string|true|none|none|
|functionDisplayName|string|true|none|none|
|functionKind|string|true|none|none|
|functionLink|string|true|none|none|
|functionName|string|true|none|none|
|id|string|true|none|none|
|logs|object|true|none|none|
|resultValue|object|true|none|none|
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
  "changeSet": {}
}

```

### [Properties](#getchangesetv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|changeSet|object|true|none|none|

## [GetComponentV1Response](#tocS_GetComponentV1Response)

<a id="schemagetcomponentv1response"></a>
<a id="schema_GetComponentV1Response"></a>
<a id="tocSgetcomponentv1response"></a>
<a id="tocsgetcomponentv1response"></a>

```json
{
  "actionFunctions": [
    {
      "funcName": "string",
      "prototypeId": "string"
    }
  ],
  "component": {
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
      "funcName": "string",
      "managementPrototypeId": "string"
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
  "funcName": "string",
  "prototypeId": "string"
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
  "funcName": "string",
  "managementPrototypeId": "string"
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
    "actionDisplayName": "string",
    "actionId": "string",
    "actionKind": "string",
    "actionOriginatingChangeSetId": "string",
    "actionOriginatingChangeSetName": "string",
    "actionPrototypeId": "string",
    "actionResultState": "string",
    "attributeValueId": "string",
    "backendKind": "string",
    "backendResponseType": "string",
    "componentId": "string",
    "componentName": "string",
    "createdAt": "string",
    "functionArgs": {},
    "functionCodeBase64": {},
    "functionDescription": "string",
    "functionDisplayName": "string",
    "functionKind": "string",
    "functionLink": "string",
    "functionName": "string",
    "id": "string",
    "logs": {},
    "resultValue": {},
    "schemaName": "string",
    "state": "string",
    "updatedAt": "string"
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
  "code": "string",
  "description": "string",
  "displayName": "string",
  "isLocked": true,
  "kind": "string",
  "link": "string",
  "name": "string"
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

## [GetSchemaV1Response](#tocS_GetSchemaV1Response)

<a id="schemagetschemav1response"></a>
<a id="schema_GetSchemaV1Response"></a>
<a id="tocSgetschemav1response"></a>
<a id="tocsgetschemav1response"></a>

```json
{
  "defaultVariantId": "string",
  "name": "string",
  "variantIds": [
    "string"
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
  "assetFuncId": "string",
  "category": "string",
  "color": "string",
  "description": "string",
  "displayName": "string",
  "isDefaultVariant": true,
  "isLocked": true,
  "link": "string",
  "variantFuncIds": [
    "string"
  ],
  "variantId": "string"
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
|isDefaultVariant|boolean|true|none|none|
|isLocked|boolean|true|none|none|
|link|string|true|none|none|
|variantFuncIds|[string]|true|none|none|
|variantId|string|true|none|none|

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
  "changeSets": "[{\"id\":\"01H9ZQD35JPMBGHH69BT0Q79VY\",\"name\":\"Add new feature\",\"status\":\"Draft\"}]"
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
  "components": "string"
}

```

### [Properties](#listcomponentsv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|components|string|true|none|none|

## [ListSchemaV1Response](#tocS_ListSchemaV1Response)

<a id="schemalistschemav1response"></a>
<a id="schema_ListSchemaV1Response"></a>
<a id="tocSlistschemav1response"></a>
<a id="tocslistschemav1response"></a>

```json
{
  "schemas": "[{\"schemaId\":\"01H9ZQD35JPMBGHH69BT0Q79VY\",\"schemaName\":\"AWS::EC2::Instance\",\"category\":\"AWS::EC2\",\"installed\": \"true\"}]"
}

```

### [Properties](#listschemav1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|schemas|[object]|true|none|none|

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
  "actions": [
    {
      "component": {},
      "id": "string",
      "kind": "string",
      "name": "string",
      "state": "string"
    }
  ],
  "changeSet": {}
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
  "component": {},
  "id": "string",
  "kind": "string",
  "name": "string",
  "state": "string"
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
  "id": "string",
  "name": "string"
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
  "executionId": "string",
  "group": "string",
  "level": "string",
  "message": "string",
  "stream": "string",
  "timestamp": 0
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
  "definition": "string",
  "description": "string",
  "id": "string",
  "name": "string"
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

## [SystemStatusResponse](#tocS_SystemStatusResponse)

<a id="schemasystemstatusresponse"></a>
<a id="schema_SystemStatusResponse"></a>
<a id="tocSsystemstatusresponse"></a>
<a id="tocssystemstatusresponse"></a>

```json
{
  "API Documentation": "Available at /swagger-ui",
  "What is this?": "I am luminork, the new System Initiative External API application"
}

```

### [Properties](#systemstatusresponse-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|API Documentation|string|true|none|none|
|What is this?|string|true|none|none|

## [UpdateComponentV1Request](#tocS_UpdateComponentV1Request)

<a id="schemaupdatecomponentv1request"></a>
<a id="schema_UpdateComponentV1Request"></a>
<a id="tocSupdatecomponentv1request"></a>
<a id="tocsupdatecomponentv1request"></a>

```json
{
  "connectionChanges": {
    "add": [
      {
        "from": {
          "component": "OtherComponentName",
          "socketName": "output"
        },
        "to": "ThisComponentInputSocketName"
      },
      {
        "from": "ThisComponentOutputSocketName",
        "to": {
          "componentId": "01H9ZQD35JPMBGHH69BT0Q79VY",
          "socketName": "InputSocketName"
        }
      }
    ],
    "remove": [
      {
        "from": {
          "componentId": "01H9ZQD35JPMBGHH69BT0Q79VY",
          "socketName": "output"
        },
        "to": "ThisComponentInputSocketName"
      },
      {
        "from": "ThisComponentOutputSocketName",
        "to": {
          "component": "OtherComponentName",
          "socketName": "InputSocketName"
        }
      }
    ]
  },
  "domain": {
    "propId1": "value1",
    "path/to/prop": "value2"
  },
  "name": "MyUpdatedComponentName",
  "secrets": {
    "secretDefinitionName": "secretName"
  },
  "unset": [
    "propId1",
    "path/to/prop"
  ]
}

```

### [Properties](#updatecomponentv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|connectionChanges|[ConnectionDetails](#schemaconnectiondetails)|false|none|none|
|domain|object|false|none|none|
|» **additionalProperties**|any|false|none|none|
|name|string,null|false|none|none|
|secrets|object|false|none|none|
|» **additionalProperties**|any|false|none|none|
|unset|[string]|false|none|none|

## [UpdateComponentV1Response](#tocS_UpdateComponentV1Response)

<a id="schemaupdatecomponentv1response"></a>
<a id="schema_UpdateComponentV1Response"></a>
<a id="tocSupdatecomponentv1response"></a>
<a id="tocsupdatecomponentv1response"></a>

```json
{
  "component": {
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
  "description": "string",
  "id": "string",
  "name": "string",
  "rawData": {}
}

```

### [Properties](#updatesecretv1request-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|description|string|true|none|none|
|id|string|true|none|none|
|name|string|true|none|none|
|rawData|object|true|none|none|

## [UpdateSecretV1Response](#tocS_UpdateSecretV1Response)

<a id="schemaupdatesecretv1response"></a>
<a id="schema_UpdateSecretV1Response"></a>
<a id="tocSupdatesecretv1response"></a>
<a id="tocsupdatesecretv1response"></a>

```json
{
  "secret": {
    "definition": "string",
    "description": "string",
    "id": "string",
    "name": "string"
  }
}

```

### [Properties](#updatesecretv1response-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|secret|[SecretV1](#schemasecretv1)|true|none|none|

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
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "userEmail": "user@example.com",
  "userId": "01H9ZQCBJ3E7HBTRN3J58JQX8K",
  "workspaceId": "01H9ZQD35JPMBGHH69BT0Q79VY"
}

```

### [Properties](#whoamiresponse-properties)

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|token|string|true|none|none|
|userEmail|string|true|none|none|
|userId|string|true|none|none|
|workspaceId|string|true|none|none|

