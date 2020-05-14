export type Maybe<T> = T | null;
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: string;
  String: string;
  Boolean: boolean;
  Int: number;
  Float: number;
};

export type AwsEksClusterRuntimeAddNodegroupReply = {
  __typename?: "AwsEksClusterRuntimeAddNodegroupReply";
  /** event */
  event?: Maybe<AwsEksClusterRuntimeEntityEvent>;
};

export type AwsEksClusterRuntimeAddNodegroupRequest = {
  /** entityId */
  entityId?: Maybe<Scalars["String"]>;
};

export enum AwsEksClusterRuntimeBool {
  BoolUnknown = "BOOL_UNKNOWN",
  False = "FALSE",
  True = "TRUE",
}

export enum AwsEksClusterRuntimeClusterStatus {
  Active = "ACTIVE",
  ClusterStatusUnknown = "CLUSTER_STATUS_UNKNOWN",
  Creating = "CREATING",
  Deleting = "DELETING",
  Failed = "FAILED",
  Updating = "UPDATING",
}

export type AwsEksClusterRuntimeComponent = {
  __typename?: "AwsEksClusterRuntimeComponent";
  /** description */
  description?: Maybe<Scalars["String"]>;
  /** displayName */
  displayName?: Maybe<Scalars["String"]>;
  /** displayTypeName */
  displayTypeName?: Maybe<Scalars["String"]>;
  /** id */
  id?: Maybe<Scalars["ID"]>;
  /** integrationId */
  integration?: Maybe<Integration>;
  /** integrationId */
  integrationId?: Maybe<Scalars["String"]>;
  /** integrationServiceId */
  integrationService?: Maybe<IntegrationService>;
  /** integrationServiceId */
  integrationServiceId?: Maybe<Scalars["String"]>;
  /** kubernetesVersion */
  kubernetesVersion?: Maybe<Scalars["String"]>;
  /** name */
  name?: Maybe<Scalars["String"]>;
  /** naturalKey */
  naturalKey?: Maybe<Scalars["String"]>;
  /** tenantIds */
  tenantIds?: Maybe<Array<Scalars["String"]>>;
  /** typeName */
  typeName?: Maybe<Scalars["String"]>;
  /** version */
  version?: Maybe<Scalars["Int"]>;
};

export type AwsEksClusterRuntimeConstraints = {
  __typename?: "AwsEksClusterRuntimeConstraints";
  /** displayName */
  displayName?: Maybe<Scalars["String"]>;
  /** integrationId */
  integrationId?: Maybe<Scalars["String"]>;
  /** integrationServiceId */
  integrationServiceId?: Maybe<Scalars["String"]>;
  /** kubernetesVersion */
  kubernetesVersion?: Maybe<Scalars["String"]>;
  /** name */
  name?: Maybe<Scalars["String"]>;
};

export type AwsEksClusterRuntimeCreateEntityReply = {
  __typename?: "AwsEksClusterRuntimeCreateEntityReply";
  /** entity */
  entity?: Maybe<AwsEksClusterRuntimeEntity>;
  /** event */
  event?: Maybe<AwsEksClusterRuntimeEntityEvent>;
};

export type AwsEksClusterRuntimeCreateEntityRequest = {
  /** cloudwatchLogs */
  cloudwatchLogs?: Maybe<Scalars["Boolean"]>;
  /** constraints */
  constraints?: Maybe<AwsEksClusterRuntimePickComponentRequest>;
  /** description */
  description?: Maybe<Scalars["String"]>;
  /** displayName */
  displayName?: Maybe<Scalars["String"]>;
  /** name */
  name?: Maybe<Scalars["String"]>;
  /** nodegroupDesiredSize */
  nodegroupDesiredSize?: Maybe<Scalars["Int"]>;
  /** nodegroupDiskSize */
  nodegroupDiskSize?: Maybe<Scalars["Int"]>;
  /** nodegroupInstanceType */
  nodegroupInstanceType?: Maybe<Scalars["String"]>;
  /** nodegroupMaxSize */
  nodegroupMaxSize?: Maybe<Scalars["Int"]>;
  /** nodegroupMinSize */
  nodegroupMinSize?: Maybe<Scalars["Int"]>;
  /** nodegroupSshKeyId */
  nodegroupSshKeyId?: Maybe<Scalars["String"]>;
  /** tags */
  tags?: Maybe<Array<AwsEksClusterRuntimeTagRequest>>;
  /** workspaceId */
  workspaceId?: Maybe<Scalars["String"]>;
};

export type AwsEksClusterRuntimeEntity = {
  __typename?: "AwsEksClusterRuntimeEntity";
  /** billingAccountId */
  billingAccount?: Maybe<BillingAccount>;
  /** billingAccountId */
  billingAccountId?: Maybe<Scalars["String"]>;
  /** certificateAuthority */
  certificateAuthority?: Maybe<Scalars["String"]>;
  /** cloudwatchLogs */
  cloudwatchLogs?: Maybe<Scalars["Boolean"]>;
  /** clusterName */
  clusterName?: Maybe<Scalars["String"]>;
  /** clusterStatus */
  clusterStatus?: Maybe<AwsEksClusterRuntimeClusterStatus>;
  /** componentId */
  componentId?: Maybe<Scalars["String"]>;
  /** constraints */
  constraints?: Maybe<AwsEksClusterRuntimeConstraints>;
  /** description */
  description?: Maybe<Scalars["String"]>;
  /** displayName */
  displayName?: Maybe<Scalars["String"]>;
  /** endpoint */
  endpoint?: Maybe<Scalars["String"]>;
  /** endpointPrivateAccess */
  endpointPrivateAccess?: Maybe<AwsEksClusterRuntimeBool>;
  /** endpointPublicAccess */
  endpointPublicAccess?: Maybe<AwsEksClusterRuntimeBool>;
  /** id */
  entityEvents?: Maybe<AwsEksClusterRuntimeListEntityEventsReply>;
  /** id */
  id?: Maybe<Scalars["ID"]>;
  /** implicitConstraints */
  implicitConstraints?: Maybe<Array<AwsEksClusterRuntimeImplicitConstraint>>;
  /** integrationId */
  integration?: Maybe<Integration>;
  /** integrationId */
  integrationId?: Maybe<Scalars["String"]>;
  /** integrationServiceId */
  integrationService?: Maybe<IntegrationService>;
  /** integrationServiceId */
  integrationServiceId?: Maybe<Scalars["String"]>;
  /** kubernetesVersion */
  kubernetesVersion?: Maybe<Scalars["String"]>;
  /** linkedEntityIds */
  linkedEntityIds?: Maybe<Array<Scalars["String"]>>;
  /** name */
  name?: Maybe<Scalars["String"]>;
  /** naturalKey */
  naturalKey?: Maybe<Scalars["String"]>;
  /** nodegroupDesiredSize */
  nodegroupDesiredSize?: Maybe<Scalars["Int"]>;
  /** nodegroupDiskSize */
  nodegroupDiskSize?: Maybe<Scalars["Int"]>;
  /** nodegroupInstanceType */
  nodegroupInstanceType?: Maybe<Scalars["String"]>;
  /** nodegroupMaxSize */
  nodegroupMaxSize?: Maybe<Scalars["Int"]>;
  /** nodegroupMinSize */
  nodegroupMinSize?: Maybe<Scalars["Int"]>;
  /** nodegroupName */
  nodegroupName?: Maybe<Scalars["String"]>;
  /** nodegroupSshKeyId */
  nodegroupSshKeyId?: Maybe<Scalars["String"]>;
  /** nodegroupStatus */
  nodegroupStatus?: Maybe<AwsEksClusterRuntimeNodegroupStatus>;
  /** organizationId */
  organization?: Maybe<Organization>;
  /** organizationId */
  organizationId?: Maybe<Scalars["String"]>;
  /** state */
  state?: Maybe<AwsEksClusterRuntimeState>;
  /** tags */
  tags?: Maybe<Array<AwsEksClusterRuntimeTag>>;
  /** tenantIds */
  tenantIds?: Maybe<Array<Scalars["String"]>>;
  /** typeName */
  typeName?: Maybe<Scalars["String"]>;
  /** workspaceId */
  workspace?: Maybe<Workspace>;
  /** workspaceId */
  workspaceId?: Maybe<Scalars["String"]>;
};

export type AwsEksClusterRuntimeEntityEntityEventsArgs = {
  input?: Maybe<AwsEksClusterRuntimeListEntityEventsRequest>;
};

export type AwsEksClusterRuntimeEntityEvent = EntityEvent & {
  __typename?: "AwsEksClusterRuntimeEntityEvent";
  /** actionName */
  actionName?: Maybe<Scalars["String"]>;
  /** billingAccountId */
  billingAccountId?: Maybe<Scalars["String"]>;
  /** componentId */
  componentId?: Maybe<Scalars["String"]>;
  /** createTime */
  createTime?: Maybe<Scalars["String"]>;
  /** entityId */
  entity?: Maybe<AwsEksClusterRuntimeEntity>;
  /** entityId */
  entityId?: Maybe<Scalars["String"]>;
  /** errorLines */
  errorLines?: Maybe<Array<Scalars["String"]>>;
  /** errorMessage */
  errorMessage?: Maybe<Scalars["String"]>;
  /** finalized */
  finalized?: Maybe<Scalars["Boolean"]>;
  /** finalTime */
  finalTime?: Maybe<Scalars["String"]>;
  /** id */
  id?: Maybe<Scalars["ID"]>;
  /** inputEntity */
  inputEntity?: Maybe<AwsEksClusterRuntimeEntity>;
  /** integrationId */
  integrationId?: Maybe<Scalars["String"]>;
  /** integrationServiceId */
  integrationServiceId?: Maybe<Scalars["String"]>;
  /** naturalKey */
  naturalKey?: Maybe<Scalars["String"]>;
  /** nextState */
  nextState?: Maybe<AwsEksClusterRuntimeNextState>;
  /** organizationId */
  organizationId?: Maybe<Scalars["String"]>;
  /** outputEntity */
  outputEntity?: Maybe<AwsEksClusterRuntimeEntity>;
  /** outputLines */
  outputLines?: Maybe<Array<Scalars["String"]>>;
  /** previousEntity */
  previousEntity?: Maybe<AwsEksClusterRuntimeEntity>;
  /** success */
  success?: Maybe<Scalars["Boolean"]>;
  /** tenantIds */
  tenantIds?: Maybe<Array<Scalars["String"]>>;
  /** typeName */
  typeName?: Maybe<Scalars["String"]>;
  /** updatedTime */
  updatedTime?: Maybe<Scalars["String"]>;
  /** userId */
  user?: Maybe<User>;
  /** userId */
  userId?: Maybe<Scalars["String"]>;
  /** workspaceId */
  workspaceId?: Maybe<Scalars["String"]>;
};

