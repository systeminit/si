## system-initiative-api-client@1.0.0

This generator creates TypeScript/JavaScript client that utilizes [axios](https://github.com/axios/axios). The generated Node module can be used in the following environments:

Environment
* Node.js
* Webpack
* Browserify

Language level
* ES5 - you must have a Promises/A+ library installed
* ES6

Module system
* CommonJS
* ES6 module system

It can be used in both TypeScript and JavaScript. In TypeScript, the definition will be automatically resolved via `package.json`. ([Reference](https://www.typescriptlang.org/docs/handbook/declaration-files/consumption.html))

### Building

To build and compile the typescript sources to javascript use:
```
npm install
npm run build
```

### Publishing

First build the package then run `npm publish`

### Consuming

navigate to the folder of your consuming project and run one of the following commands.

_published:_

```
npm install system-initiative-api-client@1.0.0 --save
```

_unPublished (not recommended):_

```
npm install PATH_TO_GENERATED_PACKAGE --save
```

### Documentation for API Endpoints

All URIs are relative to *http://localhost*

Class | Method | HTTP request | Description
------------ | ------------- | ------------- | -------------
*ActionsApi* | [**cancelAction**](docs/ActionsApi.md#cancelaction) | **POST** /v1/w/{workspace_id}/change-sets/{change_set_id}/actions/{action_id}/cancel | 
*ActionsApi* | [**getActions**](docs/ActionsApi.md#getactions) | **GET** /v1/w/{workspace_id}/change-sets/{change_set_id}/actions/ | 
*ActionsApi* | [**putOnHold**](docs/ActionsApi.md#putonhold) | **POST** /v1/w/{workspace_id}/change-sets/{change_set_id}/actions/{action_id}/put_on_hold | 
*ActionsApi* | [**retryAction**](docs/ActionsApi.md#retryaction) | **POST** /v1/w/{workspace_id}/change-sets/{change_set_id}/actions/{action_id}/retry | 
*ChangeSetsApi* | [**abandonChangeSet**](docs/ChangeSetsApi.md#abandonchangeset) | **DELETE** /v1/w/{workspace_id}/change-sets/{change_set_id} | 
*ChangeSetsApi* | [**createChangeSet**](docs/ChangeSetsApi.md#createchangeset) | **POST** /v1/w/{workspace_id}/change-sets | 
*ChangeSetsApi* | [**forceApply**](docs/ChangeSetsApi.md#forceapply) | **POST** /v1/w/{workspace_id}/change-sets/{change_set_id}/force_apply | 
*ChangeSetsApi* | [**getChangeSet**](docs/ChangeSetsApi.md#getchangeset) | **GET** /v1/w/{workspace_id}/change-sets/{change_set_id} | 
*ChangeSetsApi* | [**listChangeSets**](docs/ChangeSetsApi.md#listchangesets) | **GET** /v1/w/{workspace_id}/change-sets | 
*ChangeSetsApi* | [**mergeStatus**](docs/ChangeSetsApi.md#mergestatus) | **GET** /v1/w/{workspace_id}/change-sets/{change_set_id}/merge_status | 
*ChangeSetsApi* | [**purgeOpen**](docs/ChangeSetsApi.md#purgeopen) | **POST** /v1/w/{workspace_id}/change-sets/purge_open | 
*ChangeSetsApi* | [**requestApproval**](docs/ChangeSetsApi.md#requestapproval) | **POST** /v1/w/{workspace_id}/change-sets/{change_set_id}/request_approval | 
*ComponentsApi* | [**addAction**](docs/ComponentsApi.md#addaction) | **POST** /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}/action | 
*ComponentsApi* | [**createComponent**](docs/ComponentsApi.md#createcomponent) | **POST** /v1/w/{workspace_id}/change-sets/{change_set_id}/components | 
*ComponentsApi* | [**deleteComponent**](docs/ComponentsApi.md#deletecomponent) | **DELETE** /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id} | 
*ComponentsApi* | [**executeManagementFunction**](docs/ComponentsApi.md#executemanagementfunction) | **POST** /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}/execute-management-function | 
*ComponentsApi* | [**findComponent**](docs/ComponentsApi.md#findcomponent) | **GET** /v1/w/{workspace_id}/change-sets/{change_set_id}/components/find | 
*ComponentsApi* | [**getComponent**](docs/ComponentsApi.md#getcomponent) | **GET** /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id} | 
*ComponentsApi* | [**listComponents**](docs/ComponentsApi.md#listcomponents) | **GET** /v1/w/{workspace_id}/change-sets/{change_set_id}/components | 
*ComponentsApi* | [**updateComponent**](docs/ComponentsApi.md#updatecomponent) | **PUT** /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id} | 
*FuncsApi* | [**getFunc**](docs/FuncsApi.md#getfunc) | **GET** /v1/w/{workspace_id}/change-sets/{change_set_id}/funcs/{func_id} | 
*FuncsApi* | [**getFuncRun**](docs/FuncsApi.md#getfuncrun) | **GET** /v1/w/{workspace_id}/change-sets/{change_set_id}/funcs/runs/{func_run_id} | 
*RootApi* | [**systemStatusRoute**](docs/RootApi.md#systemstatusroute) | **GET** / | 
*SchemasApi* | [**findSchema**](docs/SchemasApi.md#findschema) | **GET** /v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/find | 
*SchemasApi* | [**getDefaultVariant**](docs/SchemasApi.md#getdefaultvariant) | **GET** /v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/variant/default | 
*SchemasApi* | [**getSchema**](docs/SchemasApi.md#getschema) | **GET** /v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id} | 
*SchemasApi* | [**getVariant**](docs/SchemasApi.md#getvariant) | **GET** /v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/variant/{schema_variant_id} | 
*SchemasApi* | [**listSchemas**](docs/SchemasApi.md#listschemas) | **GET** /v1/w/{workspace_id}/change-sets/{change_set_id}/schemas | 
*SecretsApi* | [**createSecret**](docs/SecretsApi.md#createsecret) | **POST** /v1/w/{workspace_id}/change-sets/{change_set_id}/secrets | 
*SecretsApi* | [**deleteSecret**](docs/SecretsApi.md#deletesecret) | **DELETE** /v1/w/{workspace_id}/change-sets/{change_set_id}/secrets/{secret_id} | 
*SecretsApi* | [**getSecrets**](docs/SecretsApi.md#getsecrets) | **GET** /v1/w/{workspace_id}/change-sets/{change_set_id}/secrets | 
*SecretsApi* | [**updateSecret**](docs/SecretsApi.md#updatesecret) | **PUT** /v1/w/{workspace_id}/change-sets/{change_set_id}/secrets/{secret_id} | 
*WhoamiApi* | [**whoami**](docs/WhoamiApi.md#whoami) | **GET** /whoami | 


### Documentation For Models

 - [ActionReference](docs/ActionReference.md)
 - [ActionReferenceOneOf](docs/ActionReferenceOneOf.md)
 - [ActionReferenceOneOf1](docs/ActionReferenceOneOf1.md)
 - [ActionV1RequestPath](docs/ActionV1RequestPath.md)
 - [ActionViewV1](docs/ActionViewV1.md)
 - [AddActionV1Request](docs/AddActionV1Request.md)
 - [AddActionV1Response](docs/AddActionV1Response.md)
 - [ApiError](docs/ApiError.md)
 - [ApiSuccessString](docs/ApiSuccessString.md)
 - [CancelActionV1Response](docs/CancelActionV1Response.md)
 - [ChangeSetViewV1](docs/ChangeSetViewV1.md)
 - [ComponentPropKey](docs/ComponentPropKey.md)
 - [ComponentPropViewV1](docs/ComponentPropViewV1.md)
 - [ComponentReference](docs/ComponentReference.md)
 - [ComponentReferenceOneOf](docs/ComponentReferenceOneOf.md)
 - [ComponentReferenceOneOf1](docs/ComponentReferenceOneOf1.md)
 - [ComponentV1RequestPath](docs/ComponentV1RequestPath.md)
 - [ComponentViewV1](docs/ComponentViewV1.md)
 - [Connection](docs/Connection.md)
 - [ConnectionDetails](docs/ConnectionDetails.md)
 - [ConnectionOneOf](docs/ConnectionOneOf.md)
 - [ConnectionOneOf1](docs/ConnectionOneOf1.md)
 - [ConnectionPoint](docs/ConnectionPoint.md)
 - [ConnectionViewV1](docs/ConnectionViewV1.md)
 - [ConnectionViewV1OneOf](docs/ConnectionViewV1OneOf.md)
 - [ConnectionViewV1OneOf1](docs/ConnectionViewV1OneOf1.md)
 - [ConnectionViewV1OneOf2](docs/ConnectionViewV1OneOf2.md)
 - [ConnectionViewV1OneOf3](docs/ConnectionViewV1OneOf3.md)
 - [CreateChangeSetV1Request](docs/CreateChangeSetV1Request.md)
 - [CreateChangeSetV1Response](docs/CreateChangeSetV1Response.md)
 - [CreateComponentV1Request](docs/CreateComponentV1Request.md)
 - [CreateComponentV1Response](docs/CreateComponentV1Response.md)
 - [CreateSecretV1Request](docs/CreateSecretV1Request.md)
 - [CreateSecretV1Response](docs/CreateSecretV1Response.md)
 - [DeleteChangeSetV1Response](docs/DeleteChangeSetV1Response.md)
 - [DeleteComponentV1Response](docs/DeleteComponentV1Response.md)
 - [DeleteSecretV1Response](docs/DeleteSecretV1Response.md)
 - [ErrorDetail](docs/ErrorDetail.md)
 - [ErrorResponse](docs/ErrorResponse.md)
 - [ExecuteManagementFunctionV1Request](docs/ExecuteManagementFunctionV1Request.md)
 - [ExecuteManagementFunctionV1Response](docs/ExecuteManagementFunctionV1Response.md)
 - [FindComponentV1Params](docs/FindComponentV1Params.md)
 - [FindSchemaV1Params](docs/FindSchemaV1Params.md)
 - [FindSchemaV1Response](docs/FindSchemaV1Response.md)
 - [ForceApplyChangeSetV1Response](docs/ForceApplyChangeSetV1Response.md)
 - [FuncRunLogViewV1](docs/FuncRunLogViewV1.md)
 - [FuncRunV1RequestPath](docs/FuncRunV1RequestPath.md)
 - [FuncRunViewV1](docs/FuncRunViewV1.md)
 - [GetActionsV1Response](docs/GetActionsV1Response.md)
 - [GetChangeSetV1Response](docs/GetChangeSetV1Response.md)
 - [GetComponentV1Response](docs/GetComponentV1Response.md)
 - [GetComponentV1ResponseActionFunction](docs/GetComponentV1ResponseActionFunction.md)
 - [GetComponentV1ResponseManagementFunction](docs/GetComponentV1ResponseManagementFunction.md)
 - [GetFuncRunV1Response](docs/GetFuncRunV1Response.md)
 - [GetFuncV1Response](docs/GetFuncV1Response.md)
 - [GetSchemaV1Response](docs/GetSchemaV1Response.md)
 - [GetSchemaVariantV1Response](docs/GetSchemaVariantV1Response.md)
 - [HashMapValue](docs/HashMapValue.md)
 - [IncomingConnectionViewV1](docs/IncomingConnectionViewV1.md)
 - [ListChangeSetV1Response](docs/ListChangeSetV1Response.md)
 - [ListComponentsV1Response](docs/ListComponentsV1Response.md)
 - [ListSchemaV1Response](docs/ListSchemaV1Response.md)
 - [ManagedByConnectionViewV1](docs/ManagedByConnectionViewV1.md)
 - [ManagementFunctionReference](docs/ManagementFunctionReference.md)
 - [ManagementFunctionReferenceOneOf](docs/ManagementFunctionReferenceOneOf.md)
 - [ManagementFunctionReferenceOneOf1](docs/ManagementFunctionReferenceOneOf1.md)
 - [ManagingConnectionViewV1](docs/ManagingConnectionViewV1.md)
 - [MergeStatusV1Response](docs/MergeStatusV1Response.md)
 - [MergeStatusV1ResponseAction](docs/MergeStatusV1ResponseAction.md)
 - [MergeStatusV1ResponseActionComponent](docs/MergeStatusV1ResponseActionComponent.md)
 - [OutgoingConnectionViewV1](docs/OutgoingConnectionViewV1.md)
 - [OutputLineViewV1](docs/OutputLineViewV1.md)
 - [PropSchemaV1](docs/PropSchemaV1.md)
 - [PurgeOpenChangeSetsV1Response](docs/PurgeOpenChangeSetsV1Response.md)
 - [PutOnHoldActionV1Response](docs/PutOnHoldActionV1Response.md)
 - [RequestApprovalChangeSetV1Response](docs/RequestApprovalChangeSetV1Response.md)
 - [RetryActionV1Response](docs/RetryActionV1Response.md)
 - [SchemaResponse](docs/SchemaResponse.md)
 - [SchemaV1RequestPath](docs/SchemaV1RequestPath.md)
 - [SchemaVariantV1RequestPath](docs/SchemaVariantV1RequestPath.md)
 - [SecretDefinitionV1](docs/SecretDefinitionV1.md)
 - [SecretFormDataV1](docs/SecretFormDataV1.md)
 - [SecretPropKey](docs/SecretPropKey.md)
 - [SecretV1](docs/SecretV1.md)
 - [SocketDirection](docs/SocketDirection.md)
 - [SocketViewV1](docs/SocketViewV1.md)
 - [SystemStatusResponse](docs/SystemStatusResponse.md)
 - [UpdateComponentV1Request](docs/UpdateComponentV1Request.md)
 - [UpdateComponentV1Response](docs/UpdateComponentV1Response.md)
 - [UpdateSecretV1Request](docs/UpdateSecretV1Request.md)
 - [UpdateSecretV1Response](docs/UpdateSecretV1Response.md)
 - [ViewV1](docs/ViewV1.md)
 - [WhoamiResponse](docs/WhoamiResponse.md)


<a id="documentation-for-authorization"></a>
## Documentation For Authorization

Endpoints do not require authorization.

