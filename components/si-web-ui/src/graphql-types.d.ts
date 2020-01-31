export type Maybe<T> = T | null;
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: string;
  String: string;
  Boolean: boolean;
  Int: number;
  Float: number;
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
  createAccount?: Maybe<CreateAccountReply>;
  createIntegrationInstance?: Maybe<CreateIntegrationInstanceReply>;
  sshKeyCreateEntity?: Maybe<SshKeyCreateEntityReply>;
};

export type MutationCreateAccountArgs = {
  input?: Maybe<CreateAccountRequest>;
};

export type MutationCreateIntegrationInstanceArgs = {
  input?: Maybe<CreateIntegrationInstanceRequest>;
};

export type MutationSshKeyCreateEntityArgs = {
  input?: Maybe<SshKeyCreateEntityRequest>;
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
  getBillingAccount?: Maybe<GetBillingAccountReply>;
  getUser?: Maybe<GetUserReply>;
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
  sshKeyPickComponent?: Maybe<SshKeyPickComponentReply>;
};

export type QueryGetBillingAccountArgs = {
  input?: Maybe<GetBillingAccountRequest>;
};

export type QueryGetUserArgs = {
  input?: Maybe<GetUserRequest>;
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

export type SshKeyEntityEvent = {
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
  /** success */
  success?: Maybe<Scalars["Boolean"]>;
  /** tenantIds */
  tenantIds?: Maybe<Array<Scalars["String"]>>;
  /** typeName */
  typeName?: Maybe<Scalars["String"]>;
  /** updatedTime */
  updatedTime?: Maybe<Scalars["String"]>;
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
  Uninitialized = "UNINITIALIZED",
}

export type StreamEntityEventsRequest = {
  workspaceId: Scalars["String"];
};

export type Subscription = {
  __typename?: "Subscription";
  streamEntityEvents?: Maybe<SshKeyEntityEvent>;
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