export type AwsEksClusterRuntimeGetComponentReply = {
  __typename?: "AwsEksClusterRuntimeGetComponentReply";
  /** component */
  component?: Maybe<AwsEksClusterRuntimeComponent>;
};

export type AwsEksClusterRuntimeGetComponentRequest = {
  /** componentId */
  componentId?: Maybe<Scalars["String"]>;
};

export type AwsEksClusterRuntimeGetEntityReply = {
  __typename?: "AwsEksClusterRuntimeGetEntityReply";
  /** entity */
  entity?: Maybe<AwsEksClusterRuntimeEntity>;
};

export type AwsEksClusterRuntimeGetEntityRequest = {
  /** entityId */
  entityId?: Maybe<Scalars["String"]>;
};

export type AwsEksClusterRuntimeImplicitConstraint = {
  __typename?: "AwsEksClusterRuntimeImplicitConstraint";
  /** field */
  field?: Maybe<Scalars["String"]>;
  /** value */
  value?: Maybe<Scalars["String"]>;
};

export type AwsEksClusterRuntimeListComponentsReply = {
  __typename?: "AwsEksClusterRuntimeListComponentsReply";
  /** items */
  items?: Maybe<Array<AwsEksClusterRuntimeComponent>>;
  /** nextPageToken */
  nextPageToken?: Maybe<Scalars["String"]>;
  /** totalCount */
  totalCount?: Maybe<Scalars["Int"]>;
};

export type AwsEksClusterRuntimeListComponentsRequest = {
  /** orderBy */
  orderBy?: Maybe<Scalars["String"]>;
  /** orderByDirection */
  orderByDirection?: Maybe<DataOrderByDirection>;
  /** pageSize */
  pageSize?: Maybe<Scalars["Int"]>;
  /** pageToken */
  pageToken?: Maybe<Scalars["String"]>;
  /** query */
  query?: Maybe<DataQuery>;
  /** scopeByTenantId */
  scopeByTenantId?: Maybe<Scalars["String"]>;
};

export type AwsEksClusterRuntimeListEntitiesReply = {
  __typename?: "AwsEksClusterRuntimeListEntitiesReply";
  /** items */
  items?: Maybe<Array<AwsEksClusterRuntimeEntity>>;
  /** nextPageToken */
  nextPageToken?: Maybe<Scalars["String"]>;
  /** totalCount */
  totalCount?: Maybe<Scalars["Int"]>;
};

export type AwsEksClusterRuntimeListEntitiesRequest = {
  /** orderBy */
  orderBy?: Maybe<Scalars["String"]>;
  /** orderByDirection */
  orderByDirection?: Maybe<DataOrderByDirection>;
  /** pageSize */
  pageSize?: Maybe<Scalars["Int"]>;
  /** pageToken */
  pageToken?: Maybe<Scalars["String"]>;
  /** query */
  query?: Maybe<DataQuery>;
  /** scopeByTenantId */
  scopeByTenantId?: Maybe<Scalars["String"]>;
};

export type AwsEksClusterRuntimeListEntityEventsReply = {
  __typename?: "AwsEksClusterRuntimeListEntityEventsReply";
  /** items */
  items?: Maybe<Array<AwsEksClusterRuntimeEntityEvent>>;
  /** nextPageToken */
  nextPageToken?: Maybe<Scalars["String"]>;
  /** totalCount */
  totalCount?: Maybe<Scalars["Int"]>;
};

export type AwsEksClusterRuntimeListEntityEventsRequest = {
  /** orderBy */
  orderBy?: Maybe<Scalars["String"]>;
  /** orderByDirection */
  orderByDirection?: Maybe<DataOrderByDirection>;
  /** pageSize */
  pageSize?: Maybe<Scalars["Int"]>;
  /** pageToken */
  pageToken?: Maybe<Scalars["String"]>;
  /** query */
  query?: Maybe<DataQuery>;
  /** scopeByTenantId */
  scopeByTenantId?: Maybe<Scalars["String"]>;
};

export enum AwsEksClusterRuntimeNextState {
  Error = "ERROR",
  None = "NONE",
  Ok = "OK",
  Uninitialized = "UNINITIALIZED",
}

export enum AwsEksClusterRuntimeNodegroupStatus {
  NodegroupActive = "NODEGROUP_ACTIVE",
  NodegroupCreateFailed = "NODEGROUP_CREATE_FAILED",
  NodegroupCreating = "NODEGROUP_CREATING",
  NodegroupDegraded = "NODEGROUP_DEGRADED",
  NodegroupDeleteFailed = "NODEGROUP_DELETE_FAILED",
  NodegroupDeleting = "NODEGROUP_DELETING",
  NodegroupStatusUnknown = "NODEGROUP_STATUS_UNKNOWN",
  NodegroupUpdating = "NODEGROUP_UPDATING",
}

export type AwsEksClusterRuntimePickComponentReply = {
  __typename?: "AwsEksClusterRuntimePickComponentReply";
  /** component */
  component?: Maybe<AwsEksClusterRuntimeComponent>;
  /** implicitConstraints */
  implicitConstraints?: Maybe<Array<AwsEksClusterRuntimeImplicitConstraint>>;
};

export type AwsEksClusterRuntimePickComponentRequest = {
  /** displayName */
  displayName?: Maybe<Scalars["String"]>;
  /** integrationId */
  integrationId?: Maybe<Scalars["String"]>;
  /** integrationServiceId */
  integrationServiceId?: Maybe<Scalars["String"]>;
  /** kubernetesVersion */
  kubernetesVersion?: Maybe<Scalars["String"]>;
  /** name */
  name?: Maybe<Scalars["String"]>;
};

export enum AwsEksClusterRuntimeState {
  Error = "ERROR",
  Ok = "OK",
  StateUnknown = "STATE_UNKNOWN",
  Transition = "TRANSITION",
}

export type AwsEksClusterRuntimeSyncEntityReply = {
  __typename?: "AwsEksClusterRuntimeSyncEntityReply";
  /** event */
  event?: Maybe<AwsEksClusterRuntimeEntityEvent>;
};

export type AwsEksClusterRuntimeSyncEntityRequest = {
  /** entityId */
  entityId?: Maybe<Scalars["String"]>;
};

export type AwsEksClusterRuntimeTag = {
  __typename?: "AwsEksClusterRuntimeTag";
  /** key */
  key?: Maybe<Scalars["String"]>;
  /** value */
  value?: Maybe<Scalars["String"]>;
};

export type AwsEksClusterRuntimeTagRequest = {
  /** key */
  key?: Maybe<Scalars["String"]>;
  /** value */
  value?: Maybe<Scalars["String"]>;
};

export type BillingAccount = {
  __typename?: "BillingAccount";
  /** displayName */
  displayName?: Maybe<Scalars["String"]>;
  /** id */
  id?: Maybe<Scalars["ID"]>;
  /** id */
  integrationInstances?: Maybe<ListIntegrationInstancesReply>;
  /** naturalKey */
  naturalKey?: Maybe<Scalars["String"]>;
  /** id */
  organizations?: Maybe<ListOrganizationsReply>;
  /** shortName */
  shortName?: Maybe<Scalars["String"]>;
  /** tenantIds */
  tenantIds?: Maybe<Array<Scalars["String"]>>;
  /** typeName */
  typeName?: Maybe<Scalars["String"]>;
  /** id */
  users?: Maybe<ListUsersReply>;
};

export type BillingAccountIntegrationInstancesArgs = {
  input?: Maybe<ListIntegrationInstancesRequest>;
};

export type BillingAccountOrganizationsArgs = {
  input?: Maybe<ListOrganizationsRequest>;
};

export type BillingAccountUsersArgs = {
  input?: Maybe<ListUsersRequest>;
};

export type Capability = {
  __typename?: "Capability";
  /** actions */
  actions?: Maybe<Array<Scalars["String"]>>;
  /** subject */
  subject?: Maybe<Scalars["String"]>;
};

export type CreateAccountReply = {
  __typename?: "CreateAccountReply";
  /** billingAccount */
  billingAccount?: Maybe<BillingAccount>;
  /** user */
  user?: Maybe<User>;
};

export type CreateAccountRequest = {
  /** billingAccount */
  billingAccount?: Maybe<CreateBillingAccountRequest>;
  /** user */
  user?: Maybe<CreateUserRequest>;
};

export type CreateBillingAccountReply = {
  __typename?: "CreateBillingAccountReply";
  /** billingAccount */
  billingAccount?: Maybe<BillingAccount>;
};

export type CreateBillingAccountRequest = {
  /** displayName */
  displayName?: Maybe<Scalars["String"]>;
  /** shortName */
  shortName?: Maybe<Scalars["String"]>;
};

export type CreateIntegrationInstanceReply = {
  __typename?: "CreateIntegrationInstanceReply";
  /** integrationInstance */
  integrationInstance?: Maybe<IntegrationInstance>;
};

export type CreateIntegrationInstanceRequest = {
  /** displayName */
  displayName?: Maybe<Scalars["String"]>;
  /** integrationId */
  integrationId?: Maybe<Scalars["String"]>;
  /** integrationOptionValues */
  integrationOptionValues?: Maybe<Array<CreateIntegrationOptionValueRequest>>;
  /** name */
  name?: Maybe<Scalars["String"]>;
};

export type CreateIntegrationOptionValueRequest = {
  /** name */
  name?: Maybe<Scalars["String"]>;
  /** optionType */
  optionType?: Maybe<IntegrationOptionType>;
  /** value */
  value?: Maybe<Scalars["String"]>;
};

export type CreateUserReply = {
  __typename?: "CreateUserReply";
  /** user */
  user?: Maybe<User>;
};

export type CreateUserRequest = {
  /** displayName */
  displayName?: Maybe<Scalars["String"]>;
  /** email */
  email?: Maybe<Scalars["String"]>;
  /** familyName */
  familyName?: Maybe<Scalars["String"]>;
  /** givenName */
  givenName?: Maybe<Scalars["String"]>;
  /** password */
  password?: Maybe<Scalars["String"]>;
};

export enum DataOrderByDirection {
  Asc = "ASC",
  Desc = "DESC",
}

export type DataPageToken = {
  /** containedWithin */
  containedWithin?: Maybe<Scalars["String"]>;
  /** itemId */
  itemId?: Maybe<Scalars["String"]>;
  /** orderBy */
  orderBy?: Maybe<Scalars["String"]>;
  /** orderByDirection */
  orderByDirection?: Maybe<DataOrderByDirection>;
  /** pageSize */
  pageSize?: Maybe<Scalars["Int"]>;
  /** query */
  query?: Maybe<DataQuery>;
};

export type DataQuery = {
  /** booleanTerm */
  booleanTerm?: Maybe<DataQueryBooleanLogic>;
  /** isNot */
  isNot?: Maybe<Scalars["Boolean"]>;
  /** items */
  items?: Maybe<Array<DataQueryExpressionOption>>;
};

export enum DataQueryBooleanLogic {
  And = "AND",
  Or = "OR",
}

export enum DataQueryComparison {
  Contains = "CONTAINS",
  Equals = "EQUALS",
  Like = "LIKE",
  Notequals = "NOTEQUALS",
  Notlike = "NOTLIKE",
}

export type DataQueryExpression = {
  /** comparison */
  comparison?: Maybe<DataQueryComparison>;
  /** field */
  field?: Maybe<Scalars["String"]>;
  /** fieldType */
  fieldType?: Maybe<DataQueryFieldType>;
  /** value */
  value?: Maybe<Scalars["String"]>;
};

export type DataQueryExpressionOption = {
  /** expression */
  expression?: Maybe<DataQueryExpression>;
  /** query */
  query?: Maybe<DataQuery>;
};

export enum DataQueryFieldType {
  Int = "INT",
  String = "STRING",
}

export type EntityEvent = {
  actionName?: Maybe<Scalars["String"]>;
  billingAccountId?: Maybe<Scalars["String"]>;
  componentId?: Maybe<Scalars["String"]>;
  createTime?: Maybe<Scalars["String"]>;
  entityId?: Maybe<Scalars["String"]>;
  errorLines?: Maybe<Array<Scalars["String"]>>;
  errorMessage?: Maybe<Scalars["String"]>;
  finalized?: Maybe<Scalars["Boolean"]>;
  finalTime?: Maybe<Scalars["String"]>;
  id?: Maybe<Scalars["ID"]>;
  integrationId?: Maybe<Scalars["String"]>;
  integrationServiceId?: Maybe<Scalars["String"]>;
  naturalKey?: Maybe<Scalars["String"]>;
  organizationId?: Maybe<Scalars["String"]>;
  outputLines?: Maybe<Array<Scalars["String"]>>;
  success?: Maybe<Scalars["Boolean"]>;
  tenantIds?: Maybe<Array<Scalars["String"]>>;
  typeName?: Maybe<Scalars["String"]>;
  updatedTime?: Maybe<Scalars["String"]>;
  userId?: Maybe<Scalars["String"]>;
  workspaceId?: Maybe<Scalars["String"]>;
};

export type GetBillingAccountReply = {
  __typename?: "GetBillingAccountReply";
  /** billingAccount */
  billingAccount?: Maybe<BillingAccount>;
};

export type GetBillingAccountRequest = {
  /** billingAccountId */
  billingAccountId?: Maybe<Scalars["String"]>;
};

export type GetIntegrationReply = {
  __typename?: "GetIntegrationReply";
  /** integration */
  integration?: Maybe<Integration>;
};

export type GetIntegrationRequest = {
  /** integrationId */
  integrationId?: Maybe<Scalars["String"]>;
};

export type GetIntegrationServiceReply = {
  __typename?: "GetIntegrationServiceReply";
  /** integrationService */
  integrationService?: Maybe<IntegrationService>;
};

export type GetIntegrationServiceRequest = {
  /** integrationServiceId */
  integrationServiceId?: Maybe<Scalars["String"]>;
};

export type GetOrganizationReply = {
  __typename?: "GetOrganizationReply";
  /** organization */
  organization?: Maybe<Organization>;
};

export type GetOrganizationRequest = {
  /** organizationId */
  organizationId?: Maybe<Scalars["String"]>;
};

export type GetUserReply = {
  __typename?: "GetUserReply";
  /** user */
  user?: Maybe<User>;
};

export type GetUserRequest = {
  /** userId */
  userId?: Maybe<Scalars["String"]>;
};

export type Group = {
  __typename?: "Group";
  /** billingAccountId */
  billingAccountId?: Maybe<Scalars["String"]>;
  /** capabilities */
  capabilities?: Maybe<Array<Capability>>;
  /** displayName */
  displayName?: Maybe<Scalars["String"]>;
  /** id */
  id?: Maybe<Scalars["ID"]>;
  /** name */
  name?: Maybe<Scalars["String"]>;
  /** naturalKey */
  naturalKey?: Maybe<Scalars["String"]>;
  /** tenantIds */
  tenantIds?: Maybe<Array<Scalars["String"]>>;
  /** typeName */
  typeName?: Maybe<Scalars["String"]>;
  /** userIds */
  userIds?: Maybe<Array<Scalars["String"]>>;
};

export type Integration = {
  __typename?: "Integration";
  /** displayName */
  displayName?: Maybe<Scalars["String"]>;
  /** id */
  id?: Maybe<Scalars["ID"]>;
  /** integrationOptions */
  integrationOptions?: Maybe<Array<IntegrationOption>>;
  /** id */
  integrationServices?: Maybe<ListIntegrationServicesReply>;
  /** name */
  name?: Maybe<Scalars["String"]>;
  /** naturalKey */
  naturalKey?: Maybe<Scalars["String"]>;
  /** tenantIds */
  tenantIds?: Maybe<Array<Scalars["String"]>>;
  /** typeName */
  typeName?: Maybe<Scalars["String"]>;
  /** version */
  version?: Maybe<Scalars["Int"]>;
};

export type IntegrationIntegrationServicesArgs = {
  input?: Maybe<ListIntegrationServicesRequest>;
};

export type IntegrationInstance = {
  __typename?: "IntegrationInstance";
  /** billingAccountId */
  billingAccount?: Maybe<BillingAccount>;
  /** billingAccountId */
  billingAccountId?: Maybe<Scalars["String"]>;
  /** displayName */
  displayName?: Maybe<Scalars["String"]>;
  /** enabledOnOrganizationIds */
  enabledOnOrganizationIds?: Maybe<Array<Scalars["String"]>>;
  /** enabledOnOrganizationIds */
  enabledOnOrganizations?: Maybe<ListOrganizationsReply>;
  /** enabledOnWorkspaceIds */
  enabledOnWorkspaceIds?: Maybe<Array<Scalars["String"]>>;
  /** enabledOnWorkspaceIds */
  enabledOnWorkspaces?: Maybe<ListWorkspacesReply>;
  /** id */
  id?: Maybe<Scalars["ID"]>;
  /** integrationId */
  integration?: Maybe<Integration>;
  /** integrationId */
  integrationId?: Maybe<Scalars["String"]>;
  /** integrationOptionValues */
  integrationOptionValues?: Maybe<Array<IntegrationOptionValue>>;
  /** integrationServiceIds */
  integrationServiceIds?: Maybe<Array<Scalars["String"]>>;
  /** name */
  name?: Maybe<Scalars["String"]>;
  /** naturalKey */
  naturalKey?: Maybe<Scalars["String"]>;
  /** tenantIds */
  tenantIds?: Maybe<Array<Scalars["String"]>>;
  /** typeName */
  typeName?: Maybe<Scalars["String"]>;
};

export type IntegrationInstanceEnabledOnOrganizationsArgs = {
  input?: Maybe<ListOrganizationsRequest>;
};

export type IntegrationInstanceEnabledOnWorkspacesArgs = {
  input?: Maybe<ListWorkspacesRequest>;
};

export type IntegrationOption = {
  __typename?: "IntegrationOption";
  /** displayName */
  displayName?: Maybe<Scalars["String"]>;
  /** name */
  name?: Maybe<Scalars["String"]>;
  /** optionType */
  optionType?: Maybe<IntegrationOptionType>;
};

export enum IntegrationOptionType {
  Secret = "SECRET",
  String = "STRING",
}

export type IntegrationOptionValue = {
  __typename?: "IntegrationOptionValue";
  /** name */
  name?: Maybe<Scalars["String"]>;
  /** optionType */
  optionType?: Maybe<IntegrationOptionType>;
  /** value */
  value?: Maybe<Scalars["String"]>;
};

export type IntegrationService = {
  __typename?: "IntegrationService";
  /** displayName */
  displayName?: Maybe<Scalars["String"]>;
  /** id */
  id?: Maybe<Scalars["ID"]>;
  /** integrationId */
  integration?: Maybe<Integration>;
  /** integrationId */
  integrationId?: Maybe<Scalars["String"]>;
  /** name */
  name?: Maybe<Scalars["String"]>;
  /** naturalKey */
  naturalKey?: Maybe<Scalars["String"]>;
  /** tenantIds */
  tenantIds?: Maybe<Array<Scalars["String"]>>;
  /** typeName */
  typeName?: Maybe<Scalars["String"]>;
  /** version */
  version?: Maybe<Scalars["Int"]>;
};

export type KubernetesContainer = {
  __typename?: "KubernetesContainer";
  /** image */
  image?: Maybe<Scalars["String"]>;
  /** name */
  name?: Maybe<Scalars["String"]>;
  /** ports */
  ports?: Maybe<Array<KubernetesContainerPorts>>;
};

export type KubernetesContainerPorts = {
  __typename?: "KubernetesContainerPorts";
  /** containerPort */
  containerPort?: Maybe<Scalars["Int"]>;
};

export type KubernetesContainerPortsRequest = {
  /** containerPort */
  containerPort?: Maybe<Scalars["Int"]>;
};

export type KubernetesContainerRequest = {
  /** image */
  image?: Maybe<Scalars["String"]>;
  /** name */
  name?: Maybe<Scalars["String"]>;
  /** ports */
  ports?: Maybe<Array<KubernetesContainerPortsRequest>>;
};

export type KubernetesDeploymentComponent = {
  __typename?: "KubernetesDeploymentComponent";
  /** description */
  description?: Maybe<Scalars["String"]>;
  /** displayName */
  displayName?: Maybe<Scalars["String"]>;
  /** displayTypeName */
  displayTypeName?: Maybe<Scalars["String"]>;
  /** id */
  id?: Maybe<Scalars["ID"]>;
  /** integrationId */
  integration?: Maybe<Integration>;
  /** integrationId */
  integrationId?: Maybe<Scalars["String"]>;
  /** integrationServiceId */
  integrationService?: Maybe<IntegrationService>;
  /** integrationServiceId */
  integrationServiceId?: Maybe<Scalars["String"]>;
  /** kubernetesVersion */
  kubernetesVersion?: Maybe<Scalars["String"]>;
  /** name */
  name?: Maybe<Scalars["String"]>;
  /** naturalKey */
  naturalKey?: Maybe<Scalars["String"]>;
  /** tenantIds */
  tenantIds?: Maybe<Array<Scalars["String"]>>;
  /** typeName */
  typeName?: Maybe<Scalars["String"]>;
  /** version */
  version?: Maybe<Scalars["Int"]>;
};

export type KubernetesDeploymentConstraints = {
  __typename?: "KubernetesDeploymentConstraints";
  /** displayName */
  displayName?: Maybe<Scalars["String"]>;
  /** integrationId */
  integrationId?: Maybe<Scalars["String"]>;
  /** integrationServiceId */
  integrationServiceId?: Maybe<Scalars["String"]>;
  /** kubernetesVersion */
  kubernetesVersion?: Maybe<Scalars["String"]>;
  /** name */
  name?: Maybe<Scalars["String"]>;
};

export type KubernetesDeploymentCreateEntityReply = {
  __typename?: "KubernetesDeploymentCreateEntityReply";
  /** entity */
  entity?: Maybe<KubernetesDeploymentEntity>;
  /** event */
  event?: Maybe<KubernetesDeploymentEntityEvent>;
};

export type KubernetesDeploymentCreateEntityRequest = {
  /** constraints */
  constraints?: Maybe<KubernetesDeploymentPickComponentRequest>;
  /** description */
  description?: Maybe<Scalars["String"]>;
  /** displayName */
  displayName?: Maybe<Scalars["String"]>;
  /** name */
  name?: Maybe<Scalars["String"]>;
  /** props */
  props?: Maybe<KubernetesDeploymentPropsRequest>;
  /** workspaceId */
  workspaceId?: Maybe<Scalars["String"]>;
};

export type KubernetesDeploymentDeployment = {
  __typename?: "KubernetesDeploymentDeployment";
  /** apiVersion */
  apiVersion?: Maybe<Scalars["String"]>;
  /** kind */
  kind?: Maybe<Scalars["String"]>;
  /** metadata */
  metadata?: Maybe<KubernetesMetaData>;
  /** spec */
  spec?: Maybe<KubernetesDeploymentSpec>;
};

export type KubernetesDeploymentDeploymentRequest = {
  /** apiVersion */
  apiVersion?: Maybe<Scalars["String"]>;
  /** kind */
  kind?: Maybe<Scalars["String"]>;
  /** metadata */
  metadata?: Maybe<KubernetesMetaDataRequest>;
  /** spec */
  spec?: Maybe<KubernetesDeploymentSpecRequest>;
};

export type KubernetesDeploymentEditPropObjectReply = {
  __typename?: "KubernetesDeploymentEditPropObjectReply";
  /** entity */
  entity?: Maybe<KubernetesDeploymentEntity>;
  /** event */
  event?: Maybe<KubernetesDeploymentEntityEvent>;
};

export type KubernetesDeploymentEditPropObjectRequest = {
  /** entityId */
  entityId?: Maybe<Scalars["String"]>;
  /** prop */
  prop?: Maybe<KubernetesDeploymentDeploymentRequest>;
  /** workspaceId */
  workspaceId?: Maybe<Scalars["String"]>;
};

export type KubernetesDeploymentEditPropObjectYamlReply = {
  __typename?: "KubernetesDeploymentEditPropObjectYamlReply";
  /** entity */
  entity?: Maybe<KubernetesDeploymentEntity>;
  /** event */
  event?: Maybe<KubernetesDeploymentEntityEvent>;
};

export type KubernetesDeploymentEditPropObjectYamlRequest = {
  /** entityId */
  entityId?: Maybe<Scalars["String"]>;
  /** prop */
  prop?: Maybe<Scalars["String"]>;
  /** workspaceId */
  workspaceId?: Maybe<Scalars["String"]>;
};

export type KubernetesDeploymentEntity = {
  __typename?: "KubernetesDeploymentEntity";
  /** billingAccountId */
  billingAccount?: Maybe<BillingAccount>;
  /** billingAccountId */
  billingAccountId?: Maybe<Scalars["String"]>;
  /** componentId */
  componentId?: Maybe<Scalars["String"]>;
  /** constraints */
  constraints?: Maybe<KubernetesDeploymentConstraints>;
  /** description */
  description?: Maybe<Scalars["String"]>;
  /** displayName */
  displayName?: Maybe<Scalars["String"]>;
  /** id */
  entityEvents?: Maybe<KubernetesDeploymentListEntityEventsReply>;
  /** id */
  id?: Maybe<Scalars["ID"]>;
  /** implicitConstraints */
  implicitConstraints?: Maybe<Array<KubernetesDeploymentImplicitConstraint>>;
  /** integrationId */
  integration?: Maybe<Integration>;
  /** integrationId */
  integrationId?: Maybe<Scalars["String"]>;
  /** integrationServiceId */
  integrationService?: Maybe<IntegrationService>;
  /** integrationServiceId */
  integrationServiceId?: Maybe<Scalars["String"]>;
  /** kubernetesVersion */
  kubernetesVersion?: Maybe<Scalars["String"]>;
  /** linkedEntityIds */
  linkedEntityIds?: Maybe<Array<Scalars["String"]>>;
  /** name */
  name?: Maybe<Scalars["String"]>;
  /** naturalKey */
  naturalKey?: Maybe<Scalars["String"]>;
  /** object */
  object?: Maybe<KubernetesDeploymentDeployment>;
  /** objectYaml */
  objectYaml?: Maybe<Scalars["String"]>;
  /** organizationId */
  organization?: Maybe<Organization>;
  /** organizationId */
  organizationId?: Maybe<Scalars["String"]>;
  /** state */
  state?: Maybe<KubernetesDeploymentState>;
  /** tenantIds */
  tenantIds?: Maybe<Array<Scalars["String"]>>;
  /** typeName */
  typeName?: Maybe<Scalars["String"]>;
  /** workspaceId */
  workspace?: Maybe<Workspace>;
  /** workspaceId */
  workspaceId?: Maybe<Scalars["String"]>;
};

export type KubernetesDeploymentEntityEntityEventsArgs = {
  input?: Maybe<KubernetesDeploymentListEntityEventsRequest>;
};

export type KubernetesDeploymentEntityEvent = EntityEvent & {
  __typename?: "KubernetesDeploymentEntityEvent";
  /** actionName */
  actionName?: Maybe<Scalars["String"]>;
  /** billingAccountId */
  billingAccountId?: Maybe<Scalars["String"]>;
  /** componentId */
  componentId?: Maybe<Scalars["String"]>;
  /** createTime */
  createTime?: Maybe<Scalars["String"]>;
  /** entityId */
  entity?: Maybe<KubernetesDeploymentEntity>;
  /** entityId */
  entityId?: Maybe<Scalars["String"]>;
  /** errorLines */
  errorLines?: Maybe<Array<Scalars["String"]>>;
  /** errorMessage */
  errorMessage?: Maybe<Scalars["String"]>;
  /** finalized */
  finalized?: Maybe<Scalars["Boolean"]>;
  /** finalTime */
  finalTime?: Maybe<Scalars["String"]>;
  /** id */
  id?: Maybe<Scalars["ID"]>;
  /** inputEntity */
  inputEntity?: Maybe<KubernetesDeploymentEntity>;
  /** integrationId */
  integrationId?: Maybe<Scalars["String"]>;
  /** integrationServiceId */
  integrationServiceId?: Maybe<Scalars["String"]>;
  /** naturalKey */
  naturalKey?: Maybe<Scalars["String"]>;
  /** nextState */
  nextState?: Maybe<KubernetesDeploymentNextState>;
  /** organizationId */
  organizationId?: Maybe<Scalars["String"]>;
  /** outputEntity */
  outputEntity?: Maybe<KubernetesDeploymentEntity>;
  /** outputLines */
  outputLines?: Maybe<Array<Scalars["String"]>>;
  /** previousEntity */
  previousEntity?: Maybe<KubernetesDeploymentEntity>;
  /** success */
  success?: Maybe<Scalars["Boolean"]>;
  /** tenantIds */
  tenantIds?: Maybe<Array<Scalars["String"]>>;
  /** typeName */
  typeName?: Maybe<Scalars["String"]>;
  /** updatedTime */
  updatedTime?: Maybe<Scalars["String"]>;
  /** userId */
  user?: Maybe<User>;
  /** userId */
  userId?: Maybe<Scalars["String"]>;
  /** workspaceId */
  workspaceId?: Maybe<Scalars["String"]>;
};

export type KubernetesDeploymentGetComponentReply = {
  __typename?: "KubernetesDeploymentGetComponentReply";
  /** component */
  component?: Maybe<KubernetesDeploymentComponent>;
};

export type KubernetesDeploymentGetComponentRequest = {
  /** componentId */
  componentId?: Maybe<Scalars["String"]>;
};

export type KubernetesDeploymentGetEntityReply = {
  __typename?: "KubernetesDeploymentGetEntityReply";
  /** entity */
  entity?: Maybe<KubernetesDeploymentEntity>;
};

export type KubernetesDeploymentGetEntityRequest = {
  /** entityId */
  entityId?: Maybe<Scalars["String"]>;
};

export type KubernetesDeploymentImplicitConstraint = {
  __typename?: "KubernetesDeploymentImplicitConstraint";
  /** field */
  field?: Maybe<Scalars["String"]>;
  /** value */
  value?: Maybe<Scalars["String"]>;
};

export type KubernetesDeploymentListComponentsReply = {
  __typename?: "KubernetesDeploymentListComponentsReply";
  /** items */
  items?: Maybe<Array<KubernetesDeploymentComponent>>;
  /** nextPageToken */
  nextPageToken?: Maybe<Scalars["String"]>;
  /** totalCount */
  totalCount?: Maybe<Scalars["Int"]>;
};

export type KubernetesDeploymentListComponentsRequest = {
  /** orderBy */
  orderBy?: Maybe<Scalars["String"]>;
  /** orderByDirection */
  orderByDirection?: Maybe<DataOrderByDirection>;
  /** pageSize */
  pageSize?: Maybe<Scalars["Int"]>;
  /** pageToken */
  pageToken?: Maybe<Scalars["String"]>;
  /** query */
  query?: Maybe<DataQuery>;
  /** scopeByTenantId */
  scopeByTenantId?: Maybe<Scalars["String"]>;
};

export type KubernetesDeploymentListEntitiesReply = {
  __typename?: "KubernetesDeploymentListEntitiesReply";
  /** items */
  items?: Maybe<Array<KubernetesDeploymentEntity>>;
  /** nextPageToken */
  nextPageToken?: Maybe<Scalars["String"]>;
  /** totalCount */
  totalCount?: Maybe<Scalars["Int"]>;
};

export type KubernetesDeploymentListEntitiesRequest = {
  /** orderBy */
  orderBy?: Maybe<Scalars["String"]>;
  /** orderByDirection */
  orderByDirection?: Maybe<DataOrderByDirection>;
  /** pageSize */
  pageSize?: Maybe<Scalars["Int"]>;
  /** pageToken */
  pageToken?: Maybe<Scalars["String"]>;
  /** query */
  query?: Maybe<DataQuery>;
  /** scopeByTenantId */
  scopeByTenantId?: Maybe<Scalars["String"]>;
};

export type KubernetesDeploymentListEntityEventsReply = {
  __typename?: "KubernetesDeploymentListEntityEventsReply";
  /** items */
  items?: Maybe<Array<KubernetesDeploymentEntityEvent>>;
  /** nextPageToken */
  nextPageToken?: Maybe<Scalars["String"]>;
  /** totalCount */
  totalCount?: Maybe<Scalars["Int"]>;
};

export type KubernetesDeploymentListEntityEventsRequest = {
  /** orderBy */
  orderBy?: Maybe<Scalars["String"]>;
  /** orderByDirection */
  orderByDirection?: Maybe<DataOrderByDirection>;
  /** pageSize */
  pageSize?: Maybe<Scalars["Int"]>;
  /** pageToken */
  pageToken?: Maybe<Scalars["String"]>;
  /** query */
  query?: Maybe<DataQuery>;
  /** scopeByTenantId */
  scopeByTenantId?: Maybe<Scalars["String"]>;
};

export enum KubernetesDeploymentNextState {
  Error = "ERROR",
  None = "NONE",
  Ok = "OK",
  Uninitialized = "UNINITIALIZED",
}

export type KubernetesDeploymentPickComponentReply = {
  __typename?: "KubernetesDeploymentPickComponentReply";
  /** component */
  component?: Maybe<KubernetesDeploymentComponent>;
  /** implicitConstraints */
  implicitConstraints?: Maybe<Array<KubernetesDeploymentImplicitConstraint>>;
};

export type KubernetesDeploymentPickComponentRequest = {
  /** displayName */
  displayName?: Maybe<Scalars["String"]>;
  /** integrationId */
  integrationId?: Maybe<Scalars["String"]>;
  /** integrationServiceId */
  integrationServiceId?: Maybe<Scalars["String"]>;
  /** kubernetesVersion */
  kubernetesVersion?: Maybe<Scalars["String"]>;
  /** name */
  name?: Maybe<Scalars["String"]>;
};

export type KubernetesDeploymentPropsRequest = {
  /** object */
  object?: Maybe<KubernetesDeploymentDeploymentRequest>;
};

export type KubernetesDeploymentSpec = {
  __typename?: "KubernetesDeploymentSpec";
  /** replicas */
  replicas?: Maybe<Scalars["Int"]>;
  /** selector */
  selector?: Maybe<KubernetesLabelSelector>;
  /** template */
  template?: Maybe<KubernetesPodTemplateSpec>;
};

export type KubernetesDeploymentSpecRequest = {
  /** replicas */
  replicas?: Maybe<Scalars["Int"]>;
  /** selector */
  selector?: Maybe<KubernetesLabelSelectorRequest>;
  /** template */
  template?: Maybe<KubernetesPodTemplateSpecRequest>;
};

export enum KubernetesDeploymentState {
  Error = "ERROR",
  Ok = "OK",
  StateUnknown = "STATE_UNKNOWN",
  Transition = "TRANSITION",
}

export type KubernetesDeploymentSyncEntityReply = {
  __typename?: "KubernetesDeploymentSyncEntityReply";
  /** event */
  event?: Maybe<KubernetesDeploymentEntityEvent>;
};

export type KubernetesDeploymentSyncEntityRequest = {
  /** entityId */
  entityId?: Maybe<Scalars["String"]>;
};

export type KubernetesLabelSelector = {
  __typename?: "KubernetesLabelSelector";
  /** matchLabels */
  matchLabels?: Maybe<Array<KubernetesLabelSelectorMatchLabelsMap>>;
};

export type KubernetesLabelSelectorMatchLabelsMap = {
  __typename?: "KubernetesLabelSelectorMatchLabelsMap";
  key?: Maybe<Scalars["String"]>;
  value?: Maybe<Scalars["String"]>;
};

export type KubernetesLabelSelectorRequest = {
  /** matchLabels */
  matchLabels?: Maybe<Array<KubernetesLabelSelectorRequestMatchLabelsMap>>;
};

export type KubernetesLabelSelectorRequestMatchLabelsMap = {
  key?: Maybe<Scalars["String"]>;
  value?: Maybe<Scalars["String"]>;
};

export type KubernetesMetaData = {
  __typename?: "KubernetesMetaData";
  /** labels */
  labels?: Maybe<Array<KubernetesMetaDataLabelsMap>>;
  /** name */
  name?: Maybe<Scalars["String"]>;
};

export type KubernetesMetaDataLabelsMap = {
  __typename?: "KubernetesMetaDataLabelsMap";
  key?: Maybe<Scalars["String"]>;
  value?: Maybe<Scalars["String"]>;
};

export type KubernetesMetaDataRequest = {
  /** labels */
  labels?: Maybe<Array<KubernetesMetaDataRequestLabelsMap>>;
  /** name */
  name?: Maybe<Scalars["String"]>;
};

export type KubernetesMetaDataRequestLabelsMap = {
  key?: Maybe<Scalars["String"]>;
  value?: Maybe<Scalars["String"]>;
};

export type KubernetesPodSpec = {
  __typename?: "KubernetesPodSpec";
  /** containers */
  containers?: Maybe<Array<KubernetesContainer>>;
};

export type KubernetesPodSpecRequest = {
  /** containers */
  containers?: Maybe<Array<KubernetesContainerRequest>>;
};

export type KubernetesPodTemplateSpec = {
  __typename?: "KubernetesPodTemplateSpec";
  /** metadata */
  metadata?: Maybe<KubernetesMetaData>;
  /** spec */
  spec?: Maybe<KubernetesPodSpec>;
};

export type KubernetesPodTemplateSpecRequest = {
  /** metadata */
  metadata?: Maybe<KubernetesMetaDataRequest>;
  /** spec */
  spec?: Maybe<KubernetesPodSpecRequest>;
};

export type ListIntegrationInstancesReply = {
  __typename?: "ListIntegrationInstancesReply";
  /** items */
  items?: Maybe<Array<IntegrationInstance>>;
  /** nextPageToken */
  nextPageToken?: Maybe<Scalars["String"]>;
  /** totalCount */
  totalCount?: Maybe<Scalars["Int"]>;
};

export type ListIntegrationInstancesRequest = {
  /** orderBy */
  orderBy?: Maybe<Scalars["String"]>;
  /** orderByDirection */
  orderByDirection?: Maybe<DataOrderByDirection>;
  /** pageSize */
  pageSize?: Maybe<Scalars["Int"]>;
  /** pageToken */
  pageToken?: Maybe<Scalars["String"]>;
  /** query */
  query?: Maybe<DataQuery>;
  /** scopeByTenantId */
  scopeByTenantId?: Maybe<Scalars["String"]>;
};

export type ListIntegrationServicesReply = {
  __typename?: "ListIntegrationServicesReply";
  /** items */
  items?: Maybe<Array<IntegrationService>>;
  /** nextPageToken */
  nextPageToken?: Maybe<Scalars["String"]>;
  /** totalCount */
  totalCount?: Maybe<Scalars["Int"]>;
};

export type ListIntegrationServicesRequest = {
  /** orderBy */
  orderBy?: Maybe<Scalars["String"]>;
  /** orderByDirection */
  orderByDirection?: Maybe<DataOrderByDirection>;
  /** pageSize */
  pageSize?: Maybe<Scalars["Int"]>;
  /** pageToken */
  pageToken?: Maybe<Scalars["String"]>;
  /** query */
  query?: Maybe<DataQuery>;
  /** scopeByTenantId */
  scopeByTenantId?: Maybe<Scalars["String"]>;
};

export type ListIntegrationsReply = {
  __typename?: "ListIntegrationsReply";
  /** items */
  items?: Maybe<Array<Integration>>;
  /** nextPageToken */
  nextPageToken?: Maybe<Scalars["String"]>;
  /** totalCount */
  totalCount?: Maybe<Scalars["Int"]>;
};

export type ListIntegrationsRequest = {
  /** orderBy */
  orderBy?: Maybe<Scalars["String"]>;
  /** orderByDirection */
  orderByDirection?: Maybe<DataOrderByDirection>;
  /** pageSize */
  pageSize?: Maybe<Scalars["Int"]>;
  /** pageToken */
  pageToken?: Maybe<Scalars["String"]>;
  /** query */
  query?: Maybe<DataQuery>;
  /** scopeByTenantId */
  scopeByTenantId?: Maybe<Scalars["String"]>;
};

export type ListOrganizationsReply = {
  __typename?: "ListOrganizationsReply";
  /** items */
  items?: Maybe<Array<Organization>>;
  /** nextPageToken */
  nextPageToken?: Maybe<Scalars["String"]>;
  /** totalCount */
  totalCount?: Maybe<Scalars["Int"]>;
};

export type ListOrganizationsRequest = {
  /** orderBy */
  orderBy?: Maybe<Scalars["String"]>;
  /** orderByDirection */
  orderByDirection?: Maybe<DataOrderByDirection>;
  /** pageSize */
  pageSize?: Maybe<Scalars["Int"]>;
  /** pageToken */
  pageToken?: Maybe<Scalars["String"]>;
  /** query */
  query?: Maybe<DataQuery>;
  /** scopeByTenantId */
  scopeByTenantId?: Maybe<Scalars["String"]>;
};

export type ListUsersReply = {
  __typename?: "ListUsersReply";
  /** items */
  items?: Maybe<Array<User>>;
  /** nextPageToken */
  nextPageToken?: Maybe<Scalars["String"]>;
  /** totalCount */
  totalCount?: Maybe<Scalars["Int"]>;
};

export type ListUsersRequest = {
  /** orderBy */
  orderBy?: Maybe<Scalars["String"]>;
  /** orderByDirection */
  orderByDirection?: Maybe<DataOrderByDirection>;
  /** pageSize */
  pageSize?: Maybe<Scalars["Int"]>;
  /** pageToken */
  pageToken?: Maybe<Scalars["String"]>;
  /** query */
  query?: Maybe<DataQuery>;
  /** scopeByTenantId */
  scopeByTenantId?: Maybe<Scalars["String"]>;
};

export type ListWorkspacesReply = {
  __typename?: "ListWorkspacesReply";
  /** items */
  items?: Maybe<Array<Workspace>>;
  /** nextPageToken */
  nextPageToken?: Maybe<Scalars["String"]>;
  /** totalCount */
  totalCount?: Maybe<Scalars["Int"]>;
};

export type ListWorkspacesRequest = {
  /** orderBy */
  orderBy?: Maybe<Scalars["String"]>;
  /** orderByDirection */
  orderByDirection?: Maybe<DataOrderByDirection>;
  /** pageSize */
  pageSize?: Maybe<Scalars["Int"]>;
  /** pageToken */
  pageToken?: Maybe<Scalars["String"]>;
  /** query */
  query?: Maybe<DataQuery>;
  /** scopeByTenantId */
  scopeByTenantId?: Maybe<Scalars["String"]>;
};

export type LoginReply = {
  __typename?: "LoginReply";
  billingAccountId?: Maybe<Scalars["String"]>;
  jwt?: Maybe<Scalars["String"]>;
  userId?: Maybe<Scalars["String"]>;
};

export type LoginRequest = {
  billingAccountShortName: Scalars["String"];
  email: Scalars["String"];
  password: Scalars["String"];
};

export type Mutation = {
  __typename?: "Mutation";
  awsEksClusterRuntimeAddNodegroup?: Maybe<
    AwsEksClusterRuntimeAddNodegroupReply
  >;
  awsEksClusterRuntimeCreateEntity?: Maybe<
    AwsEksClusterRuntimeCreateEntityReply
  >;
  awsEksClusterRuntimeSyncEntity?: Maybe<AwsEksClusterRuntimeSyncEntityReply>;
  createAccount?: Maybe<CreateAccountReply>;
  createIntegrationInstance?: Maybe<CreateIntegrationInstanceReply>;
  kubernetesDeploymentCreateEntity?: Maybe<
    KubernetesDeploymentCreateEntityReply
  >;
  kubernetesDeploymentEditPropObject?: Maybe<
    KubernetesDeploymentEditPropObjectReply
  >;
  kubernetesDeploymentEditPropObjectYaml?: Maybe<
    KubernetesDeploymentEditPropObjectYamlReply
  >;
  kubernetesDeploymentSyncEntity?: Maybe<KubernetesDeploymentSyncEntityReply>;
  sshKeyCreateEntity?: Maybe<SshKeyCreateEntityReply>;
  sshKeySyncEntity?: Maybe<SshKeySyncEntityReply>;
};

export type MutationAwsEksClusterRuntimeAddNodegroupArgs = {
  input?: Maybe<AwsEksClusterRuntimeAddNodegroupRequest>;
};

export type MutationAwsEksClusterRuntimeCreateEntityArgs = {
  input?: Maybe<AwsEksClusterRuntimeCreateEntityRequest>;
};

export type MutationAwsEksClusterRuntimeSyncEntityArgs = {
  input?: Maybe<AwsEksClusterRuntimeSyncEntityRequest>;
};

export type MutationCreateAccountArgs = {
  input?: Maybe<CreateAccountRequest>;
};

export type MutationCreateIntegrationInstanceArgs = {
  input?: Maybe<CreateIntegrationInstanceRequest>;
};

export type MutationKubernetesDeploymentCreateEntityArgs = {
  input?: Maybe<KubernetesDeploymentCreateEntityRequest>;
};

export type MutationKubernetesDeploymentEditPropObjectArgs = {
  input?: Maybe<KubernetesDeploymentEditPropObjectRequest>;
};

export type MutationKubernetesDeploymentEditPropObjectYamlArgs = {
  input?: Maybe<KubernetesDeploymentEditPropObjectYamlRequest>;
};

export type MutationKubernetesDeploymentSyncEntityArgs = {
  input?: Maybe<KubernetesDeploymentSyncEntityRequest>;
};

export type MutationSshKeyCreateEntityArgs = {
  input?: Maybe<SshKeyCreateEntityRequest>;
};

export type MutationSshKeySyncEntityArgs = {
  input?: Maybe<SshKeySyncEntityRequest>;
};

export type Organization = {
  __typename?: "Organization";
  /** billingAccountId */
  billingAccount?: Maybe<BillingAccount>;
  /** billingAccountId */
  billingAccountId?: Maybe<Scalars["String"]>;
  /** id */
  enabledIntegrationInstances?: Maybe<ListIntegrationInstancesReply>;
  /** id */
  id?: Maybe<Scalars["ID"]>;
  /** name */
  name?: Maybe<Scalars["String"]>;
  /** naturalKey */
  naturalKey?: Maybe<Scalars["String"]>;
  /** tenantIds */
  tenantIds?: Maybe<Array<Scalars["String"]>>;
  /** typeName */
  typeName?: Maybe<Scalars["String"]>;
  /** id */
  workspaces?: Maybe<ListWorkspacesReply>;
};

export type OrganizationEnabledIntegrationInstancesArgs = {
  input?: Maybe<ListIntegrationInstancesRequest>;
};

export type OrganizationWorkspacesArgs = {
  input?: Maybe<ListWorkspacesRequest>;
};

export type Query = {
  __typename?: "Query";
  awsEksClusterRuntimeGetComponent?: Maybe<
    AwsEksClusterRuntimeGetComponentReply
  >;
  awsEksClusterRuntimeGetEntity?: Maybe<AwsEksClusterRuntimeGetEntityReply>;
  awsEksClusterRuntimeListComponents?: Maybe<
    AwsEksClusterRuntimeListComponentsReply
  >;
  awsEksClusterRuntimeListEntities?: Maybe<
    AwsEksClusterRuntimeListEntitiesReply
  >;
  awsEksClusterRuntimeListEntityEvents?: Maybe<
    AwsEksClusterRuntimeListEntityEventsReply
  >;
  awsEksClusterRuntimePickComponent?: Maybe<
    AwsEksClusterRuntimePickComponentReply
  >;
  getBillingAccount?: Maybe<GetBillingAccountReply>;
  getUser?: Maybe<GetUserReply>;
  kubernetesDeploymentGetComponent?: Maybe<
    KubernetesDeploymentGetComponentReply
  >;
  kubernetesDeploymentGetEntity?: Maybe<KubernetesDeploymentGetEntityReply>;
  kubernetesDeploymentListComponents?: Maybe<
    KubernetesDeploymentListComponentsReply
  >;
  kubernetesDeploymentListEntities?: Maybe<
    KubernetesDeploymentListEntitiesReply
  >;
  kubernetesDeploymentListEntityEvents?: Maybe<
    KubernetesDeploymentListEntityEventsReply
  >;
  kubernetesDeploymentPickComponent?: Maybe<
    KubernetesDeploymentPickComponentReply
  >;
  listIntegrationInstances?: Maybe<ListIntegrationInstancesReply>;
  listIntegrations?: Maybe<ListIntegrationsReply>;
  listOrganizations?: Maybe<ListOrganizationsReply>;
  listUsers?: Maybe<ListUsersReply>;
  listWorkspaces?: Maybe<ListWorkspacesReply>;
  login?: Maybe<LoginReply>;
  sshKeyGetComponent?: Maybe<SshKeyGetComponentReply>;
  sshKeyGetEntity?: Maybe<SshKeyGetEntityReply>;
  sshKeyListComponents?: Maybe<SshKeyListComponentsReply>;
  sshKeyListEntities?: Maybe<SshKeyListEntitiesReply>;
  sshKeyListEntityEvents?: Maybe<SshKeyListEntityEventsReply>;
  sshKeyPickComponent?: Maybe<SshKeyPickComponentReply>;
};

export type QueryAwsEksClusterRuntimeGetComponentArgs = {
  input?: Maybe<AwsEksClusterRuntimeGetComponentRequest>;
};

export type QueryAwsEksClusterRuntimeGetEntityArgs = {
  input?: Maybe<AwsEksClusterRuntimeGetEntityRequest>;
};

export type QueryAwsEksClusterRuntimeListComponentsArgs = {
  input?: Maybe<AwsEksClusterRuntimeListComponentsRequest>;
};

export type QueryAwsEksClusterRuntimeListEntitiesArgs = {
  input?: Maybe<AwsEksClusterRuntimeListEntitiesRequest>;
};

export type QueryAwsEksClusterRuntimeListEntityEventsArgs = {
  input?: Maybe<AwsEksClusterRuntimeListEntityEventsRequest>;
};

export type QueryAwsEksClusterRuntimePickComponentArgs = {
  input?: Maybe<AwsEksClusterRuntimePickComponentRequest>;
};

export type QueryGetBillingAccountArgs = {
  input?: Maybe<GetBillingAccountRequest>;
};

export type QueryGetUserArgs = {
  input?: Maybe<GetUserRequest>;
};

export type QueryKubernetesDeploymentGetComponentArgs = {
  input?: Maybe<KubernetesDeploymentGetComponentRequest>;
};

export type QueryKubernetesDeploymentGetEntityArgs = {
  input?: Maybe<KubernetesDeploymentGetEntityRequest>;
};

export type QueryKubernetesDeploymentListComponentsArgs = {
  input?: Maybe<KubernetesDeploymentListComponentsRequest>;
};

export type QueryKubernetesDeploymentListEntitiesArgs = {
  input?: Maybe<KubernetesDeploymentListEntitiesRequest>;
};

export type QueryKubernetesDeploymentListEntityEventsArgs = {
  input?: Maybe<KubernetesDeploymentListEntityEventsRequest>;
};

export type QueryKubernetesDeploymentPickComponentArgs = {
  input?: Maybe<KubernetesDeploymentPickComponentRequest>;
};

export type QueryListIntegrationInstancesArgs = {
  input?: Maybe<ListIntegrationInstancesRequest>;
};

export type QueryListIntegrationsArgs = {
  input?: Maybe<ListIntegrationsRequest>;
};

export type QueryListOrganizationsArgs = {
  input?: Maybe<ListOrganizationsRequest>;
};

export type QueryListUsersArgs = {
  input?: Maybe<ListUsersRequest>;
};

export type QueryListWorkspacesArgs = {
  input?: Maybe<ListWorkspacesRequest>;
};

export type QueryLoginArgs = {
  input?: Maybe<LoginRequest>;
};

export type QuerySshKeyGetComponentArgs = {
  input?: Maybe<SshKeyGetComponentRequest>;
};

export type QuerySshKeyGetEntityArgs = {
  input?: Maybe<SshKeyGetEntityRequest>;
};

export type QuerySshKeyListComponentsArgs = {
  input?: Maybe<SshKeyListComponentsRequest>;
};

export type QuerySshKeyListEntitiesArgs = {
  input?: Maybe<SshKeyListEntitiesRequest>;
};

export type QuerySshKeyListEntityEventsArgs = {
  input?: Maybe<SshKeyListEntityEventsRequest>;
};

export type QuerySshKeyPickComponentArgs = {
  input?: Maybe<SshKeyPickComponentRequest>;
};

export type SshKeyComponent = {
  __typename?: "SshKeyComponent";
  /** bits */
  bits?: Maybe<Scalars["Int"]>;
  /** description */
  description?: Maybe<Scalars["String"]>;
  /** displayName */
  displayName?: Maybe<Scalars["String"]>;
  /** displayTypeName */
  displayTypeName?: Maybe<Scalars["String"]>;
  /** id */
  id?: Maybe<Scalars["ID"]>;
  /** integrationId */
  integration?: Maybe<Integration>;
  /** integrationId */
  integrationId?: Maybe<Scalars["String"]>;
  /** integrationServiceId */
  integrationService?: Maybe<IntegrationService>;
  /** integrationServiceId */
  integrationServiceId?: Maybe<Scalars["String"]>;
  /** keyFormat */
  keyFormat?: Maybe<SshKeyKeyFormat>;
  /** keyType */
  keyType?: Maybe<SshKeyKeyType>;
  /** name */
  name?: Maybe<Scalars["String"]>;
  /** naturalKey */
  naturalKey?: Maybe<Scalars["String"]>;
  /** tenantIds */
  tenantIds?: Maybe<Array<Scalars["String"]>>;
  /** typeName */
  typeName?: Maybe<Scalars["String"]>;
  /** version */
  version?: Maybe<Scalars["Int"]>;
};

export type SshKeyConstraints = {
  __typename?: "SshKeyConstraints";
  /** bits */
  bits?: Maybe<Scalars["Int"]>;
  /** displayName */
  displayName?: Maybe<Scalars["String"]>;
  /** integrationId */
  integrationId?: Maybe<Scalars["String"]>;
  /** integrationServiceId */
  integrationServiceId?: Maybe<Scalars["String"]>;
  /** keyFormat */
  keyFormat?: Maybe<SshKeyKeyFormatRequest>;
  /** keyType */
  keyType?: Maybe<SshKeyKeyTypeRequest>;
  /** name */
  name?: Maybe<Scalars["String"]>;
};

export type SshKeyCreateEntityReply = {
  __typename?: "SshKeyCreateEntityReply";
  /** entity */
  entity?: Maybe<SshKeyEntity>;
  /** event */
  event?: Maybe<SshKeyEntityEvent>;
};

export type SshKeyCreateEntityRequest = {
  /** constraints */
  constraints?: Maybe<SshKeyPickComponentRequest>;
  /** description */
  description?: Maybe<Scalars["String"]>;
  /** displayName */
  displayName?: Maybe<Scalars["String"]>;
  /** name */
  name?: Maybe<Scalars["String"]>;
  /** workspaceId */
  workspaceId?: Maybe<Scalars["String"]>;
};

export type SshKeyEntity = {
  __typename?: "SshKeyEntity";
  /** billingAccountId */
  billingAccount?: Maybe<BillingAccount>;
  /** billingAccountId */
  billingAccountId?: Maybe<Scalars["String"]>;
  /** bits */
  bits?: Maybe<Scalars["Int"]>;
  /** bubbleBabble */
  bubbleBabble?: Maybe<Scalars["String"]>;
  /** comment */
  comment?: Maybe<Scalars["String"]>;
  /** componentId */
  componentId?: Maybe<Scalars["String"]>;
  /** constraints */
  constraints?: Maybe<SshKeyConstraints>;
  /** description */
  description?: Maybe<Scalars["String"]>;
  /** displayName */
  displayName?: Maybe<Scalars["String"]>;
  /** id */
  entityEvents?: Maybe<SshKeyListEntityEventsReply>;
  /** fingerprint */
  fingerprint?: Maybe<Scalars["String"]>;
  /** id */
  id?: Maybe<Scalars["ID"]>;
  /** implicitConstraints */
  implicitConstraints?: Maybe<Array<SshKeyImplicitConstraint>>;
  /** integrationId */
  integration?: Maybe<Integration>;
  /** integrationId */
  integrationId?: Maybe<Scalars["String"]>;
  /** integrationServiceId */
  integrationService?: Maybe<IntegrationService>;
  /** integrationServiceId */
  integrationServiceId?: Maybe<Scalars["String"]>;
  /** keyFormat */
  keyFormat?: Maybe<SshKeyKeyFormat>;
  /** keyType */
  keyType?: Maybe<SshKeyKeyType>;
  /** linkedEntityIds */
  linkedEntityIds?: Maybe<Array<Scalars["String"]>>;
  /** name */
  name?: Maybe<Scalars["String"]>;
  /** naturalKey */
  naturalKey?: Maybe<Scalars["String"]>;
  /** organizationId */
  organization?: Maybe<Organization>;
  /** organizationId */
  organizationId?: Maybe<Scalars["String"]>;
  /** privateKey */
  privateKey?: Maybe<Scalars["String"]>;
  /** publicKey */
  publicKey?: Maybe<Scalars["String"]>;
  /** randomArt */
  randomArt?: Maybe<Scalars["String"]>;
  /** state */
  state?: Maybe<SshKeyState>;
  /** tenantIds */
  tenantIds?: Maybe<Array<Scalars["String"]>>;
  /** typeName */
  typeName?: Maybe<Scalars["String"]>;
  /** workspaceId */
  workspace?: Maybe<Workspace>;
  /** workspaceId */
  workspaceId?: Maybe<Scalars["String"]>;
};

export type SshKeyEntityEntityEventsArgs = {
  input?: Maybe<SshKeyListEntityEventsRequest>;
};

export type SshKeyEntityEvent = EntityEvent & {
  __typename?: "SshKeyEntityEvent";
  /** actionName */
  actionName?: Maybe<Scalars["String"]>;
  /** billingAccountId */
  billingAccountId?: Maybe<Scalars["String"]>;
  /** componentId */
  componentId?: Maybe<Scalars["String"]>;
  /** createTime */
  createTime?: Maybe<Scalars["String"]>;
  /** entityId */
  entity?: Maybe<SshKeyEntity>;
  /** entityId */
  entityId?: Maybe<Scalars["String"]>;
  /** errorLines */
  errorLines?: Maybe<Array<Scalars["String"]>>;
  /** errorMessage */
  errorMessage?: Maybe<Scalars["String"]>;
  /** finalized */
  finalized?: Maybe<Scalars["Boolean"]>;
  /** finalTime */
  finalTime?: Maybe<Scalars["String"]>;
  /** id */
  id?: Maybe<Scalars["ID"]>;
  /** inputEntity */
  inputEntity?: Maybe<SshKeyEntity>;
  /** integrationId */
  integrationId?: Maybe<Scalars["String"]>;
  /** integrationServiceId */
  integrationServiceId?: Maybe<Scalars["String"]>;
  /** naturalKey */
  naturalKey?: Maybe<Scalars["String"]>;
  /** nextState */
  nextState?: Maybe<SshKeyNextState>;
  /** organizationId */
  organizationId?: Maybe<Scalars["String"]>;
  /** outputEntity */
  outputEntity?: Maybe<SshKeyEntity>;
  /** outputLines */
  outputLines?: Maybe<Array<Scalars["String"]>>;
  /** previousEntity */
  previousEntity?: Maybe<SshKeyEntity>;
  /** success */
  success?: Maybe<Scalars["Boolean"]>;
  /** tenantIds */
  tenantIds?: Maybe<Array<Scalars["String"]>>;
  /** typeName */
  typeName?: Maybe<Scalars["String"]>;
  /** updatedTime */
  updatedTime?: Maybe<Scalars["String"]>;
  /** userId */
  user?: Maybe<User>;
  /** userId */
  userId?: Maybe<Scalars["String"]>;
  /** workspaceId */
  workspaceId?: Maybe<Scalars["String"]>;
};

export type SshKeyGetComponentReply = {
  __typename?: "SshKeyGetComponentReply";
  /** component */
  component?: Maybe<SshKeyComponent>;
};

export type SshKeyGetComponentRequest = {
  /** componentId */
  componentId?: Maybe<Scalars["String"]>;
};

export type SshKeyGetEntityReply = {
  __typename?: "SshKeyGetEntityReply";
  /** entity */
  entity?: Maybe<SshKeyEntity>;
};

export type SshKeyGetEntityRequest = {
  /** entityId */
  entityId?: Maybe<Scalars["String"]>;
};

export type SshKeyImplicitConstraint = {
  __typename?: "SshKeyImplicitConstraint";
  /** field */
  field?: Maybe<Scalars["String"]>;
  /** value */
  value?: Maybe<Scalars["String"]>;
};

export enum SshKeyKeyFormat {
  Pem = "PEM",
  Pkcs8 = "PKCS8",
  Rfc4716 = "RFC4716",
}

export enum SshKeyKeyFormatRequest {
  Nokeyformat = "NOKEYFORMAT",
  Pem = "PEM",
  Pkcs8 = "PKCS8",
  Rfc4716 = "RFC4716",
}

export enum SshKeyKeyType {
  Dsa = "DSA",
  Ecdsa = "ECDSA",
  Ed25519 = "ED25519",
  Rsa = "RSA",
}

export enum SshKeyKeyTypeRequest {
  Dsa = "DSA",
  Ecdsa = "ECDSA",
  Ed25519 = "ED25519",
  Nokeytype = "NOKEYTYPE",
  Rsa = "RSA",
}

export type SshKeyListComponentsReply = {
  __typename?: "SshKeyListComponentsReply";
  /** items */
  items?: Maybe<Array<SshKeyComponent>>;
  /** nextPageToken */
  nextPageToken?: Maybe<Scalars["String"]>;
  /** totalCount */
  totalCount?: Maybe<Scalars["Int"]>;
};

export type SshKeyListComponentsRequest = {
  /** orderBy */
  orderBy?: Maybe<Scalars["String"]>;
  /** orderByDirection */
  orderByDirection?: Maybe<DataOrderByDirection>;
  /** pageSize */
  pageSize?: Maybe<Scalars["Int"]>;
  /** pageToken */
  pageToken?: Maybe<Scalars["String"]>;
  /** query */
  query?: Maybe<DataQuery>;
  /** scopeByTenantId */
  scopeByTenantId?: Maybe<Scalars["String"]>;
};

export type SshKeyListEntitiesReply = {
  __typename?: "SshKeyListEntitiesReply";
  /** items */
  items?: Maybe<Array<SshKeyEntity>>;
  /** nextPageToken */
  nextPageToken?: Maybe<Scalars["String"]>;
  /** totalCount */
  totalCount?: Maybe<Scalars["Int"]>;
};

export type SshKeyListEntitiesRequest = {
  /** orderBy */
  orderBy?: Maybe<Scalars["String"]>;
  /** orderByDirection */
  orderByDirection?: Maybe<DataOrderByDirection>;
  /** pageSize */
  pageSize?: Maybe<Scalars["Int"]>;
  /** pageToken */
  pageToken?: Maybe<Scalars["String"]>;
  /** query */
  query?: Maybe<DataQuery>;
  /** scopeByTenantId */
  scopeByTenantId?: Maybe<Scalars["String"]>;
};

export type SshKeyListEntityEventsReply = {
  __typename?: "SshKeyListEntityEventsReply";
  /** items */
  items?: Maybe<Array<SshKeyEntityEvent>>;
  /** nextPageToken */
  nextPageToken?: Maybe<Scalars["String"]>;
  /** totalCount */
  totalCount?: Maybe<Scalars["Int"]>;
};

export type SshKeyListEntityEventsRequest = {
  /** orderBy */
  orderBy?: Maybe<Scalars["String"]>;
  /** orderByDirection */
  orderByDirection?: Maybe<DataOrderByDirection>;
  /** pageSize */
  pageSize?: Maybe<Scalars["Int"]>;
  /** pageToken */
  pageToken?: Maybe<Scalars["String"]>;
  /** query */
  query?: Maybe<DataQuery>;
  /** scopeByTenantId */
  scopeByTenantId?: Maybe<Scalars["String"]>;
};

export enum SshKeyNextState {
  Error = "ERROR",
  None = "NONE",
  Ok = "OK",
  Uninitialized = "UNINITIALIZED",
}

export type SshKeyPickComponentReply = {
  __typename?: "SshKeyPickComponentReply";
  /** component */
  component?: Maybe<SshKeyComponent>;
  /** implicitConstraints */
  implicitConstraints?: Maybe<Array<SshKeyImplicitConstraint>>;
};

export type SshKeyPickComponentRequest = {
  /** bits */
  bits?: Maybe<Scalars["Int"]>;
  /** displayName */
  displayName?: Maybe<Scalars["String"]>;
  /** integrationId */
  integrationId?: Maybe<Scalars["String"]>;
  /** integrationServiceId */
  integrationServiceId?: Maybe<Scalars["String"]>;
  /** keyFormat */
  keyFormat?: Maybe<SshKeyKeyFormatRequest>;
  /** keyType */
  keyType?: Maybe<SshKeyKeyTypeRequest>;
  /** name */
  name?: Maybe<Scalars["String"]>;
};

export enum SshKeyState {
  Error = "ERROR",
  Ok = "OK",
  Transition = "TRANSITION",
  Uninitialized = "UNINITIALIZED",
}

export type SshKeySyncEntityReply = {
  __typename?: "SshKeySyncEntityReply";
  /** event */
  event?: Maybe<SshKeyEntityEvent>;
};

export type SshKeySyncEntityRequest = {
  /** entityId */
  entityId?: Maybe<Scalars["String"]>;
};

export type StreamEntityEventsRequest = {
  scopeByTenantId: Scalars["String"];
};

export type Subscription = {
  __typename?: "Subscription";
  streamEntityEvents?: Maybe<EntityEvent>;
};

export type SubscriptionStreamEntityEventsArgs = {
  input: StreamEntityEventsRequest;
};

export type User = {
  __typename?: "User";
  /** billingAccountId */
  billingAccount?: Maybe<BillingAccount>;
  /** billingAccountId */
  billingAccountId?: Maybe<Scalars["String"]>;
  /** capabilities */
  capabilities?: Maybe<Array<Capability>>;
  /** displayName */
  displayName?: Maybe<Scalars["String"]>;
  /** domain */
  domain?: Maybe<Scalars["String"]>;
  /** email */
  email?: Maybe<Scalars["String"]>;
  /** familyName */
  familyName?: Maybe<Scalars["String"]>;
  /** givenName */
  givenName?: Maybe<Scalars["String"]>;
  /** id */
  id?: Maybe<Scalars["ID"]>;
  /** naturalKey */
  naturalKey?: Maybe<Scalars["String"]>;
  /** picture */
  picture?: Maybe<Scalars["String"]>;
  /** tenantIds */
  tenantIds?: Maybe<Array<Scalars["String"]>>;
  /** typeName */
  typeName?: Maybe<Scalars["String"]>;
};

export type Workspace = {
  __typename?: "Workspace";
  /** billingAccountId */
  billingAccount?: Maybe<BillingAccount>;
  /** billingAccountId */
  billingAccountId?: Maybe<Scalars["String"]>;
  /** id */
  enabledIntegrationInstances?: Maybe<ListIntegrationInstancesReply>;
  /** id */
  id?: Maybe<Scalars["ID"]>;
  /** name */
  name?: Maybe<Scalars["String"]>;
  /** naturalKey */
  naturalKey?: Maybe<Scalars["String"]>;
  /** organizationId */
  organization?: Maybe<Organization>;
  /** organizationId */
  organizationId?: Maybe<Scalars["String"]>;
  /** tenantIds */
  tenantIds?: Maybe<Array<Scalars["String"]>>;
  /** typeName */
  typeName?: Maybe<Scalars["String"]>;
};

export type WorkspaceEnabledIntegrationInstancesArgs = {
  input?: Maybe<ListIntegrationInstancesRequest>;
};

export type CreateAccountMutationVariables = {
  billingAccountShortName: Scalars["String"];
  billingAccountDisplayName: Scalars["String"];
  userDisplayName: Scalars["String"];
  userGivenName: Scalars["String"];
  userFamilyName: Scalars["String"];
  userEmail: Scalars["String"];
  userPassword: Scalars["String"];
};

export type CreateAccountMutation = { __typename?: "Mutation" } & {
  createAccount: Maybe<
    { __typename?: "CreateAccountReply" } & {
      user: Maybe<{ __typename?: "User" } & Pick<User, "id">>;
      billingAccount: Maybe<
        { __typename?: "BillingAccount" } & Pick<BillingAccount, "id">
      >;
    }
  >;
};

export type CreateEntityMutationVariables = {
  name: Scalars["String"];
  displayName: Scalars["String"];
  description: Scalars["String"];
  workspaceId: Scalars["String"];
  constraints?: Maybe<SshKeyPickComponentRequest>;
};

export type CreateEntityMutation = { __typename?: "Mutation" } & {
  sshKeyCreateEntity: Maybe<
    { __typename?: "SshKeyCreateEntityReply" } & {
      entity: Maybe<
        { __typename?: "SshKeyEntity" } & Pick<
          SshKeyEntity,
          | "id"
          | "privateKey"
          | "publicKey"
          | "bubbleBabble"
          | "randomArt"
          | "state"
        >
      >;
      event: Maybe<
        { __typename?: "SshKeyEntityEvent" } & Pick<
          SshKeyEntityEvent,
          | "id"
          | "actionName"
          | "entityId"
          | "typeName"
          | "createTime"
          | "updatedTime"
          | "finalTime"
          | "finalized"
          | "outputLines"
          | "errorLines"
          | "success"
        > & {
            inputEntity: Maybe<
              { __typename?: "SshKeyEntity" } & Pick<
                SshKeyEntity,
                "id" | "keyType" | "keyFormat" | "bits" | "state"
              >
            >;
            outputEntity: Maybe<
              { __typename?: "SshKeyEntity" } & Pick<
                SshKeyEntity,
                | "id"
                | "keyType"
                | "keyFormat"
                | "bits"
                | "fingerprint"
                | "publicKey"
                | "privateKey"
                | "state"
              >
            >;
          }
      >;
    }
  >;
};

export type GetProfileQueryVariables = {
  userId: Scalars["String"];
};

export type GetProfileQuery = { __typename?: "Query" } & {
  getUser: Maybe<
    { __typename?: "GetUserReply" } & {
      user: Maybe<
        { __typename?: "User" } & Pick<
          User,
          | "id"
          | "email"
          | "domain"
          | "displayName"
          | "givenName"
          | "familyName"
          | "picture"
        > & {
            billingAccount: Maybe<
              { __typename?: "BillingAccount" } & Pick<
                BillingAccount,
                "id" | "displayName" | "shortName"
              > & {
                  organizations: Maybe<
                    { __typename?: "ListOrganizationsReply" } & Pick<
                      ListOrganizationsReply,
                      "nextPageToken" | "totalCount"
                    > & {
                        items: Maybe<
                          Array<
                            { __typename?: "Organization" } & Pick<
                              Organization,
                              "id" | "name"
                            > & {
                                workspaces: Maybe<
                                  { __typename?: "ListWorkspacesReply" } & Pick<
                                    ListWorkspacesReply,
                                    "nextPageToken" | "totalCount"
                                  > & {
                                      items: Maybe<
                                        Array<
                                          { __typename?: "Workspace" } & Pick<
                                            Workspace,
                                            "id" | "name"
                                          >
                                        >
                                      >;
                                    }
                                >;
                              }
                          >
                        >;
                      }
                  >;
                }
            >;
          }
      >;
    }
  >;
};

export type LoginQueryVariables = {
  email: Scalars["String"];
  password: Scalars["String"];
  billingAccountShortName: Scalars["String"];
};

export type LoginQuery = { __typename?: "Query" } & {
  login: Maybe<
    { __typename?: "LoginReply" } & Pick<
      LoginReply,
      "jwt" | "userId" | "billingAccountId"
    >
  >;
};

export type PickComponentQueryVariables = {
  keyType?: Maybe<SshKeyKeyTypeRequest>;
  keyFormat?: Maybe<SshKeyKeyFormatRequest>;
  bits?: Maybe<Scalars["Int"]>;
};

export type PickComponentQuery = { __typename?: "Query" } & {
  sshKeyPickComponent: Maybe<
    { __typename?: "SshKeyPickComponentReply" } & {
      component: Maybe<
        { __typename?: "SshKeyComponent" } & Pick<
          SshKeyComponent,
          "id" | "displayName" | "keyType" | "keyFormat" | "bits"
        >
      >;
      implicitConstraints: Maybe<
        Array<
          { __typename?: "SshKeyImplicitConstraint" } & Pick<
            SshKeyImplicitConstraint,
            "field" | "value"
          >
        >
      >;
    }
  >;
};

export type WorkspaceListQueryVariables = {};

export type WorkspaceListQuery = { __typename?: "Query" } & {
  sshKeyListComponents: Maybe<
    { __typename?: "SshKeyListComponentsReply" } & Pick<
      SshKeyListComponentsReply,
      "totalCount" | "nextPageToken"
    > & {
        items: Maybe<
          Array<
            { __typename?: "SshKeyComponent" } & Pick<
              SshKeyComponent,
              "id" | "naturalKey" | "name" | "displayName"
            >
          >
        >;
      }
  >;
  sshKeyListEntities: Maybe<
    { __typename?: "SshKeyListEntitiesReply" } & Pick<
      SshKeyListEntitiesReply,
      "totalCount" | "nextPageToken"
    > & {
        items: Maybe<
          Array<
            { __typename?: "SshKeyEntity" } & Pick<
              SshKeyEntity,
              "id" | "naturalKey" | "typeName" | "name" | "displayName"
            >
          >
        >;
      }
  >;
};
