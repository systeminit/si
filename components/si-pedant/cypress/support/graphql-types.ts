export type Maybe<T> = T | null;
export type Exact<T extends { [key: string]: any }> = { [K in keyof T]: T[K] };
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: string;
  String: string;
  Boolean: boolean;
  Int: number;
  Float: number;
};

export type ApplicationComponent = {
  __typename?: 'ApplicationComponent';
  /** Component Constraints */
  constraints?: Maybe<ApplicationComponentConstraints>;
  /** Component Description */
  description?: Maybe<Scalars['String']>;
  /** A System Initiative Application Component Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** A System Initiative Application Component ID */
  id?: Maybe<Scalars['ID']>;
  /** A System Initiative Application Component Name */
  name?: Maybe<Scalars['String']>;
  /** SI Properties */
  siProperties?: Maybe<ComponentSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
};

export type ApplicationComponentConstraints = {
  __typename?: 'ApplicationComponentConstraints';
  /** Component Display Name */
  componentDisplayName?: Maybe<Scalars['String']>;
  /** Component Name */
  componentName?: Maybe<Scalars['String']>;
};

export type ApplicationComponentConstraintsRequest = {
  /** Component Display Name */
  componentDisplayName?: Maybe<Scalars['String']>;
  /** Component Name */
  componentName?: Maybe<Scalars['String']>;
};

/** Get a A System Initiative Application Component Reply */
export type ApplicationComponentGetReply = {
  __typename?: 'ApplicationComponentGetReply';
  /** A System Initiative Application Component Item */
  item?: Maybe<ApplicationComponent>;
};

/** Get a A System Initiative Application Component Request */
export type ApplicationComponentGetRequest = {
  /** A System Initiative Application Component ID */
  id?: Maybe<Scalars['ID']>;
};

/** List A System Initiative Application Component Reply */
export type ApplicationComponentListReply = {
  __typename?: 'ApplicationComponentListReply';
  /** Items */
  items?: Maybe<Array<ApplicationComponent>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars['String']>;
  /** Total Count */
  totalCount?: Maybe<Scalars['String']>;
};

/** List A System Initiative Application Component Request */
export type ApplicationComponentListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars['String']>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars['String']>;
  /** Page Token */
  pageToken?: Maybe<Scalars['String']>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars['String']>;
};

/** Pick Component Reply */
export type ApplicationComponentPickReply = {
  __typename?: 'ApplicationComponentPickReply';
  /** Chosen Component */
  component?: Maybe<ApplicationComponent>;
  /** Implicit Constraints */
  implicitConstraints?: Maybe<ApplicationComponentConstraints>;
};

/** Pick Component Request */
export type ApplicationComponentPickRequest = {
  /** Constraints */
  constraints?: Maybe<ApplicationComponentConstraintsRequest>;
};

export type ApplicationEntity = {
  __typename?: 'ApplicationEntity';
  /** Constraints */
  constraints?: Maybe<ApplicationComponentConstraints>;
  /** Entity Description */
  description?: Maybe<Scalars['String']>;
  /** A System Initiative Application Entity Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** A System Initiative Application Entity ID */
  id?: Maybe<Scalars['ID']>;
  /** Implicit Constraints */
  implicitConstraints?: Maybe<ApplicationComponentConstraints>;
  /** A System Initiative Application Entity Name */
  name?: Maybe<Scalars['String']>;
  /** Properties */
  properties?: Maybe<ApplicationEntityProperties>;
  /** SI Properties */
  siProperties?: Maybe<EntitySiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
};

/** Create Entity Reply */
export type ApplicationEntityCreateReply = {
  __typename?: 'ApplicationEntityCreateReply';
  /** applicationEntity Item */
  item?: Maybe<ApplicationEntity>;
};

/** Create Entity Request */
export type ApplicationEntityCreateRequest = {
  /** Change Set ID */
  changeSetId?: Maybe<Scalars['String']>;
  /** Constraints */
  constraints?: Maybe<ApplicationComponentConstraintsRequest>;
  /** Description */
  description?: Maybe<Scalars['String']>;
  /** Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** Name */
  name?: Maybe<Scalars['String']>;
  /** Properties */
  properties?: Maybe<ApplicationEntityPropertiesRequest>;
  /** Workspace ID */
  workspaceId?: Maybe<Scalars['String']>;
};

/** Delete Entity Reply */
export type ApplicationEntityDeleteReply = {
  __typename?: 'ApplicationEntityDeleteReply';
  /** application Item */
  item?: Maybe<ApplicationEntity>;
};

/** Delete Entity Request */
export type ApplicationEntityDeleteRequest = {
  /** Change Set ID */
  changeSetId?: Maybe<Scalars['String']>;
  /** applicationEntity ID */
  id?: Maybe<Scalars['ID']>;
};

export type ApplicationEntityEvent = {
  __typename?: 'ApplicationEntityEvent';
  /** Action Name */
  actionName?: Maybe<Scalars['String']>;
  /** Creation Time */
  createTime?: Maybe<Scalars['String']>;
  /** Error Lines */
  errorLines?: Maybe<Array<Scalars['String']>>;
  /** Error Message */
  errorMessage?: Maybe<Scalars['String']>;
  /** Finalized */
  finalized?: Maybe<Scalars['Boolean']>;
  /** Final Time */
  finalTime?: Maybe<Scalars['String']>;
  /** A System Initiative Application EntityEvent ID */
  id?: Maybe<Scalars['ID']>;
  /** Input Entity */
  inputEntity?: Maybe<ApplicationEntity>;
  /** Output Entity */
  outputEntity?: Maybe<ApplicationEntity>;
  /** Output Lines */
  outputLines?: Maybe<Array<Scalars['String']>>;
  /** Previous Entity */
  previousEntity?: Maybe<ApplicationEntity>;
  /** SI Properties */
  siProperties?: Maybe<EntityEventSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
  /** success */
  success?: Maybe<Scalars['Boolean']>;
  /** Updated Time */
  updatedTime?: Maybe<Scalars['String']>;
  /** User ID */
  userId?: Maybe<Scalars['String']>;
};

/** List A System Initiative Application EntityEvent Reply */
export type ApplicationEntityEventListReply = {
  __typename?: 'ApplicationEntityEventListReply';
  /** Items */
  items?: Maybe<Array<ApplicationEntityEvent>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars['String']>;
  /** Total Count */
  totalCount?: Maybe<Scalars['String']>;
};

/** List A System Initiative Application EntityEvent Request */
export type ApplicationEntityEventListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars['String']>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars['String']>;
  /** Page Token */
  pageToken?: Maybe<Scalars['String']>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars['String']>;
};

/** Get a A System Initiative Application Entity Reply */
export type ApplicationEntityGetReply = {
  __typename?: 'ApplicationEntityGetReply';
  /** A System Initiative Application Entity Item */
  item?: Maybe<ApplicationEntity>;
};

/** Get a A System Initiative Application Entity Request */
export type ApplicationEntityGetRequest = {
  /** A System Initiative Application Entity ID */
  id?: Maybe<Scalars['ID']>;
};

/** List A System Initiative Application Entity Reply */
export type ApplicationEntityListReply = {
  __typename?: 'ApplicationEntityListReply';
  /** Items */
  items?: Maybe<Array<ApplicationEntity>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars['String']>;
  /** Total Count */
  totalCount?: Maybe<Scalars['String']>;
};

/** List A System Initiative Application Entity Request */
export type ApplicationEntityListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars['String']>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars['String']>;
  /** Page Token */
  pageToken?: Maybe<Scalars['String']>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars['String']>;
};

/** Edit Phantom Property Reply */
export type ApplicationEntityPhantomEditReply = {
  __typename?: 'ApplicationEntityPhantomEditReply';
  /** Entity Event */
  item?: Maybe<ApplicationEntityEvent>;
};

/** Edit Phantom Property Request */
export type ApplicationEntityPhantomEditRequest = {
  /** Entity ID */
  id?: Maybe<Scalars['ID']>;
  /** The Phantom Data property value */
  property?: Maybe<Scalars['Boolean']>;
};

export type ApplicationEntityProperties = {
  __typename?: 'ApplicationEntityProperties';
  /** Phantom Data */
  phantom?: Maybe<Scalars['Boolean']>;
};

export type ApplicationEntityPropertiesRequest = {
  /** Phantom Data */
  phantom?: Maybe<Scalars['Boolean']>;
};

/** Sync State Reply */
export type ApplicationEntitySyncReply = {
  __typename?: 'ApplicationEntitySyncReply';
  /** Entity Event */
  item?: Maybe<ApplicationEntityEvent>;
};

/** Sync State Request */
export type ApplicationEntitySyncRequest = {
  /** Entity ID */
  id?: Maybe<Scalars['ID']>;
};

/** Update an Entity Reply */
export type ApplicationEntityUpdateReply = {
  __typename?: 'ApplicationEntityUpdateReply';
  /** application Item */
  item?: Maybe<ApplicationEntity>;
};

/** Update an Entity Request */
export type ApplicationEntityUpdateRequest = {
  /** Change Set ID */
  changeSetId?: Maybe<Scalars['String']>;
  /** applicationEntity ID */
  id?: Maybe<Scalars['ID']>;
  /** application Item Update */
  update?: Maybe<ApplicationEntityUpdateRequestUpdateRequest>;
};

export type ApplicationEntityUpdateRequestUpdateRequest = {
  /** description */
  description?: Maybe<Scalars['String']>;
  /** displayName */
  displayName?: Maybe<Scalars['String']>;
  /** name */
  name?: Maybe<Scalars['String']>;
  /** properties */
  properties?: Maybe<ApplicationEntityPropertiesRequest>;
};

export type BillingAccount = {
  __typename?: 'BillingAccount';
  associations?: Maybe<BillingAccountAssociations>;
  /** System Initiative Billing Account Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** System Initiative Billing Account ID */
  id?: Maybe<Scalars['ID']>;
  /** System Initiative Billing Account Name */
  name?: Maybe<Scalars['String']>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
};

/** System Initiative Billing Account Associations */
export type BillingAccountAssociations = {
  __typename?: 'BillingAccountAssociations';
  /** An instance of an integration with another system */
  integrationInstances?: Maybe<IntegrationInstanceListReply>;
  /** A System Initiative Organization */
  organizations?: Maybe<OrganizationListReply>;
  /** A System Initiative User */
  users?: Maybe<UserListReply>;
  /** A System Initiative Workspace */
  workspaces?: Maybe<WorkspaceListReply>;
};


/** System Initiative Billing Account Associations */
export type BillingAccountAssociationsIntegrationInstancesArgs = {
  input?: Maybe<IntegrationInstanceListRequest>;
};


/** System Initiative Billing Account Associations */
export type BillingAccountAssociationsOrganizationsArgs = {
  input?: Maybe<OrganizationListRequest>;
};


/** System Initiative Billing Account Associations */
export type BillingAccountAssociationsUsersArgs = {
  input?: Maybe<UserListRequest>;
};


/** System Initiative Billing Account Associations */
export type BillingAccountAssociationsWorkspacesArgs = {
  input?: Maybe<WorkspaceListRequest>;
};

/** Get a System Initiative Billing Account Reply */
export type BillingAccountGetReply = {
  __typename?: 'BillingAccountGetReply';
  /** System Initiative Billing Account Item */
  item?: Maybe<BillingAccount>;
};

/** Get a System Initiative Billing Account Request */
export type BillingAccountGetRequest = {
  /** System Initiative Billing Account ID */
  id?: Maybe<Scalars['ID']>;
};

/** List System Initiative Billing Account Reply */
export type BillingAccountListReply = {
  __typename?: 'BillingAccountListReply';
  /** Items */
  items?: Maybe<Array<BillingAccount>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars['String']>;
  /** Total Count */
  totalCount?: Maybe<Scalars['String']>;
};

/** List System Initiative Billing Account Request */
export type BillingAccountListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars['String']>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars['String']>;
  /** Page Token */
  pageToken?: Maybe<Scalars['String']>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars['String']>;
};

/** Create a Billing Account and Administrative User Reply */
export type BillingAccountSignupReply = {
  __typename?: 'BillingAccountSignupReply';
  /** Billing Account Object */
  billingAccount?: Maybe<BillingAccount>;
  /** User Object */
  user?: Maybe<User>;
};

/** Create a Billing Account and Administrative User Request */
export type BillingAccountSignupRequest = {
  /** Billing Account Information */
  billingAccount?: Maybe<BillingAccountSignupRequestBillingAccountRequest>;
  /** User Information */
  user?: Maybe<BillingAccountSignupRequestUserRequest>;
};

export type BillingAccountSignupRequestBillingAccountRequest = {
  /** Billing Account Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** Billing Account Name */
  name?: Maybe<Scalars['String']>;
};

export type BillingAccountSignupRequestUserRequest = {
  /** User Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** A valid email address */
  email?: Maybe<Scalars['String']>;
  /** User Name */
  name?: Maybe<Scalars['String']>;
  /** The users password hash */
  password?: Maybe<Scalars['String']>;
};

export type Capability = {
  __typename?: 'Capability';
  /** The actions this capability allows */
  actions?: Maybe<Array<Scalars['String']>>;
  /** The object the capability applies to */
  subject?: Maybe<Scalars['String']>;
};

export type CapabilityRequest = {
  /** The actions this capability allows */
  actions?: Maybe<Array<Scalars['String']>>;
  /** The object the capability applies to */
  subject?: Maybe<Scalars['String']>;
};

export type ChangeSet = {
  __typename?: 'ChangeSet';
  associations?: Maybe<ChangeSetAssociations>;
  /** User ID who created this Change Set */
  createdByUserId?: Maybe<Scalars['String']>;
  /** A change set for your system Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** Entry Count */
  entryCount?: Maybe<Scalars['String']>;
  /** A change set for your system ID */
  id?: Maybe<Scalars['ID']>;
  /** A change set for your system Name */
  name?: Maybe<Scalars['String']>;
  /** Note */
  note?: Maybe<Scalars['String']>;
  /** SI Internal Properties */
  siProperties?: Maybe<ChangeSetSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
  /** The status of this Change Set */
  status?: Maybe<ChangeSetStatus>;
};

/** A change set for your system Associations */
export type ChangeSetAssociations = {
  __typename?: 'ChangeSetAssociations';
  /** System Initiative Billing Account */
  billingAccount?: Maybe<BillingAccountGetReply>;
  /** An item */
  changeSetEntries?: Maybe<ItemListReply>;
  /** A System Initiative Organization */
  organization?: Maybe<OrganizationGetReply>;
  /** A System Initiative Workspace */
  workspace?: Maybe<WorkspaceGetReply>;
};


/** A change set for your system Associations */
export type ChangeSetAssociationsChangeSetEntriesArgs = {
  input?: Maybe<ItemListRequest>;
};

/** Create a Change Set Reply */
export type ChangeSetCreateReply = {
  __typename?: 'ChangeSetCreateReply';
  /** A change set for your system Item */
  item?: Maybe<ChangeSet>;
};

/** Create a Change Set Request */
export type ChangeSetCreateRequest = {
  /** User ID who created this Change Set */
  createdByUserId?: Maybe<Scalars['String']>;
  /** Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** Name */
  name?: Maybe<Scalars['String']>;
  /** Note */
  note?: Maybe<Scalars['String']>;
  /** Workspace ID */
  workspaceId?: Maybe<Scalars['String']>;
};

/** Execute a Change Set Reply */
export type ChangeSetExecuteReply = {
  __typename?: 'ChangeSetExecuteReply';
  /** ChangeSet Item */
  item?: Maybe<ChangeSet>;
};

/** Execute a Change Set Request */
export type ChangeSetExecuteRequest = {
  /** Change Set ID */
  id?: Maybe<Scalars['ID']>;
};

/** Get a A change set for your system Reply */
export type ChangeSetGetReply = {
  __typename?: 'ChangeSetGetReply';
  /** A change set for your system Item */
  item?: Maybe<ChangeSet>;
};

/** Get a A change set for your system Request */
export type ChangeSetGetRequest = {
  /** A change set for your system ID */
  id?: Maybe<Scalars['ID']>;
};

/** List A change set for your system Reply */
export type ChangeSetListReply = {
  __typename?: 'ChangeSetListReply';
  /** Items */
  items?: Maybe<Array<ChangeSet>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars['String']>;
  /** Total Count */
  totalCount?: Maybe<Scalars['String']>;
};

/** List A change set for your system Request */
export type ChangeSetListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars['String']>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars['String']>;
  /** Page Token */
  pageToken?: Maybe<Scalars['String']>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars['String']>;
};

export type ChangeSetSiProperties = {
  __typename?: 'ChangeSetSiProperties';
  /** Billing Account ID */
  billingAccountId?: Maybe<Scalars['String']>;
  /** Organization ID */
  organizationId?: Maybe<Scalars['String']>;
  /** Organization ID */
  workspaceId?: Maybe<Scalars['String']>;
};

/** The status of this Change Set */
export enum ChangeSetStatus {
  Abandoned = 'ABANDONED',
  Closed = 'CLOSED',
  Executing = 'EXECUTING',
  Failed = 'FAILED',
  Open = 'OPEN',
  Unknown = 'UNKNOWN'
}

export type ComponentSiProperties = {
  __typename?: 'ComponentSiProperties';
  /** Integration Id */
  integrationId?: Maybe<Scalars['String']>;
  /** Integration Service Id */
  integrationServiceId?: Maybe<Scalars['String']>;
  /** Version */
  version?: Maybe<Scalars['Int']>;
};

export type DataPageToken = {
  __typename?: 'DataPageToken';
  /** Contained Within */
  containedWithin?: Maybe<Scalars['String']>;
  /** Item ID */
  itemId?: Maybe<Scalars['String']>;
  /** Order by */
  orderBy?: Maybe<Scalars['String']>;
  /** Order by direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars['String']>;
  /** Query */
  query?: Maybe<DataQuery>;
};

/** Order by direction */
export enum DataPageTokenOrderByDirection {
  Asc = 'ASC',
  Desc = 'DESC',
  Unknown = 'UNKNOWN'
}

export type DataQuery = {
  __typename?: 'DataQuery';
  /** Query Boolean Logic */
  booleanTerm?: Maybe<DataQueryBooleanTerm>;
  /** Filter by Change Set ID */
  changeSetId?: Maybe<Scalars['String']>;
  /** Is Not */
  isNot?: Maybe<Scalars['Boolean']>;
  /** Expression Option */
  items?: Maybe<Array<DataQueryItems>>;
  /** Filter by View Context Tag */
  viewContext?: Maybe<Scalars['String']>;
};

/** Query Boolean Logic */
export enum DataQueryBooleanTerm {
  And = 'AND',
  Or = 'OR',
  Unknown = 'UNKNOWN'
}

export type DataQueryItems = {
  __typename?: 'DataQueryItems';
  /** Query Expression */
  expression?: Maybe<DataQueryItemsExpression>;
  /** Query */
  query?: Maybe<DataQuery>;
};

export type DataQueryItemsExpression = {
  __typename?: 'DataQueryItemsExpression';
  /** Query Comparison */
  comparison?: Maybe<DataQueryItemsExpressionComparison>;
  /** Field */
  field?: Maybe<Scalars['String']>;
  /** Query Field Type */
  fieldType?: Maybe<DataQueryItemsExpressionFieldType>;
  /** Value */
  value?: Maybe<Scalars['String']>;
};

/** Query Comparison */
export enum DataQueryItemsExpressionComparison {
  Contains = 'CONTAINS',
  Equals = 'EQUALS',
  Like = 'LIKE',
  NotEquals = 'NOT_EQUALS',
  NotLike = 'NOT_LIKE',
  Unknown = 'UNKNOWN'
}

/** Query Field Type */
export enum DataQueryItemsExpressionFieldType {
  Int = 'INT',
  String = 'STRING',
  Unknown = 'UNKNOWN'
}

export type DataQueryItemsExpressionRequest = {
  /** Query Comparison */
  comparison?: Maybe<DataQueryItemsExpressionComparison>;
  /** Field */
  field?: Maybe<Scalars['String']>;
  /** Query Field Type */
  fieldType?: Maybe<DataQueryItemsExpressionFieldType>;
  /** Value */
  value?: Maybe<Scalars['String']>;
};

export type DataQueryItemsRequest = {
  /** Query Expression */
  expression?: Maybe<DataQueryItemsExpressionRequest>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
};

export type DataQueryRequest = {
  /** Query Boolean Logic */
  booleanTerm?: Maybe<DataQueryBooleanTerm>;
  /** Filter by Change Set ID */
  changeSetId?: Maybe<Scalars['String']>;
  /** Is Not */
  isNot?: Maybe<Scalars['Boolean']>;
  /** Expression Option */
  items?: Maybe<Array<DataQueryItemsRequest>>;
  /** Filter by View Context Tag */
  viewContext?: Maybe<Scalars['String']>;
};

export type DataStorable = {
  __typename?: 'DataStorable';
  /** Order for the Change Set Entry */
  changeSetEntryCount?: Maybe<Scalars['String']>;
  /** The Change Set event type */
  changeSetEventType?: Maybe<DataStorableChangeSetEventType>;
  /** has this been executed */
  changeSetExecuted?: Maybe<Scalars['Boolean']>;
  /** The Change Set ID for this item */
  changeSetId?: Maybe<Scalars['String']>;
  /** has this been deleted */
  deleted?: Maybe<Scalars['Boolean']>;
  /** The canonical ID for this item */
  itemId?: Maybe<Scalars['String']>;
  /** Natural Key */
  naturalKey?: Maybe<Scalars['String']>;
  /** Tenant IDs */
  tenantIds?: Maybe<Array<Scalars['String']>>;
  /** Type Name */
  typeName?: Maybe<Scalars['String']>;
  /** View context tags */
  viewContext?: Maybe<Scalars['String']>;
};

/** The Change Set event type */
export enum DataStorableChangeSetEventType {
  Action = 'ACTION',
  Create = 'CREATE',
  Delete = 'DELETE',
  Unknown = 'UNKNOWN',
  Update = 'UPDATE'
}

export type EntityEventSiProperties = {
  __typename?: 'EntityEventSiProperties';
  /** Billing Account ID */
  billingAccountId?: Maybe<Scalars['String']>;
  /** Component Id */
  componentId?: Maybe<Scalars['String']>;
  /** Entity Id */
  entityId?: Maybe<Scalars['String']>;
  /** Integration Id */
  integrationId?: Maybe<Scalars['String']>;
  /** Integration Service Id */
  integrationServiceId?: Maybe<Scalars['String']>;
  /** Organization ID */
  organizationId?: Maybe<Scalars['String']>;
  /** Workspace ID */
  workspaceId?: Maybe<Scalars['String']>;
};

export type EntitySiProperties = {
  __typename?: 'EntitySiProperties';
  /** Billing Account ID */
  billingAccountId?: Maybe<Scalars['String']>;
  /** Change Set ID */
  changeSetId?: Maybe<Scalars['String']>;
  /** Component Id */
  componentId?: Maybe<Scalars['String']>;
  /** Entity State */
  entityState?: Maybe<EntitySiPropertiesEntityState>;
  /** Integration Id */
  integrationId?: Maybe<Scalars['String']>;
  /** Integration Service Id */
  integrationServiceId?: Maybe<Scalars['String']>;
  /** Organization ID */
  organizationId?: Maybe<Scalars['String']>;
  /** Version */
  version?: Maybe<Scalars['Int']>;
  /** Workspace ID */
  workspaceId?: Maybe<Scalars['String']>;
};

/** Entity State */
export enum EntitySiPropertiesEntityState {
  Error = 'ERROR',
  Ok = 'OK',
  Transition = 'TRANSITION',
  Unknown = 'UNKNOWN'
}

export type Group = {
  __typename?: 'Group';
  /** Authorized capabilities for this user */
  capabilities?: Maybe<Array<Capability>>;
  /** A System Initiative User Group Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** A System Initiative User Group ID */
  id?: Maybe<Scalars['ID']>;
  /** A System Initiative User Group Name */
  name?: Maybe<Scalars['String']>;
  /** SI Internal Properties */
  siProperties?: Maybe<GroupSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
  /** User IDs of our groups members */
  userIds?: Maybe<Array<Scalars['String']>>;
};

/** Create a Group Reply */
export type GroupCreateReply = {
  __typename?: 'GroupCreateReply';
  /** A System Initiative User Group Item */
  item?: Maybe<Group>;
};

/** Create a Group Request */
export type GroupCreateRequest = {
  /** Authorized capabilities for this user */
  capabilities?: Maybe<Array<CapabilityRequest>>;
  /** Group Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** Group Name */
  name?: Maybe<Scalars['String']>;
  /** The SI Properties for this User */
  siProperties?: Maybe<GroupSiPropertiesRequest>;
  /** Group user IDs */
  userIds?: Maybe<Array<Scalars['String']>>;
};

/** Get a A System Initiative User Group Reply */
export type GroupGetReply = {
  __typename?: 'GroupGetReply';
  /** A System Initiative User Group Item */
  item?: Maybe<Group>;
};

/** Get a A System Initiative User Group Request */
export type GroupGetRequest = {
  /** A System Initiative User Group ID */
  id?: Maybe<Scalars['ID']>;
};

/** List A System Initiative User Group Reply */
export type GroupListReply = {
  __typename?: 'GroupListReply';
  /** Items */
  items?: Maybe<Array<Group>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars['String']>;
  /** Total Count */
  totalCount?: Maybe<Scalars['String']>;
};

/** List A System Initiative User Group Request */
export type GroupListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars['String']>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars['String']>;
  /** Page Token */
  pageToken?: Maybe<Scalars['String']>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars['String']>;
};

export type GroupSiProperties = {
  __typename?: 'GroupSiProperties';
  /** Billing Account ID */
  billingAccountId?: Maybe<Scalars['String']>;
};

export type GroupSiPropertiesRequest = {
  /** Billing Account ID */
  billingAccountId?: Maybe<Scalars['String']>;
};

export type Integration = {
  __typename?: 'Integration';
  associations?: Maybe<IntegrationAssociations>;
  /** An integration with another system Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** An integration with another system ID */
  id?: Maybe<Scalars['ID']>;
  /** An integration with another system Name */
  name?: Maybe<Scalars['String']>;
  /** Options for this Integration */
  options?: Maybe<Array<IntegrationOptions>>;
  /** SI Internal Properties */
  siProperties?: Maybe<IntegrationSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
};

/** An integration with another system Associations */
export type IntegrationAssociations = {
  __typename?: 'IntegrationAssociations';
  /** System Initiative Billing Account */
  billingAccount?: Maybe<BillingAccountGetReply>;
  /** An instance of an integration with another system */
  integrationInstances?: Maybe<IntegrationInstanceListReply>;
};


/** An integration with another system Associations */
export type IntegrationAssociationsIntegrationInstancesArgs = {
  input?: Maybe<IntegrationInstanceListRequest>;
};

/** Get a An integration with another system Reply */
export type IntegrationGetReply = {
  __typename?: 'IntegrationGetReply';
  /** An integration with another system Item */
  item?: Maybe<Integration>;
};

/** Get a An integration with another system Request */
export type IntegrationGetRequest = {
  /** An integration with another system ID */
  id?: Maybe<Scalars['ID']>;
};

export type IntegrationInstance = {
  __typename?: 'IntegrationInstance';
  associations?: Maybe<IntegrationInstanceAssociations>;
  /** An instance of an integration with another system Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** An instance of an integration with another system ID */
  id?: Maybe<Scalars['ID']>;
  /** An instance of an integration with another system Name */
  name?: Maybe<Scalars['String']>;
  /** Options for this Integration */
  optionValues?: Maybe<Array<IntegrationInstanceOptionValues>>;
  /** SI Internal Properties */
  siProperties?: Maybe<IntegrationInstanceSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
};

/** An instance of an integration with another system Associations */
export type IntegrationInstanceAssociations = {
  __typename?: 'IntegrationInstanceAssociations';
  /** System Initiative Billing Account */
  billingAccount?: Maybe<BillingAccountGetReply>;
  /** An integration with another system */
  integration?: Maybe<IntegrationGetReply>;
  /** A System Initiative Organization */
  organizations?: Maybe<OrganizationListReply>;
  /** A System Initiative Workspace */
  workspaces?: Maybe<WorkspaceListReply>;
};


/** An instance of an integration with another system Associations */
export type IntegrationInstanceAssociationsOrganizationsArgs = {
  input?: Maybe<OrganizationListRequest>;
};


/** An instance of an integration with another system Associations */
export type IntegrationInstanceAssociationsWorkspacesArgs = {
  input?: Maybe<WorkspaceListRequest>;
};

/** Get a An instance of an integration with another system Reply */
export type IntegrationInstanceGetReply = {
  __typename?: 'IntegrationInstanceGetReply';
  /** An instance of an integration with another system Item */
  item?: Maybe<IntegrationInstance>;
};

/** Get a An instance of an integration with another system Request */
export type IntegrationInstanceGetRequest = {
  /** An instance of an integration with another system ID */
  id?: Maybe<Scalars['ID']>;
};

/** List An instance of an integration with another system Reply */
export type IntegrationInstanceListReply = {
  __typename?: 'IntegrationInstanceListReply';
  /** Items */
  items?: Maybe<Array<IntegrationInstance>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars['String']>;
  /** Total Count */
  totalCount?: Maybe<Scalars['String']>;
};

/** List An instance of an integration with another system Request */
export type IntegrationInstanceListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars['String']>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars['String']>;
  /** Page Token */
  pageToken?: Maybe<Scalars['String']>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars['String']>;
};

export type IntegrationInstanceOptionValues = {
  __typename?: 'IntegrationInstanceOptionValues';
  /** The name for this option */
  name?: Maybe<Scalars['String']>;
  /** The type of option */
  optionType?: Maybe<IntegrationOptionsOptionType>;
  /** The value for this option */
  value?: Maybe<Scalars['String']>;
};

export type IntegrationInstanceSiProperties = {
  __typename?: 'IntegrationInstanceSiProperties';
  /** Billing Account ID */
  billingAccountId?: Maybe<Scalars['String']>;
  /** List of organization id's this integration instance is enabled on */
  enabledOrganizationIdList?: Maybe<Array<Scalars['String']>>;
  /** List of workspace id's this integration instance is enabled on */
  enabledWorkspaceIdList?: Maybe<Array<Scalars['String']>>;
  /** Integration ID */
  integrationId?: Maybe<Scalars['String']>;
};

/** List An integration with another system Reply */
export type IntegrationListReply = {
  __typename?: 'IntegrationListReply';
  /** Items */
  items?: Maybe<Array<Integration>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars['String']>;
  /** Total Count */
  totalCount?: Maybe<Scalars['String']>;
};

/** List An integration with another system Request */
export type IntegrationListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars['String']>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars['String']>;
  /** Page Token */
  pageToken?: Maybe<Scalars['String']>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars['String']>;
};

export type IntegrationOptions = {
  __typename?: 'IntegrationOptions';
  /** The display name for this option */
  displayName?: Maybe<Scalars['String']>;
  /** The name for this option */
  name?: Maybe<Scalars['String']>;
  /** The type of option */
  optionType?: Maybe<IntegrationOptionsOptionType>;
};

/** The type of option */
export enum IntegrationOptionsOptionType {
  Secret = 'SECRET',
  String = 'STRING',
  Unknown = 'UNKNOWN'
}

export type IntegrationService = {
  __typename?: 'IntegrationService';
  associations?: Maybe<IntegrationServiceAssociations>;
  /** An service within an integration Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** An service within an integration ID */
  id?: Maybe<Scalars['ID']>;
  /** An service within an integration Name */
  name?: Maybe<Scalars['String']>;
  /** SI Internal Properties */
  siProperties?: Maybe<IntegrationServiceSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
};

/** An service within an integration Associations */
export type IntegrationServiceAssociations = {
  __typename?: 'IntegrationServiceAssociations';
  /** An integration with another system */
  integration?: Maybe<IntegrationGetReply>;
};

/** Get a An service within an integration Reply */
export type IntegrationServiceGetReply = {
  __typename?: 'IntegrationServiceGetReply';
  /** An service within an integration Item */
  item?: Maybe<IntegrationService>;
};

/** Get a An service within an integration Request */
export type IntegrationServiceGetRequest = {
  /** An service within an integration ID */
  id?: Maybe<Scalars['ID']>;
};

export type IntegrationServiceSiProperties = {
  __typename?: 'IntegrationServiceSiProperties';
  /** Integration ID */
  integrationId?: Maybe<Scalars['String']>;
  /** The version of this integration */
  version?: Maybe<Scalars['Int']>;
};

export type IntegrationSiProperties = {
  __typename?: 'IntegrationSiProperties';
  /** The version of this integration */
  version?: Maybe<Scalars['Int']>;
};

export type Item = {
  __typename?: 'Item';
  associations?: Maybe<ItemAssociations>;
  /** An item Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** An item ID */
  id?: Maybe<Scalars['ID']>;
  /** An item Name */
  name?: Maybe<Scalars['String']>;
  /** SI Internal Properties */
  siProperties?: Maybe<ItemSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
};

/** An item Associations */
export type ItemAssociations = {
  __typename?: 'ItemAssociations';
  /** System Initiative Billing Account */
  billingAccount?: Maybe<BillingAccountGetReply>;
  /** A System Initiative Organization */
  organization?: Maybe<OrganizationGetReply>;
  /** A System Initiative Workspace */
  workspace?: Maybe<WorkspaceGetReply>;
};

/** Get an Item Reply */
export type ItemGetReply = {
  __typename?: 'ItemGetReply';
  /** The Item */
  item?: Maybe<Item>;
};

/** Get an Item Request */
export type ItemGetRequest = {
  /** Item ID */
  id?: Maybe<Scalars['ID']>;
};

/** List Items Reply */
export type ItemListReply = {
  __typename?: 'ItemListReply';
  /** Items */
  items?: Maybe<Array<Item>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars['String']>;
  /** Total Count */
  totalCount?: Maybe<Scalars['String']>;
};

/** List Items Request */
export type ItemListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars['String']>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars['String']>;
  /** Page Token */
  pageToken?: Maybe<Scalars['String']>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars['String']>;
};

export type ItemSiProperties = {
  __typename?: 'ItemSiProperties';
  /** Billing Account ID */
  billingAccountId?: Maybe<Scalars['String']>;
  /** Organization ID */
  organizationId?: Maybe<Scalars['String']>;
  /** Organization ID */
  workspaceId?: Maybe<Scalars['String']>;
};

export type KubernetesContainer = {
  __typename?: 'KubernetesContainer';
  /** Image */
  image?: Maybe<Scalars['String']>;
  /** Name */
  name?: Maybe<Scalars['String']>;
  /** Ports */
  ports?: Maybe<Array<KubernetesContainerPort>>;
};

export type KubernetesContainerPort = {
  __typename?: 'KubernetesContainerPort';
  /** Container Port */
  containerPort?: Maybe<Scalars['Int']>;
  /** Host IP */
  hostIp?: Maybe<Scalars['String']>;
  /** Name */
  name?: Maybe<Scalars['String']>;
  /** Protocol */
  protocol?: Maybe<Scalars['String']>;
};

export type KubernetesContainerPortRequest = {
  /** Container Port */
  containerPort?: Maybe<Scalars['Int']>;
  /** Host IP */
  hostIp?: Maybe<Scalars['String']>;
  /** Name */
  name?: Maybe<Scalars['String']>;
  /** Protocol */
  protocol?: Maybe<Scalars['String']>;
};

export type KubernetesContainerRequest = {
  /** Image */
  image?: Maybe<Scalars['String']>;
  /** Name */
  name?: Maybe<Scalars['String']>;
  /** Ports */
  ports?: Maybe<Array<KubernetesContainerPortRequest>>;
};

export type KubernetesDeploymentComponent = {
  __typename?: 'KubernetesDeploymentComponent';
  /** Component Constraints */
  constraints?: Maybe<KubernetesDeploymentComponentConstraints>;
  /** Component Description */
  description?: Maybe<Scalars['String']>;
  /** Kubernetes Deployment Object Component Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** Kubernetes Deployment Object Component ID */
  id?: Maybe<Scalars['ID']>;
  /** Kubernetes Deployment Object Component Name */
  name?: Maybe<Scalars['String']>;
  /** SI Properties */
  siProperties?: Maybe<ComponentSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
};

export type KubernetesDeploymentComponentConstraints = {
  __typename?: 'KubernetesDeploymentComponentConstraints';
  /** Component Display Name */
  componentDisplayName?: Maybe<Scalars['String']>;
  /** Component Name */
  componentName?: Maybe<Scalars['String']>;
  /** Kubernetes Version */
  kubernetesVersion?: Maybe<KubernetesDeploymentComponentConstraintsKubernetesVersion>;
};

/** Kubernetes Version */
export enum KubernetesDeploymentComponentConstraintsKubernetesVersion {
  Unknown = 'UNKNOWN',
  V1_12 = 'V1_12',
  V1_13 = 'V1_13',
  V1_14 = 'V1_14',
  V1_15 = 'V1_15'
}

export type KubernetesDeploymentComponentConstraintsRequest = {
  /** Component Display Name */
  componentDisplayName?: Maybe<Scalars['String']>;
  /** Component Name */
  componentName?: Maybe<Scalars['String']>;
  /** Kubernetes Version */
  kubernetesVersion?: Maybe<KubernetesDeploymentComponentConstraintsKubernetesVersion>;
};

/** Get a Kubernetes Deployment Object Component Reply */
export type KubernetesDeploymentComponentGetReply = {
  __typename?: 'KubernetesDeploymentComponentGetReply';
  /** Kubernetes Deployment Object Component Item */
  item?: Maybe<KubernetesDeploymentComponent>;
};

/** Get a Kubernetes Deployment Object Component Request */
export type KubernetesDeploymentComponentGetRequest = {
  /** Kubernetes Deployment Object Component ID */
  id?: Maybe<Scalars['ID']>;
};

/** List Kubernetes Deployment Object Component Reply */
export type KubernetesDeploymentComponentListReply = {
  __typename?: 'KubernetesDeploymentComponentListReply';
  /** Items */
  items?: Maybe<Array<KubernetesDeploymentComponent>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars['String']>;
  /** Total Count */
  totalCount?: Maybe<Scalars['String']>;
};

/** List Kubernetes Deployment Object Component Request */
export type KubernetesDeploymentComponentListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars['String']>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars['String']>;
  /** Page Token */
  pageToken?: Maybe<Scalars['String']>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars['String']>;
};

/** Pick Component Reply */
export type KubernetesDeploymentComponentPickReply = {
  __typename?: 'KubernetesDeploymentComponentPickReply';
  /** Chosen Component */
  component?: Maybe<KubernetesDeploymentComponent>;
  /** Implicit Constraints */
  implicitConstraints?: Maybe<KubernetesDeploymentComponentConstraints>;
};

/** Pick Component Request */
export type KubernetesDeploymentComponentPickRequest = {
  /** Constraints */
  constraints?: Maybe<KubernetesDeploymentComponentConstraintsRequest>;
};

export type KubernetesDeploymentEntity = {
  __typename?: 'KubernetesDeploymentEntity';
  associations?: Maybe<KubernetesDeploymentEntityAssociations>;
  /** Constraints */
  constraints?: Maybe<KubernetesDeploymentComponentConstraints>;
  /** Entity Description */
  description?: Maybe<Scalars['String']>;
  /** Kubernetes Deployment Object Entity Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** Kubernetes Deployment Object Entity ID */
  id?: Maybe<Scalars['ID']>;
  /** Implicit Constraints */
  implicitConstraints?: Maybe<KubernetesDeploymentComponentConstraints>;
  /** Kubernetes Deployment Object Entity Name */
  name?: Maybe<Scalars['String']>;
  /** Properties */
  properties?: Maybe<KubernetesDeploymentEntityProperties>;
  /** SI Properties */
  siProperties?: Maybe<EntitySiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
};

/** Apply Reply */
export type KubernetesDeploymentEntityApplyReply = {
  __typename?: 'KubernetesDeploymentEntityApplyReply';
  /** Entity Event */
  item?: Maybe<KubernetesDeploymentEntityEvent>;
};

/** Apply Request */
export type KubernetesDeploymentEntityApplyRequest = {
  /** Entity ID */
  id?: Maybe<Scalars['ID']>;
};

/** Kubernetes Deployment Object Entity Associations */
export type KubernetesDeploymentEntityAssociations = {
  __typename?: 'KubernetesDeploymentEntityAssociations';
  /** System Initiative Billing Account */
  billingAccount?: Maybe<BillingAccountGetReply>;
};

/** Create Entity Reply */
export type KubernetesDeploymentEntityCreateReply = {
  __typename?: 'KubernetesDeploymentEntityCreateReply';
  /** kubernetesDeploymentEntity Item */
  item?: Maybe<KubernetesDeploymentEntity>;
};

/** Create Entity Request */
export type KubernetesDeploymentEntityCreateRequest = {
  /** Change Set ID */
  changeSetId?: Maybe<Scalars['String']>;
  /** Constraints */
  constraints?: Maybe<KubernetesDeploymentComponentConstraintsRequest>;
  /** Description */
  description?: Maybe<Scalars['String']>;
  /** Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** Name */
  name?: Maybe<Scalars['String']>;
  /** Properties */
  properties?: Maybe<KubernetesDeploymentEntityPropertiesRequest>;
  /** Workspace ID */
  workspaceId?: Maybe<Scalars['String']>;
};

/** Delete Entity Reply */
export type KubernetesDeploymentEntityDeleteReply = {
  __typename?: 'KubernetesDeploymentEntityDeleteReply';
  /** kubernetesDeployment Item */
  item?: Maybe<KubernetesDeploymentEntity>;
};

/** Delete Entity Request */
export type KubernetesDeploymentEntityDeleteRequest = {
  /** Change Set ID */
  changeSetId?: Maybe<Scalars['String']>;
  /** kubernetesDeploymentEntity ID */
  id?: Maybe<Scalars['ID']>;
};

export type KubernetesDeploymentEntityEvent = {
  __typename?: 'KubernetesDeploymentEntityEvent';
  /** Action Name */
  actionName?: Maybe<Scalars['String']>;
  /** Creation Time */
  createTime?: Maybe<Scalars['String']>;
  /** Error Lines */
  errorLines?: Maybe<Array<Scalars['String']>>;
  /** Error Message */
  errorMessage?: Maybe<Scalars['String']>;
  /** Finalized */
  finalized?: Maybe<Scalars['Boolean']>;
  /** Final Time */
  finalTime?: Maybe<Scalars['String']>;
  /** Kubernetes Deployment Object EntityEvent ID */
  id?: Maybe<Scalars['ID']>;
  /** Input Entity */
  inputEntity?: Maybe<KubernetesDeploymentEntity>;
  /** Output Entity */
  outputEntity?: Maybe<KubernetesDeploymentEntity>;
  /** Output Lines */
  outputLines?: Maybe<Array<Scalars['String']>>;
  /** Previous Entity */
  previousEntity?: Maybe<KubernetesDeploymentEntity>;
  /** SI Properties */
  siProperties?: Maybe<EntityEventSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
  /** success */
  success?: Maybe<Scalars['Boolean']>;
  /** Updated Time */
  updatedTime?: Maybe<Scalars['String']>;
  /** User ID */
  userId?: Maybe<Scalars['String']>;
};

/** List Kubernetes Deployment Object EntityEvent Reply */
export type KubernetesDeploymentEntityEventListReply = {
  __typename?: 'KubernetesDeploymentEntityEventListReply';
  /** Items */
  items?: Maybe<Array<KubernetesDeploymentEntityEvent>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars['String']>;
  /** Total Count */
  totalCount?: Maybe<Scalars['String']>;
};

/** List Kubernetes Deployment Object EntityEvent Request */
export type KubernetesDeploymentEntityEventListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars['String']>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars['String']>;
  /** Page Token */
  pageToken?: Maybe<Scalars['String']>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars['String']>;
};

/** Get a Kubernetes Deployment Object Entity Reply */
export type KubernetesDeploymentEntityGetReply = {
  __typename?: 'KubernetesDeploymentEntityGetReply';
  /** Kubernetes Deployment Object Entity Item */
  item?: Maybe<KubernetesDeploymentEntity>;
};

/** Get a Kubernetes Deployment Object Entity Request */
export type KubernetesDeploymentEntityGetRequest = {
  /** Kubernetes Deployment Object Entity ID */
  id?: Maybe<Scalars['ID']>;
};

/** Edit kubernetesDeploymentEntityPropertiesKubernetesObject Property Reply */
export type KubernetesDeploymentEntityKubernetesObjectEditReply = {
  __typename?: 'KubernetesDeploymentEntityKubernetesObjectEditReply';
  /** Entity Event */
  item?: Maybe<KubernetesDeploymentEntityEvent>;
};

/** Edit kubernetesDeploymentEntityPropertiesKubernetesObject Property Request */
export type KubernetesDeploymentEntityKubernetesObjectEditRequest = {
  /** Entity ID */
  id?: Maybe<Scalars['ID']>;
  /** The Kubernetes Object property value */
  property?: Maybe<KubernetesDeploymentEntityPropertiesKubernetesObjectRequest>;
};

/** Edit KubernetesObjectYaml Property Reply */
export type KubernetesDeploymentEntityKubernetesObjectYamlEditReply = {
  __typename?: 'KubernetesDeploymentEntityKubernetesObjectYamlEditReply';
  /** Entity Event */
  item?: Maybe<KubernetesDeploymentEntityEvent>;
};

/** Edit KubernetesObjectYaml Property Request */
export type KubernetesDeploymentEntityKubernetesObjectYamlEditRequest = {
  /** Entity ID */
  id?: Maybe<Scalars['ID']>;
  /** The Kubernetes Object YAML property value */
  property?: Maybe<Scalars['String']>;
};

/** List Kubernetes Deployment Object Entity Reply */
export type KubernetesDeploymentEntityListReply = {
  __typename?: 'KubernetesDeploymentEntityListReply';
  /** Items */
  items?: Maybe<Array<KubernetesDeploymentEntity>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars['String']>;
  /** Total Count */
  totalCount?: Maybe<Scalars['String']>;
};

/** List Kubernetes Deployment Object Entity Request */
export type KubernetesDeploymentEntityListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars['String']>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars['String']>;
  /** Page Token */
  pageToken?: Maybe<Scalars['String']>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars['String']>;
};

export type KubernetesDeploymentEntityProperties = {
  __typename?: 'KubernetesDeploymentEntityProperties';
  /** Kubernetes Object */
  kubernetesObject?: Maybe<KubernetesDeploymentEntityPropertiesKubernetesObject>;
  /** Kubernetes Object YAML */
  kubernetesObjectYaml?: Maybe<Scalars['String']>;
};

export type KubernetesDeploymentEntityPropertiesKubernetesObject = {
  __typename?: 'KubernetesDeploymentEntityPropertiesKubernetesObject';
  /** API Version */
  apiVersion?: Maybe<Scalars['String']>;
  /** Kind */
  kind?: Maybe<Scalars['String']>;
  /** Metadata */
  metadata?: Maybe<KubernetesMetadata>;
  /** Deployment Spec */
  spec?: Maybe<KubernetesDeploymentEntityPropertiesKubernetesObjectSpec>;
};

export type KubernetesDeploymentEntityPropertiesKubernetesObjectRequest = {
  /** API Version */
  apiVersion?: Maybe<Scalars['String']>;
  /** Kind */
  kind?: Maybe<Scalars['String']>;
  /** Metadata */
  metadata?: Maybe<KubernetesMetadataRequest>;
  /** Deployment Spec */
  spec?: Maybe<KubernetesDeploymentEntityPropertiesKubernetesObjectSpecRequest>;
};

export type KubernetesDeploymentEntityPropertiesKubernetesObjectSpec = {
  __typename?: 'KubernetesDeploymentEntityPropertiesKubernetesObjectSpec';
  /** Replicas */
  replicas?: Maybe<Scalars['Int']>;
  /** Selector */
  selector?: Maybe<KubernetesSelector>;
  /** Pod Template Spec */
  template?: Maybe<KubernetesPodTemplateSpec>;
};

export type KubernetesDeploymentEntityPropertiesKubernetesObjectSpecRequest = {
  /** Replicas */
  replicas?: Maybe<Scalars['Int']>;
  /** Selector */
  selector?: Maybe<KubernetesSelectorRequest>;
  /** Pod Template Spec */
  template?: Maybe<KubernetesPodTemplateSpecRequest>;
};

export type KubernetesDeploymentEntityPropertiesRequest = {
  /** Kubernetes Object */
  kubernetesObject?: Maybe<KubernetesDeploymentEntityPropertiesKubernetesObjectRequest>;
  /** Kubernetes Object YAML */
  kubernetesObjectYaml?: Maybe<Scalars['String']>;
};

/** Sync State Reply */
export type KubernetesDeploymentEntitySyncReply = {
  __typename?: 'KubernetesDeploymentEntitySyncReply';
  /** Entity Event */
  item?: Maybe<KubernetesDeploymentEntityEvent>;
};

/** Sync State Request */
export type KubernetesDeploymentEntitySyncRequest = {
  /** Entity ID */
  id?: Maybe<Scalars['ID']>;
};

/** Update an Entity Reply */
export type KubernetesDeploymentEntityUpdateReply = {
  __typename?: 'KubernetesDeploymentEntityUpdateReply';
  /** kubernetesDeployment Item */
  item?: Maybe<KubernetesDeploymentEntity>;
};

/** Update an Entity Request */
export type KubernetesDeploymentEntityUpdateRequest = {
  /** Change Set ID */
  changeSetId?: Maybe<Scalars['String']>;
  /** kubernetesDeploymentEntity ID */
  id?: Maybe<Scalars['ID']>;
  /** kubernetesDeployment Item Update */
  update?: Maybe<KubernetesDeploymentEntityUpdateRequestUpdateRequest>;
};

export type KubernetesDeploymentEntityUpdateRequestUpdateRequest = {
  /** description */
  description?: Maybe<Scalars['String']>;
  /** displayName */
  displayName?: Maybe<Scalars['String']>;
  /** name */
  name?: Maybe<Scalars['String']>;
  /** properties */
  properties?: Maybe<KubernetesDeploymentEntityPropertiesRequest>;
};

export type KubernetesLoadBalancerStatus = {
  __typename?: 'KubernetesLoadBalancerStatus';
  /** Load Balancer Ingress */
  ingress?: Maybe<Array<KubernetesLoadBalancerStatusIngress>>;
};

export type KubernetesLoadBalancerStatusIngress = {
  __typename?: 'KubernetesLoadBalancerStatusIngress';
  /** Hostname */
  hostname?: Maybe<Scalars['String']>;
  /** IP */
  ip?: Maybe<Scalars['String']>;
};

export type KubernetesLoadBalancerStatusIngressRequest = {
  /** Hostname */
  hostname?: Maybe<Scalars['String']>;
  /** IP */
  ip?: Maybe<Scalars['String']>;
};

export type KubernetesLoadBalancerStatusRequest = {
  /** Load Balancer Ingress */
  ingress?: Maybe<Array<KubernetesLoadBalancerStatusIngressRequest>>;
};

export type KubernetesMetadata = {
  __typename?: 'KubernetesMetadata';
  /** Labels */
  labels?: Maybe<Array<Labels>>;
  /** Name */
  name?: Maybe<Scalars['String']>;
};

export type KubernetesMetadataRequest = {
  /** Labels */
  labels?: Maybe<Array<LabelsRequest>>;
  /** Name */
  name?: Maybe<Scalars['String']>;
};

export type KubernetesPodSpec = {
  __typename?: 'KubernetesPodSpec';
  /** Containers */
  containers?: Maybe<Array<KubernetesContainer>>;
};

export type KubernetesPodSpecRequest = {
  /** Containers */
  containers?: Maybe<Array<KubernetesContainerRequest>>;
};

export type KubernetesPodTemplateSpec = {
  __typename?: 'KubernetesPodTemplateSpec';
  /** Meta Data */
  metadata?: Maybe<KubernetesMetadata>;
  /** Pod Spec */
  spec?: Maybe<KubernetesPodSpec>;
};

export type KubernetesPodTemplateSpecRequest = {
  /** Meta Data */
  metadata?: Maybe<KubernetesMetadataRequest>;
  /** Pod Spec */
  spec?: Maybe<KubernetesPodSpecRequest>;
};

export type KubernetesSelector = {
  __typename?: 'KubernetesSelector';
  /** Match Labels */
  matchLabels?: Maybe<Array<MatchLabels>>;
};

export type KubernetesSelectorRequest = {
  /** Match Labels */
  matchLabels?: Maybe<Array<MatchLabelsRequest>>;
};

export type KubernetesServiceComponent = {
  __typename?: 'KubernetesServiceComponent';
  /** Component Constraints */
  constraints?: Maybe<KubernetesServiceComponentConstraints>;
  /** Component Description */
  description?: Maybe<Scalars['String']>;
  /** Kubernetes Service Object Component Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** Kubernetes Service Object Component ID */
  id?: Maybe<Scalars['ID']>;
  /** Kubernetes Service Object Component Name */
  name?: Maybe<Scalars['String']>;
  /** SI Properties */
  siProperties?: Maybe<ComponentSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
};

export type KubernetesServiceComponentConstraints = {
  __typename?: 'KubernetesServiceComponentConstraints';
  /** Component Display Name */
  componentDisplayName?: Maybe<Scalars['String']>;
  /** Component Name */
  componentName?: Maybe<Scalars['String']>;
  /** Kubernetes Version */
  kubernetesVersion?: Maybe<KubernetesServiceComponentConstraintsKubernetesVersion>;
};

/** Kubernetes Version */
export enum KubernetesServiceComponentConstraintsKubernetesVersion {
  Unknown = 'UNKNOWN',
  V1_12 = 'V1_12',
  V1_13 = 'V1_13',
  V1_14 = 'V1_14',
  V1_15 = 'V1_15'
}

export type KubernetesServiceComponentConstraintsRequest = {
  /** Component Display Name */
  componentDisplayName?: Maybe<Scalars['String']>;
  /** Component Name */
  componentName?: Maybe<Scalars['String']>;
  /** Kubernetes Version */
  kubernetesVersion?: Maybe<KubernetesServiceComponentConstraintsKubernetesVersion>;
};

/** Get a Kubernetes Service Object Component Reply */
export type KubernetesServiceComponentGetReply = {
  __typename?: 'KubernetesServiceComponentGetReply';
  /** Kubernetes Service Object Component Item */
  item?: Maybe<KubernetesServiceComponent>;
};

/** Get a Kubernetes Service Object Component Request */
export type KubernetesServiceComponentGetRequest = {
  /** Kubernetes Service Object Component ID */
  id?: Maybe<Scalars['ID']>;
};

/** List Kubernetes Service Object Component Reply */
export type KubernetesServiceComponentListReply = {
  __typename?: 'KubernetesServiceComponentListReply';
  /** Items */
  items?: Maybe<Array<KubernetesServiceComponent>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars['String']>;
  /** Total Count */
  totalCount?: Maybe<Scalars['String']>;
};

/** List Kubernetes Service Object Component Request */
export type KubernetesServiceComponentListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars['String']>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars['String']>;
  /** Page Token */
  pageToken?: Maybe<Scalars['String']>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars['String']>;
};

/** Pick Component Reply */
export type KubernetesServiceComponentPickReply = {
  __typename?: 'KubernetesServiceComponentPickReply';
  /** Chosen Component */
  component?: Maybe<KubernetesServiceComponent>;
  /** Implicit Constraints */
  implicitConstraints?: Maybe<KubernetesServiceComponentConstraints>;
};

/** Pick Component Request */
export type KubernetesServiceComponentPickRequest = {
  /** Constraints */
  constraints?: Maybe<KubernetesServiceComponentConstraintsRequest>;
};

export type KubernetesServiceEntity = {
  __typename?: 'KubernetesServiceEntity';
  associations?: Maybe<KubernetesServiceEntityAssociations>;
  /** Constraints */
  constraints?: Maybe<KubernetesServiceComponentConstraints>;
  /** Entity Description */
  description?: Maybe<Scalars['String']>;
  /** Kubernetes Service Object Entity Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** Kubernetes Service Object Entity ID */
  id?: Maybe<Scalars['ID']>;
  /** Implicit Constraints */
  implicitConstraints?: Maybe<KubernetesServiceComponentConstraints>;
  /** Kubernetes Service Object Entity Name */
  name?: Maybe<Scalars['String']>;
  /** Properties */
  properties?: Maybe<KubernetesServiceEntityProperties>;
  /** SI Properties */
  siProperties?: Maybe<EntitySiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
};

/** Kubernetes Service Object Entity Associations */
export type KubernetesServiceEntityAssociations = {
  __typename?: 'KubernetesServiceEntityAssociations';
  /** System Initiative Billing Account */
  billingAccount?: Maybe<BillingAccountGetReply>;
};

/** Create Entity Reply */
export type KubernetesServiceEntityCreateReply = {
  __typename?: 'KubernetesServiceEntityCreateReply';
  /** kubernetesServiceEntity Item */
  item?: Maybe<KubernetesServiceEntity>;
};

/** Create Entity Request */
export type KubernetesServiceEntityCreateRequest = {
  /** Change Set ID */
  changeSetId?: Maybe<Scalars['String']>;
  /** Constraints */
  constraints?: Maybe<KubernetesServiceComponentConstraintsRequest>;
  /** Description */
  description?: Maybe<Scalars['String']>;
  /** Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** Name */
  name?: Maybe<Scalars['String']>;
  /** Properties */
  properties?: Maybe<KubernetesServiceEntityPropertiesRequest>;
  /** Workspace ID */
  workspaceId?: Maybe<Scalars['String']>;
};

/** Delete Entity Reply */
export type KubernetesServiceEntityDeleteReply = {
  __typename?: 'KubernetesServiceEntityDeleteReply';
  /** kubernetesService Item */
  item?: Maybe<KubernetesServiceEntity>;
};

/** Delete Entity Request */
export type KubernetesServiceEntityDeleteRequest = {
  /** Change Set ID */
  changeSetId?: Maybe<Scalars['String']>;
  /** kubernetesServiceEntity ID */
  id?: Maybe<Scalars['ID']>;
};

export type KubernetesServiceEntityEvent = {
  __typename?: 'KubernetesServiceEntityEvent';
  /** Action Name */
  actionName?: Maybe<Scalars['String']>;
  /** Creation Time */
  createTime?: Maybe<Scalars['String']>;
  /** Error Lines */
  errorLines?: Maybe<Array<Scalars['String']>>;
  /** Error Message */
  errorMessage?: Maybe<Scalars['String']>;
  /** Finalized */
  finalized?: Maybe<Scalars['Boolean']>;
  /** Final Time */
  finalTime?: Maybe<Scalars['String']>;
  /** Kubernetes Service Object EntityEvent ID */
  id?: Maybe<Scalars['ID']>;
  /** Input Entity */
  inputEntity?: Maybe<KubernetesServiceEntity>;
  /** Output Entity */
  outputEntity?: Maybe<KubernetesServiceEntity>;
  /** Output Lines */
  outputLines?: Maybe<Array<Scalars['String']>>;
  /** Previous Entity */
  previousEntity?: Maybe<KubernetesServiceEntity>;
  /** SI Properties */
  siProperties?: Maybe<EntityEventSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
  /** success */
  success?: Maybe<Scalars['Boolean']>;
  /** Updated Time */
  updatedTime?: Maybe<Scalars['String']>;
  /** User ID */
  userId?: Maybe<Scalars['String']>;
};

/** List Kubernetes Service Object EntityEvent Reply */
export type KubernetesServiceEntityEventListReply = {
  __typename?: 'KubernetesServiceEntityEventListReply';
  /** Items */
  items?: Maybe<Array<KubernetesServiceEntityEvent>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars['String']>;
  /** Total Count */
  totalCount?: Maybe<Scalars['String']>;
};

/** List Kubernetes Service Object EntityEvent Request */
export type KubernetesServiceEntityEventListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars['String']>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars['String']>;
  /** Page Token */
  pageToken?: Maybe<Scalars['String']>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars['String']>;
};

/** Get a Kubernetes Service Object Entity Reply */
export type KubernetesServiceEntityGetReply = {
  __typename?: 'KubernetesServiceEntityGetReply';
  /** Kubernetes Service Object Entity Item */
  item?: Maybe<KubernetesServiceEntity>;
};

/** Get a Kubernetes Service Object Entity Request */
export type KubernetesServiceEntityGetRequest = {
  /** Kubernetes Service Object Entity ID */
  id?: Maybe<Scalars['ID']>;
};

/** Edit kubernetesServiceEntityPropertiesKubernetesObject Property Reply */
export type KubernetesServiceEntityKubernetesObjectEditReply = {
  __typename?: 'KubernetesServiceEntityKubernetesObjectEditReply';
  /** Entity Event */
  item?: Maybe<KubernetesServiceEntityEvent>;
};

/** Edit kubernetesServiceEntityPropertiesKubernetesObject Property Request */
export type KubernetesServiceEntityKubernetesObjectEditRequest = {
  /** Entity ID */
  id?: Maybe<Scalars['ID']>;
  /** The Kubernetes Object property value */
  property?: Maybe<KubernetesServiceEntityPropertiesKubernetesObjectRequest>;
};

/** Edit KubernetesObjectYaml Property Reply */
export type KubernetesServiceEntityKubernetesObjectYamlEditReply = {
  __typename?: 'KubernetesServiceEntityKubernetesObjectYamlEditReply';
  /** Entity Event */
  item?: Maybe<KubernetesServiceEntityEvent>;
};

/** Edit KubernetesObjectYaml Property Request */
export type KubernetesServiceEntityKubernetesObjectYamlEditRequest = {
  /** Entity ID */
  id?: Maybe<Scalars['ID']>;
  /** The Kubernetes Object YAML property value */
  property?: Maybe<Scalars['String']>;
};

/** List Kubernetes Service Object Entity Reply */
export type KubernetesServiceEntityListReply = {
  __typename?: 'KubernetesServiceEntityListReply';
  /** Items */
  items?: Maybe<Array<KubernetesServiceEntity>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars['String']>;
  /** Total Count */
  totalCount?: Maybe<Scalars['String']>;
};

/** List Kubernetes Service Object Entity Request */
export type KubernetesServiceEntityListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars['String']>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars['String']>;
  /** Page Token */
  pageToken?: Maybe<Scalars['String']>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars['String']>;
};

export type KubernetesServiceEntityProperties = {
  __typename?: 'KubernetesServiceEntityProperties';
  /** Kubernetes Object */
  kubernetesObject?: Maybe<KubernetesServiceEntityPropertiesKubernetesObject>;
  /** Kubernetes Object YAML */
  kubernetesObjectYaml?: Maybe<Scalars['String']>;
};

export type KubernetesServiceEntityPropertiesKubernetesObject = {
  __typename?: 'KubernetesServiceEntityPropertiesKubernetesObject';
  /** API Version */
  apiVersion?: Maybe<Scalars['String']>;
  /** Kind */
  kind?: Maybe<Scalars['String']>;
  /** Metadat */
  metadata?: Maybe<KubernetesMetadata>;
  /** Service Spec */
  spec?: Maybe<KubernetesServiceEntityPropertiesKubernetesObjectSpec>;
  /** Service Status */
  status?: Maybe<KubernetesServiceEntityPropertiesKubernetesObjectStatus>;
};

export type KubernetesServiceEntityPropertiesKubernetesObjectRequest = {
  /** API Version */
  apiVersion?: Maybe<Scalars['String']>;
  /** Kind */
  kind?: Maybe<Scalars['String']>;
  /** Metadat */
  metadata?: Maybe<KubernetesMetadataRequest>;
  /** Service Spec */
  spec?: Maybe<KubernetesServiceEntityPropertiesKubernetesObjectSpecRequest>;
  /** Service Status */
  status?: Maybe<KubernetesServiceEntityPropertiesKubernetesObjectStatusRequest>;
};

export type KubernetesServiceEntityPropertiesKubernetesObjectSpec = {
  __typename?: 'KubernetesServiceEntityPropertiesKubernetesObjectSpec';
  /** Host IP */
  clusterIp?: Maybe<Scalars['String']>;
  /** External IPs */
  externalIps?: Maybe<Array<Scalars['String']>>;
  /** External Name */
  externalName?: Maybe<Scalars['String']>;
  /** External Traffic Policy */
  externalTrafficPolicy?: Maybe<KubernetesServiceEntityPropertiesKubernetesObjectSpecExternalTrafficPolicy>;
  /** Health Check Node Port */
  healthCheckNodePort?: Maybe<Scalars['String']>;
  /** IP Family */
  ipFamily?: Maybe<KubernetesServiceEntityPropertiesKubernetesObjectSpecIpFamily>;
  /** Load Balancer IP */
  loadBalancerIp?: Maybe<Scalars['String']>;
  /** Load Balancer Source Ranges */
  loadBalancerSourceRanges?: Maybe<Array<Scalars['String']>>;
  /** Ports */
  ports?: Maybe<Array<KubernetesServicePort>>;
  /** Publish Not Ready Address */
  publishNotReadyAddress?: Maybe<Scalars['Boolean']>;
  /** Selector */
  selector?: Maybe<KubernetesSelector>;
  /** Session Affinity */
  sessionAffinity?: Maybe<KubernetesServiceEntityPropertiesKubernetesObjectSpecSessionAffinity>;
  /** Session Affinity Config */
  sessionAffinityConfig?: Maybe<KubernetesServiceEntityPropertiesKubernetesObjectSpecSessionAffinityConfig>;
  /** Topology Keys */
  topologyKeys?: Maybe<Array<Scalars['String']>>;
  /** Type */
  type?: Maybe<KubernetesServiceEntityPropertiesKubernetesObjectSpecType>;
};

/** External Traffic Policy */
export enum KubernetesServiceEntityPropertiesKubernetesObjectSpecExternalTrafficPolicy {
  Cluster = 'CLUSTER',
  Local = 'LOCAL',
  Unknown = 'UNKNOWN'
}

/** IP Family */
export enum KubernetesServiceEntityPropertiesKubernetesObjectSpecIpFamily {
  IPv4 = 'I_PV4',
  IPv6 = 'I_PV6',
  Unknown = 'UNKNOWN'
}

export type KubernetesServiceEntityPropertiesKubernetesObjectSpecRequest = {
  /** Host IP */
  clusterIp?: Maybe<Scalars['String']>;
  /** External IPs */
  externalIps?: Maybe<Array<Scalars['String']>>;
  /** External Name */
  externalName?: Maybe<Scalars['String']>;
  /** External Traffic Policy */
  externalTrafficPolicy?: Maybe<KubernetesServiceEntityPropertiesKubernetesObjectSpecExternalTrafficPolicy>;
  /** Health Check Node Port */
  healthCheckNodePort?: Maybe<Scalars['String']>;
  /** IP Family */
  ipFamily?: Maybe<KubernetesServiceEntityPropertiesKubernetesObjectSpecIpFamily>;
  /** Load Balancer IP */
  loadBalancerIp?: Maybe<Scalars['String']>;
  /** Load Balancer Source Ranges */
  loadBalancerSourceRanges?: Maybe<Array<Scalars['String']>>;
  /** Ports */
  ports?: Maybe<Array<KubernetesServicePortRequest>>;
  /** Publish Not Ready Address */
  publishNotReadyAddress?: Maybe<Scalars['Boolean']>;
  /** Selector */
  selector?: Maybe<KubernetesSelectorRequest>;
  /** Session Affinity */
  sessionAffinity?: Maybe<KubernetesServiceEntityPropertiesKubernetesObjectSpecSessionAffinity>;
  /** Session Affinity Config */
  sessionAffinityConfig?: Maybe<KubernetesServiceEntityPropertiesKubernetesObjectSpecSessionAffinityConfigRequest>;
  /** Topology Keys */
  topologyKeys?: Maybe<Array<Scalars['String']>>;
  /** Type */
  type?: Maybe<KubernetesServiceEntityPropertiesKubernetesObjectSpecType>;
};

/** Session Affinity */
export enum KubernetesServiceEntityPropertiesKubernetesObjectSpecSessionAffinity {
  ClientIp = 'CLIENT_IP',
  None = 'NONE',
  Unknown = 'UNKNOWN'
}

export type KubernetesServiceEntityPropertiesKubernetesObjectSpecSessionAffinityConfig = {
  __typename?: 'KubernetesServiceEntityPropertiesKubernetesObjectSpecSessionAffinityConfig';
  /** Client IP Config */
  clientIp?: Maybe<KubernetesServiceEntityPropertiesKubernetesObjectSpecSessionAffinityConfigClientIp>;
};

export type KubernetesServiceEntityPropertiesKubernetesObjectSpecSessionAffinityConfigClientIp = {
  __typename?: 'KubernetesServiceEntityPropertiesKubernetesObjectSpecSessionAffinityConfigClientIp';
  /** Timeout Seconds */
  timeoutSeconds?: Maybe<Scalars['String']>;
};

export type KubernetesServiceEntityPropertiesKubernetesObjectSpecSessionAffinityConfigClientIpRequest = {
  /** Timeout Seconds */
  timeoutSeconds?: Maybe<Scalars['String']>;
};

export type KubernetesServiceEntityPropertiesKubernetesObjectSpecSessionAffinityConfigRequest = {
  /** Client IP Config */
  clientIp?: Maybe<KubernetesServiceEntityPropertiesKubernetesObjectSpecSessionAffinityConfigClientIpRequest>;
};

/** Type */
export enum KubernetesServiceEntityPropertiesKubernetesObjectSpecType {
  ClusterIp = 'CLUSTER_IP',
  ExternalName = 'EXTERNAL_NAME',
  LoadBalancer = 'LOAD_BALANCER',
  NodePort = 'NODE_PORT',
  Unknown = 'UNKNOWN'
}

export type KubernetesServiceEntityPropertiesKubernetesObjectStatus = {
  __typename?: 'KubernetesServiceEntityPropertiesKubernetesObjectStatus';
  /** Load Balancer Status */
  loadBalancer?: Maybe<KubernetesLoadBalancerStatus>;
};

export type KubernetesServiceEntityPropertiesKubernetesObjectStatusRequest = {
  /** Load Balancer Status */
  loadBalancer?: Maybe<KubernetesLoadBalancerStatusRequest>;
};

export type KubernetesServiceEntityPropertiesRequest = {
  /** Kubernetes Object */
  kubernetesObject?: Maybe<KubernetesServiceEntityPropertiesKubernetesObjectRequest>;
  /** Kubernetes Object YAML */
  kubernetesObjectYaml?: Maybe<Scalars['String']>;
};

/** Sync State Reply */
export type KubernetesServiceEntitySyncReply = {
  __typename?: 'KubernetesServiceEntitySyncReply';
  /** Entity Event */
  item?: Maybe<KubernetesServiceEntityEvent>;
};

/** Sync State Request */
export type KubernetesServiceEntitySyncRequest = {
  /** Entity ID */
  id?: Maybe<Scalars['ID']>;
};

/** Update an Entity Reply */
export type KubernetesServiceEntityUpdateReply = {
  __typename?: 'KubernetesServiceEntityUpdateReply';
  /** kubernetesService Item */
  item?: Maybe<KubernetesServiceEntity>;
};

/** Update an Entity Request */
export type KubernetesServiceEntityUpdateRequest = {
  /** Change Set ID */
  changeSetId?: Maybe<Scalars['String']>;
  /** kubernetesServiceEntity ID */
  id?: Maybe<Scalars['ID']>;
  /** kubernetesService Item Update */
  update?: Maybe<KubernetesServiceEntityUpdateRequestUpdateRequest>;
};

export type KubernetesServiceEntityUpdateRequestUpdateRequest = {
  /** description */
  description?: Maybe<Scalars['String']>;
  /** displayName */
  displayName?: Maybe<Scalars['String']>;
  /** name */
  name?: Maybe<Scalars['String']>;
  /** properties */
  properties?: Maybe<KubernetesServiceEntityPropertiesRequest>;
};

export type KubernetesServicePort = {
  __typename?: 'KubernetesServicePort';
  /** App Protocol */
  appProtocol?: Maybe<Scalars['String']>;
  /** Name */
  name?: Maybe<Scalars['String']>;
  /** Node Port */
  nodePort?: Maybe<Scalars['String']>;
  /** Port */
  port?: Maybe<Scalars['String']>;
  /** Protocol */
  protocol?: Maybe<KubernetesServicePortProtocol>;
  /** Target Port */
  targetPort?: Maybe<Scalars['String']>;
};

/** Protocol */
export enum KubernetesServicePortProtocol {
  Sctp = 'SCTP',
  Tcp = 'TCP',
  Udp = 'UDP',
  Unknown = 'UNKNOWN'
}

export type KubernetesServicePortRequest = {
  /** App Protocol */
  appProtocol?: Maybe<Scalars['String']>;
  /** Name */
  name?: Maybe<Scalars['String']>;
  /** Node Port */
  nodePort?: Maybe<Scalars['String']>;
  /** Port */
  port?: Maybe<Scalars['String']>;
  /** Protocol */
  protocol?: Maybe<KubernetesServicePortProtocol>;
  /** Target Port */
  targetPort?: Maybe<Scalars['String']>;
};

/** Labels */
export type Labels = {
  __typename?: 'Labels';
  key?: Maybe<Scalars['String']>;
  value?: Maybe<Scalars['String']>;
};

/** Labels */
export type LabelsRequest = {
  key?: Maybe<Scalars['String']>;
  value?: Maybe<Scalars['String']>;
};

/** Match Labels */
export type MatchLabels = {
  __typename?: 'MatchLabels';
  key?: Maybe<Scalars['String']>;
  value?: Maybe<Scalars['String']>;
};

/** Match Labels */
export type MatchLabelsRequest = {
  key?: Maybe<Scalars['String']>;
  value?: Maybe<Scalars['String']>;
};

export type Mutation = {
  __typename?: 'Mutation';
  applicationEntityCreate?: Maybe<ApplicationEntityCreateReply>;
  applicationEntityDelete?: Maybe<ApplicationEntityDeleteReply>;
  applicationEntityPhantomEdit?: Maybe<ApplicationEntityPhantomEditReply>;
  applicationEntitySync?: Maybe<ApplicationEntitySyncReply>;
  applicationEntityUpdate?: Maybe<ApplicationEntityUpdateReply>;
  billingAccountSignup?: Maybe<BillingAccountSignupReply>;
  changeSetCreate?: Maybe<ChangeSetCreateReply>;
  changeSetExecute?: Maybe<ChangeSetExecuteReply>;
  groupCreate?: Maybe<GroupCreateReply>;
  kubernetesDeploymentEntityApply?: Maybe<KubernetesDeploymentEntityApplyReply>;
  kubernetesDeploymentEntityCreate?: Maybe<KubernetesDeploymentEntityCreateReply>;
  kubernetesDeploymentEntityDelete?: Maybe<KubernetesDeploymentEntityDeleteReply>;
  kubernetesDeploymentEntityKubernetesObjectEdit?: Maybe<KubernetesDeploymentEntityKubernetesObjectEditReply>;
  kubernetesDeploymentEntityKubernetesObjectYamlEdit?: Maybe<KubernetesDeploymentEntityKubernetesObjectYamlEditReply>;
  kubernetesDeploymentEntitySync?: Maybe<KubernetesDeploymentEntitySyncReply>;
  kubernetesDeploymentEntityUpdate?: Maybe<KubernetesDeploymentEntityUpdateReply>;
  kubernetesServiceEntityCreate?: Maybe<KubernetesServiceEntityCreateReply>;
  kubernetesServiceEntityDelete?: Maybe<KubernetesServiceEntityDeleteReply>;
  kubernetesServiceEntityKubernetesObjectEdit?: Maybe<KubernetesServiceEntityKubernetesObjectEditReply>;
  kubernetesServiceEntityKubernetesObjectYamlEdit?: Maybe<KubernetesServiceEntityKubernetesObjectYamlEditReply>;
  kubernetesServiceEntitySync?: Maybe<KubernetesServiceEntitySyncReply>;
  kubernetesServiceEntityUpdate?: Maybe<KubernetesServiceEntityUpdateReply>;
  organizationCreate?: Maybe<OrganizationCreateReply>;
  serviceEntityCreate?: Maybe<ServiceEntityCreateReply>;
  serviceEntityDelete?: Maybe<ServiceEntityDeleteReply>;
  serviceEntityDeploy?: Maybe<ServiceEntityDeployReply>;
  serviceEntityImageEdit?: Maybe<ServiceEntityImageEditReply>;
  serviceEntityPortEdit?: Maybe<ServiceEntityPortEditReply>;
  serviceEntityReplicasEdit?: Maybe<ServiceEntityReplicasEditReply>;
  serviceEntitySync?: Maybe<ServiceEntitySyncReply>;
  serviceEntityUpdate?: Maybe<ServiceEntityUpdateReply>;
  systemEntityCreate?: Maybe<SystemEntityCreateReply>;
  systemEntityDelete?: Maybe<SystemEntityDeleteReply>;
  systemEntityPhantomEdit?: Maybe<SystemEntityPhantomEditReply>;
  systemEntitySync?: Maybe<SystemEntitySyncReply>;
  systemEntityUpdate?: Maybe<SystemEntityUpdateReply>;
  userCreate?: Maybe<UserCreateReply>;
  workspaceCreate?: Maybe<WorkspaceCreateReply>;
};


export type MutationApplicationEntityCreateArgs = {
  input?: Maybe<ApplicationEntityCreateRequest>;
};


export type MutationApplicationEntityDeleteArgs = {
  input?: Maybe<ApplicationEntityDeleteRequest>;
};


export type MutationApplicationEntityPhantomEditArgs = {
  input?: Maybe<ApplicationEntityPhantomEditRequest>;
};


export type MutationApplicationEntitySyncArgs = {
  input?: Maybe<ApplicationEntitySyncRequest>;
};


export type MutationApplicationEntityUpdateArgs = {
  input?: Maybe<ApplicationEntityUpdateRequest>;
};


export type MutationBillingAccountSignupArgs = {
  input?: Maybe<BillingAccountSignupRequest>;
};


export type MutationChangeSetCreateArgs = {
  input?: Maybe<ChangeSetCreateRequest>;
};


export type MutationChangeSetExecuteArgs = {
  input?: Maybe<ChangeSetExecuteRequest>;
};


export type MutationGroupCreateArgs = {
  input?: Maybe<GroupCreateRequest>;
};


export type MutationKubernetesDeploymentEntityApplyArgs = {
  input?: Maybe<KubernetesDeploymentEntityApplyRequest>;
};


export type MutationKubernetesDeploymentEntityCreateArgs = {
  input?: Maybe<KubernetesDeploymentEntityCreateRequest>;
};


export type MutationKubernetesDeploymentEntityDeleteArgs = {
  input?: Maybe<KubernetesDeploymentEntityDeleteRequest>;
};


export type MutationKubernetesDeploymentEntityKubernetesObjectEditArgs = {
  input?: Maybe<KubernetesDeploymentEntityKubernetesObjectEditRequest>;
};


export type MutationKubernetesDeploymentEntityKubernetesObjectYamlEditArgs = {
  input?: Maybe<KubernetesDeploymentEntityKubernetesObjectYamlEditRequest>;
};


export type MutationKubernetesDeploymentEntitySyncArgs = {
  input?: Maybe<KubernetesDeploymentEntitySyncRequest>;
};


export type MutationKubernetesDeploymentEntityUpdateArgs = {
  input?: Maybe<KubernetesDeploymentEntityUpdateRequest>;
};


export type MutationKubernetesServiceEntityCreateArgs = {
  input?: Maybe<KubernetesServiceEntityCreateRequest>;
};


export type MutationKubernetesServiceEntityDeleteArgs = {
  input?: Maybe<KubernetesServiceEntityDeleteRequest>;
};


export type MutationKubernetesServiceEntityKubernetesObjectEditArgs = {
  input?: Maybe<KubernetesServiceEntityKubernetesObjectEditRequest>;
};


export type MutationKubernetesServiceEntityKubernetesObjectYamlEditArgs = {
  input?: Maybe<KubernetesServiceEntityKubernetesObjectYamlEditRequest>;
};


export type MutationKubernetesServiceEntitySyncArgs = {
  input?: Maybe<KubernetesServiceEntitySyncRequest>;
};


export type MutationKubernetesServiceEntityUpdateArgs = {
  input?: Maybe<KubernetesServiceEntityUpdateRequest>;
};


export type MutationOrganizationCreateArgs = {
  input?: Maybe<OrganizationCreateRequest>;
};


export type MutationServiceEntityCreateArgs = {
  input?: Maybe<ServiceEntityCreateRequest>;
};


export type MutationServiceEntityDeleteArgs = {
  input?: Maybe<ServiceEntityDeleteRequest>;
};


export type MutationServiceEntityDeployArgs = {
  input?: Maybe<ServiceEntityDeployRequest>;
};


export type MutationServiceEntityImageEditArgs = {
  input?: Maybe<ServiceEntityImageEditRequest>;
};


export type MutationServiceEntityPortEditArgs = {
  input?: Maybe<ServiceEntityPortEditRequest>;
};


export type MutationServiceEntityReplicasEditArgs = {
  input?: Maybe<ServiceEntityReplicasEditRequest>;
};


export type MutationServiceEntitySyncArgs = {
  input?: Maybe<ServiceEntitySyncRequest>;
};


export type MutationServiceEntityUpdateArgs = {
  input?: Maybe<ServiceEntityUpdateRequest>;
};


export type MutationSystemEntityCreateArgs = {
  input?: Maybe<SystemEntityCreateRequest>;
};


export type MutationSystemEntityDeleteArgs = {
  input?: Maybe<SystemEntityDeleteRequest>;
};


export type MutationSystemEntityPhantomEditArgs = {
  input?: Maybe<SystemEntityPhantomEditRequest>;
};


export type MutationSystemEntitySyncArgs = {
  input?: Maybe<SystemEntitySyncRequest>;
};


export type MutationSystemEntityUpdateArgs = {
  input?: Maybe<SystemEntityUpdateRequest>;
};


export type MutationUserCreateArgs = {
  input?: Maybe<UserCreateRequest>;
};


export type MutationWorkspaceCreateArgs = {
  input?: Maybe<WorkspaceCreateRequest>;
};

export type Organization = {
  __typename?: 'Organization';
  associations?: Maybe<OrganizationAssociations>;
  /** A System Initiative Organization Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** A System Initiative Organization ID */
  id?: Maybe<Scalars['ID']>;
  /** A System Initiative Organization Name */
  name?: Maybe<Scalars['String']>;
  /** SI Internal Properties */
  siProperties?: Maybe<OrganizationSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
};

/** A System Initiative Organization Associations */
export type OrganizationAssociations = {
  __typename?: 'OrganizationAssociations';
  /** System Initiative Billing Account */
  billingAccount?: Maybe<BillingAccountGetReply>;
  /** A System Initiative Workspace */
  workspaces?: Maybe<WorkspaceListReply>;
};


/** A System Initiative Organization Associations */
export type OrganizationAssociationsWorkspacesArgs = {
  input?: Maybe<WorkspaceListRequest>;
};

/** Create an Organization Reply */
export type OrganizationCreateReply = {
  __typename?: 'OrganizationCreateReply';
  /** A System Initiative Organization Item */
  item?: Maybe<Organization>;
};

/** Create an Organization Request */
export type OrganizationCreateRequest = {
  /** User Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** User Name */
  name?: Maybe<Scalars['String']>;
  /** The SI Properties for this User */
  siProperties?: Maybe<OrganizationSiPropertiesRequest>;
};

/** Get a A System Initiative Organization Reply */
export type OrganizationGetReply = {
  __typename?: 'OrganizationGetReply';
  /** A System Initiative Organization Item */
  item?: Maybe<Organization>;
};

/** Get a A System Initiative Organization Request */
export type OrganizationGetRequest = {
  /** A System Initiative Organization ID */
  id?: Maybe<Scalars['ID']>;
};

/** List A System Initiative Organization Reply */
export type OrganizationListReply = {
  __typename?: 'OrganizationListReply';
  /** Items */
  items?: Maybe<Array<Organization>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars['String']>;
  /** Total Count */
  totalCount?: Maybe<Scalars['String']>;
};

/** List A System Initiative Organization Request */
export type OrganizationListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars['String']>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars['String']>;
  /** Page Token */
  pageToken?: Maybe<Scalars['String']>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars['String']>;
};

export type OrganizationSiProperties = {
  __typename?: 'OrganizationSiProperties';
  /** Billing Account ID */
  billingAccountId?: Maybe<Scalars['String']>;
};

export type OrganizationSiPropertiesRequest = {
  /** Billing Account ID */
  billingAccountId?: Maybe<Scalars['String']>;
};

export type Query = {
  __typename?: 'Query';
  applicationComponentGet?: Maybe<ApplicationComponentGetReply>;
  applicationComponentList?: Maybe<ApplicationComponentListReply>;
  applicationComponentPick?: Maybe<ApplicationComponentPickReply>;
  applicationEntityEventList?: Maybe<ApplicationEntityEventListReply>;
  applicationEntityGet?: Maybe<ApplicationEntityGetReply>;
  applicationEntityList?: Maybe<ApplicationEntityListReply>;
  billingAccountGet?: Maybe<BillingAccountGetReply>;
  billingAccountList?: Maybe<BillingAccountListReply>;
  changeSetGet?: Maybe<ChangeSetGetReply>;
  changeSetList?: Maybe<ChangeSetListReply>;
  groupGet?: Maybe<GroupGetReply>;
  groupList?: Maybe<GroupListReply>;
  integrationGet?: Maybe<IntegrationGetReply>;
  integrationInstanceGet?: Maybe<IntegrationInstanceGetReply>;
  integrationInstanceList?: Maybe<IntegrationInstanceListReply>;
  integrationList?: Maybe<IntegrationListReply>;
  integrationServiceGet?: Maybe<IntegrationServiceGetReply>;
  itemGet?: Maybe<ItemGetReply>;
  itemList?: Maybe<ItemListReply>;
  kubernetesDeploymentComponentGet?: Maybe<KubernetesDeploymentComponentGetReply>;
  kubernetesDeploymentComponentList?: Maybe<KubernetesDeploymentComponentListReply>;
  kubernetesDeploymentComponentPick?: Maybe<KubernetesDeploymentComponentPickReply>;
  kubernetesDeploymentEntityEventList?: Maybe<KubernetesDeploymentEntityEventListReply>;
  kubernetesDeploymentEntityGet?: Maybe<KubernetesDeploymentEntityGetReply>;
  kubernetesDeploymentEntityList?: Maybe<KubernetesDeploymentEntityListReply>;
  kubernetesServiceComponentGet?: Maybe<KubernetesServiceComponentGetReply>;
  kubernetesServiceComponentList?: Maybe<KubernetesServiceComponentListReply>;
  kubernetesServiceComponentPick?: Maybe<KubernetesServiceComponentPickReply>;
  kubernetesServiceEntityEventList?: Maybe<KubernetesServiceEntityEventListReply>;
  kubernetesServiceEntityGet?: Maybe<KubernetesServiceEntityGetReply>;
  kubernetesServiceEntityList?: Maybe<KubernetesServiceEntityListReply>;
  organizationGet?: Maybe<OrganizationGetReply>;
  organizationList?: Maybe<OrganizationListReply>;
  serviceComponentGet?: Maybe<ServiceComponentGetReply>;
  serviceComponentList?: Maybe<ServiceComponentListReply>;
  serviceComponentPick?: Maybe<ServiceComponentPickReply>;
  serviceEntityEventList?: Maybe<ServiceEntityEventListReply>;
  serviceEntityGet?: Maybe<ServiceEntityGetReply>;
  serviceEntityList?: Maybe<ServiceEntityListReply>;
  systemComponentGet?: Maybe<SystemComponentGetReply>;
  systemComponentList?: Maybe<SystemComponentListReply>;
  systemComponentPick?: Maybe<SystemComponentPickReply>;
  systemEntityEventList?: Maybe<SystemEntityEventListReply>;
  systemEntityGet?: Maybe<SystemEntityGetReply>;
  systemEntityList?: Maybe<SystemEntityListReply>;
  userGet?: Maybe<UserGetReply>;
  userList?: Maybe<UserListReply>;
  userLogin?: Maybe<UserLoginReply>;
  workspaceGet?: Maybe<WorkspaceGetReply>;
  workspaceList?: Maybe<WorkspaceListReply>;
};


export type QueryApplicationComponentGetArgs = {
  input?: Maybe<ApplicationComponentGetRequest>;
};


export type QueryApplicationComponentListArgs = {
  input?: Maybe<ApplicationComponentListRequest>;
};


export type QueryApplicationComponentPickArgs = {
  input?: Maybe<ApplicationComponentPickRequest>;
};


export type QueryApplicationEntityEventListArgs = {
  input?: Maybe<ApplicationEntityEventListRequest>;
};


export type QueryApplicationEntityGetArgs = {
  input?: Maybe<ApplicationEntityGetRequest>;
};


export type QueryApplicationEntityListArgs = {
  input?: Maybe<ApplicationEntityListRequest>;
};


export type QueryBillingAccountGetArgs = {
  input?: Maybe<BillingAccountGetRequest>;
};


export type QueryBillingAccountListArgs = {
  input?: Maybe<BillingAccountListRequest>;
};


export type QueryChangeSetGetArgs = {
  input?: Maybe<ChangeSetGetRequest>;
};


export type QueryChangeSetListArgs = {
  input?: Maybe<ChangeSetListRequest>;
};


export type QueryGroupGetArgs = {
  input?: Maybe<GroupGetRequest>;
};


export type QueryGroupListArgs = {
  input?: Maybe<GroupListRequest>;
};


export type QueryIntegrationGetArgs = {
  input?: Maybe<IntegrationGetRequest>;
};


export type QueryIntegrationInstanceGetArgs = {
  input?: Maybe<IntegrationInstanceGetRequest>;
};


export type QueryIntegrationInstanceListArgs = {
  input?: Maybe<IntegrationInstanceListRequest>;
};


export type QueryIntegrationListArgs = {
  input?: Maybe<IntegrationListRequest>;
};


export type QueryIntegrationServiceGetArgs = {
  input?: Maybe<IntegrationServiceGetRequest>;
};


export type QueryItemGetArgs = {
  input?: Maybe<ItemGetRequest>;
};


export type QueryItemListArgs = {
  input?: Maybe<ItemListRequest>;
};


export type QueryKubernetesDeploymentComponentGetArgs = {
  input?: Maybe<KubernetesDeploymentComponentGetRequest>;
};


export type QueryKubernetesDeploymentComponentListArgs = {
  input?: Maybe<KubernetesDeploymentComponentListRequest>;
};


export type QueryKubernetesDeploymentComponentPickArgs = {
  input?: Maybe<KubernetesDeploymentComponentPickRequest>;
};


export type QueryKubernetesDeploymentEntityEventListArgs = {
  input?: Maybe<KubernetesDeploymentEntityEventListRequest>;
};


export type QueryKubernetesDeploymentEntityGetArgs = {
  input?: Maybe<KubernetesDeploymentEntityGetRequest>;
};


export type QueryKubernetesDeploymentEntityListArgs = {
  input?: Maybe<KubernetesDeploymentEntityListRequest>;
};


export type QueryKubernetesServiceComponentGetArgs = {
  input?: Maybe<KubernetesServiceComponentGetRequest>;
};


export type QueryKubernetesServiceComponentListArgs = {
  input?: Maybe<KubernetesServiceComponentListRequest>;
};


export type QueryKubernetesServiceComponentPickArgs = {
  input?: Maybe<KubernetesServiceComponentPickRequest>;
};


export type QueryKubernetesServiceEntityEventListArgs = {
  input?: Maybe<KubernetesServiceEntityEventListRequest>;
};


export type QueryKubernetesServiceEntityGetArgs = {
  input?: Maybe<KubernetesServiceEntityGetRequest>;
};


export type QueryKubernetesServiceEntityListArgs = {
  input?: Maybe<KubernetesServiceEntityListRequest>;
};


export type QueryOrganizationGetArgs = {
  input?: Maybe<OrganizationGetRequest>;
};


export type QueryOrganizationListArgs = {
  input?: Maybe<OrganizationListRequest>;
};


export type QueryServiceComponentGetArgs = {
  input?: Maybe<ServiceComponentGetRequest>;
};


export type QueryServiceComponentListArgs = {
  input?: Maybe<ServiceComponentListRequest>;
};


export type QueryServiceComponentPickArgs = {
  input?: Maybe<ServiceComponentPickRequest>;
};


export type QueryServiceEntityEventListArgs = {
  input?: Maybe<ServiceEntityEventListRequest>;
};


export type QueryServiceEntityGetArgs = {
  input?: Maybe<ServiceEntityGetRequest>;
};


export type QueryServiceEntityListArgs = {
  input?: Maybe<ServiceEntityListRequest>;
};


export type QuerySystemComponentGetArgs = {
  input?: Maybe<SystemComponentGetRequest>;
};


export type QuerySystemComponentListArgs = {
  input?: Maybe<SystemComponentListRequest>;
};


export type QuerySystemComponentPickArgs = {
  input?: Maybe<SystemComponentPickRequest>;
};


export type QuerySystemEntityEventListArgs = {
  input?: Maybe<SystemEntityEventListRequest>;
};


export type QuerySystemEntityGetArgs = {
  input?: Maybe<SystemEntityGetRequest>;
};


export type QuerySystemEntityListArgs = {
  input?: Maybe<SystemEntityListRequest>;
};


export type QueryUserGetArgs = {
  input?: Maybe<UserGetRequest>;
};


export type QueryUserListArgs = {
  input?: Maybe<UserListRequest>;
};


export type QueryUserLoginArgs = {
  input?: Maybe<UserLoginRequest>;
};


export type QueryWorkspaceGetArgs = {
  input?: Maybe<WorkspaceGetRequest>;
};


export type QueryWorkspaceListArgs = {
  input?: Maybe<WorkspaceListRequest>;
};

export type ServiceComponent = {
  __typename?: 'ServiceComponent';
  /** Component Constraints */
  constraints?: Maybe<ServiceComponentConstraints>;
  /** Component Description */
  description?: Maybe<Scalars['String']>;
  /** Service Component Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** Service Component ID */
  id?: Maybe<Scalars['ID']>;
  /** Service Component Name */
  name?: Maybe<Scalars['String']>;
  /** SI Properties */
  siProperties?: Maybe<ComponentSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
};

export type ServiceComponentConstraints = {
  __typename?: 'ServiceComponentConstraints';
  /** Component Display Name */
  componentDisplayName?: Maybe<Scalars['String']>;
  /** Component Name */
  componentName?: Maybe<Scalars['String']>;
};

export type ServiceComponentConstraintsRequest = {
  /** Component Display Name */
  componentDisplayName?: Maybe<Scalars['String']>;
  /** Component Name */
  componentName?: Maybe<Scalars['String']>;
};

/** Get a Service Component Reply */
export type ServiceComponentGetReply = {
  __typename?: 'ServiceComponentGetReply';
  /** Service Component Item */
  item?: Maybe<ServiceComponent>;
};

/** Get a Service Component Request */
export type ServiceComponentGetRequest = {
  /** Service Component ID */
  id?: Maybe<Scalars['ID']>;
};

/** List Service Component Reply */
export type ServiceComponentListReply = {
  __typename?: 'ServiceComponentListReply';
  /** Items */
  items?: Maybe<Array<ServiceComponent>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars['String']>;
  /** Total Count */
  totalCount?: Maybe<Scalars['String']>;
};

/** List Service Component Request */
export type ServiceComponentListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars['String']>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars['String']>;
  /** Page Token */
  pageToken?: Maybe<Scalars['String']>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars['String']>;
};

/** Pick Component Reply */
export type ServiceComponentPickReply = {
  __typename?: 'ServiceComponentPickReply';
  /** Chosen Component */
  component?: Maybe<ServiceComponent>;
  /** Implicit Constraints */
  implicitConstraints?: Maybe<ServiceComponentConstraints>;
};

/** Pick Component Request */
export type ServiceComponentPickRequest = {
  /** Constraints */
  constraints?: Maybe<ServiceComponentConstraintsRequest>;
};

export type ServiceEntity = {
  __typename?: 'ServiceEntity';
  associations?: Maybe<ServiceEntityAssociations>;
  /** Constraints */
  constraints?: Maybe<ServiceComponentConstraints>;
  /** Entity Description */
  description?: Maybe<Scalars['String']>;
  /** Service Entity Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** Service Entity ID */
  id?: Maybe<Scalars['ID']>;
  /** Implicit Constraints */
  implicitConstraints?: Maybe<ServiceComponentConstraints>;
  /** Service Entity Name */
  name?: Maybe<Scalars['String']>;
  /** Properties */
  properties?: Maybe<ServiceEntityProperties>;
  /** SI Properties */
  siProperties?: Maybe<EntitySiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
};

/** Service Entity Associations */
export type ServiceEntityAssociations = {
  __typename?: 'ServiceEntityAssociations';
  /** System Initiative Billing Account */
  billingAccount?: Maybe<BillingAccountGetReply>;
};

/** Create Entity Reply */
export type ServiceEntityCreateReply = {
  __typename?: 'ServiceEntityCreateReply';
  /** serviceEntity Item */
  item?: Maybe<ServiceEntity>;
};

/** Create Entity Request */
export type ServiceEntityCreateRequest = {
  /** Change Set ID */
  changeSetId?: Maybe<Scalars['String']>;
  /** Constraints */
  constraints?: Maybe<ServiceComponentConstraintsRequest>;
  /** Description */
  description?: Maybe<Scalars['String']>;
  /** Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** Name */
  name?: Maybe<Scalars['String']>;
  /** Properties */
  properties?: Maybe<ServiceEntityPropertiesRequest>;
  /** Workspace ID */
  workspaceId?: Maybe<Scalars['String']>;
};

/** Delete Entity Reply */
export type ServiceEntityDeleteReply = {
  __typename?: 'ServiceEntityDeleteReply';
  /** service Item */
  item?: Maybe<ServiceEntity>;
};

/** Delete Entity Request */
export type ServiceEntityDeleteRequest = {
  /** Change Set ID */
  changeSetId?: Maybe<Scalars['String']>;
  /** serviceEntity ID */
  id?: Maybe<Scalars['ID']>;
};

/** Deploy Reply */
export type ServiceEntityDeployReply = {
  __typename?: 'ServiceEntityDeployReply';
  /** Entity Event */
  item?: Maybe<ServiceEntityEvent>;
};

/** Deploy Request */
export type ServiceEntityDeployRequest = {
  /** Entity ID */
  id?: Maybe<Scalars['ID']>;
};

export type ServiceEntityEvent = {
  __typename?: 'ServiceEntityEvent';
  /** Action Name */
  actionName?: Maybe<Scalars['String']>;
  /** Creation Time */
  createTime?: Maybe<Scalars['String']>;
  /** Error Lines */
  errorLines?: Maybe<Array<Scalars['String']>>;
  /** Error Message */
  errorMessage?: Maybe<Scalars['String']>;
  /** Finalized */
  finalized?: Maybe<Scalars['Boolean']>;
  /** Final Time */
  finalTime?: Maybe<Scalars['String']>;
  /** Service EntityEvent ID */
  id?: Maybe<Scalars['ID']>;
  /** Input Entity */
  inputEntity?: Maybe<ServiceEntity>;
  /** Output Entity */
  outputEntity?: Maybe<ServiceEntity>;
  /** Output Lines */
  outputLines?: Maybe<Array<Scalars['String']>>;
  /** Previous Entity */
  previousEntity?: Maybe<ServiceEntity>;
  /** SI Properties */
  siProperties?: Maybe<EntityEventSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
  /** success */
  success?: Maybe<Scalars['Boolean']>;
  /** Updated Time */
  updatedTime?: Maybe<Scalars['String']>;
  /** User ID */
  userId?: Maybe<Scalars['String']>;
};

/** List Service EntityEvent Reply */
export type ServiceEntityEventListReply = {
  __typename?: 'ServiceEntityEventListReply';
  /** Items */
  items?: Maybe<Array<ServiceEntityEvent>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars['String']>;
  /** Total Count */
  totalCount?: Maybe<Scalars['String']>;
};

/** List Service EntityEvent Request */
export type ServiceEntityEventListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars['String']>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars['String']>;
  /** Page Token */
  pageToken?: Maybe<Scalars['String']>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars['String']>;
};

/** Get a Service Entity Reply */
export type ServiceEntityGetReply = {
  __typename?: 'ServiceEntityGetReply';
  /** Service Entity Item */
  item?: Maybe<ServiceEntity>;
};

/** Get a Service Entity Request */
export type ServiceEntityGetRequest = {
  /** Service Entity ID */
  id?: Maybe<Scalars['ID']>;
};

/** Edit Image Property Reply */
export type ServiceEntityImageEditReply = {
  __typename?: 'ServiceEntityImageEditReply';
  /** Entity Event */
  item?: Maybe<ServiceEntityEvent>;
};

/** Edit Image Property Request */
export type ServiceEntityImageEditRequest = {
  /** Entity ID */
  id?: Maybe<Scalars['ID']>;
  /** The Container Image property value */
  property?: Maybe<Scalars['String']>;
};

/** List Service Entity Reply */
export type ServiceEntityListReply = {
  __typename?: 'ServiceEntityListReply';
  /** Items */
  items?: Maybe<Array<ServiceEntity>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars['String']>;
  /** Total Count */
  totalCount?: Maybe<Scalars['String']>;
};

/** List Service Entity Request */
export type ServiceEntityListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars['String']>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars['String']>;
  /** Page Token */
  pageToken?: Maybe<Scalars['String']>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars['String']>;
};

/** Edit Port Property Reply */
export type ServiceEntityPortEditReply = {
  __typename?: 'ServiceEntityPortEditReply';
  /** Entity Event */
  item?: Maybe<ServiceEntityEvent>;
};

/** Edit Port Property Request */
export type ServiceEntityPortEditRequest = {
  /** Entity ID */
  id?: Maybe<Scalars['ID']>;
  /** The Container Port property value */
  property?: Maybe<Scalars['String']>;
};

export type ServiceEntityProperties = {
  __typename?: 'ServiceEntityProperties';
  /** Container Image */
  image?: Maybe<Scalars['String']>;
  /** Container Port */
  port?: Maybe<Scalars['String']>;
  /** Replicas */
  replicas?: Maybe<Scalars['String']>;
};

export type ServiceEntityPropertiesRequest = {
  /** Container Image */
  image?: Maybe<Scalars['String']>;
  /** Container Port */
  port?: Maybe<Scalars['String']>;
  /** Replicas */
  replicas?: Maybe<Scalars['String']>;
};

/** Edit Replicas Property Reply */
export type ServiceEntityReplicasEditReply = {
  __typename?: 'ServiceEntityReplicasEditReply';
  /** Entity Event */
  item?: Maybe<ServiceEntityEvent>;
};

/** Edit Replicas Property Request */
export type ServiceEntityReplicasEditRequest = {
  /** Entity ID */
  id?: Maybe<Scalars['ID']>;
  /** The Replicas property value */
  property?: Maybe<Scalars['String']>;
};

/** Sync State Reply */
export type ServiceEntitySyncReply = {
  __typename?: 'ServiceEntitySyncReply';
  /** Entity Event */
  item?: Maybe<ServiceEntityEvent>;
};

/** Sync State Request */
export type ServiceEntitySyncRequest = {
  /** Entity ID */
  id?: Maybe<Scalars['ID']>;
};

/** Update an Entity Reply */
export type ServiceEntityUpdateReply = {
  __typename?: 'ServiceEntityUpdateReply';
  /** service Item */
  item?: Maybe<ServiceEntity>;
};

/** Update an Entity Request */
export type ServiceEntityUpdateRequest = {
  /** Change Set ID */
  changeSetId?: Maybe<Scalars['String']>;
  /** serviceEntity ID */
  id?: Maybe<Scalars['ID']>;
  /** service Item Update */
  update?: Maybe<ServiceEntityUpdateRequestUpdateRequest>;
};

export type ServiceEntityUpdateRequestUpdateRequest = {
  /** description */
  description?: Maybe<Scalars['String']>;
  /** displayName */
  displayName?: Maybe<Scalars['String']>;
  /** name */
  name?: Maybe<Scalars['String']>;
  /** properties */
  properties?: Maybe<ServiceEntityPropertiesRequest>;
};

export type SystemComponent = {
  __typename?: 'SystemComponent';
  /** Component Constraints */
  constraints?: Maybe<SystemComponentConstraints>;
  /** Component Description */
  description?: Maybe<Scalars['String']>;
  /** A System Initiative System Component Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** A System Initiative System Component ID */
  id?: Maybe<Scalars['ID']>;
  /** A System Initiative System Component Name */
  name?: Maybe<Scalars['String']>;
  /** SI Properties */
  siProperties?: Maybe<ComponentSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
};

export type SystemComponentConstraints = {
  __typename?: 'SystemComponentConstraints';
  /** Component Display Name */
  componentDisplayName?: Maybe<Scalars['String']>;
  /** Component Name */
  componentName?: Maybe<Scalars['String']>;
};

export type SystemComponentConstraintsRequest = {
  /** Component Display Name */
  componentDisplayName?: Maybe<Scalars['String']>;
  /** Component Name */
  componentName?: Maybe<Scalars['String']>;
};

/** Get a A System Initiative System Component Reply */
export type SystemComponentGetReply = {
  __typename?: 'SystemComponentGetReply';
  /** A System Initiative System Component Item */
  item?: Maybe<SystemComponent>;
};

/** Get a A System Initiative System Component Request */
export type SystemComponentGetRequest = {
  /** A System Initiative System Component ID */
  id?: Maybe<Scalars['ID']>;
};

/** List A System Initiative System Component Reply */
export type SystemComponentListReply = {
  __typename?: 'SystemComponentListReply';
  /** Items */
  items?: Maybe<Array<SystemComponent>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars['String']>;
  /** Total Count */
  totalCount?: Maybe<Scalars['String']>;
};

/** List A System Initiative System Component Request */
export type SystemComponentListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars['String']>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars['String']>;
  /** Page Token */
  pageToken?: Maybe<Scalars['String']>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars['String']>;
};

/** Pick Component Reply */
export type SystemComponentPickReply = {
  __typename?: 'SystemComponentPickReply';
  /** Chosen Component */
  component?: Maybe<SystemComponent>;
  /** Implicit Constraints */
  implicitConstraints?: Maybe<SystemComponentConstraints>;
};

/** Pick Component Request */
export type SystemComponentPickRequest = {
  /** Constraints */
  constraints?: Maybe<SystemComponentConstraintsRequest>;
};

export type SystemEntity = {
  __typename?: 'SystemEntity';
  /** Constraints */
  constraints?: Maybe<SystemComponentConstraints>;
  /** Entity Description */
  description?: Maybe<Scalars['String']>;
  /** A System Initiative System Entity Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** A System Initiative System Entity ID */
  id?: Maybe<Scalars['ID']>;
  /** Implicit Constraints */
  implicitConstraints?: Maybe<SystemComponentConstraints>;
  /** A System Initiative System Entity Name */
  name?: Maybe<Scalars['String']>;
  /** Properties */
  properties?: Maybe<SystemEntityProperties>;
  /** SI Properties */
  siProperties?: Maybe<EntitySiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
};

/** Create Entity Reply */
export type SystemEntityCreateReply = {
  __typename?: 'SystemEntityCreateReply';
  /** systemEntity Item */
  item?: Maybe<SystemEntity>;
};

/** Create Entity Request */
export type SystemEntityCreateRequest = {
  /** Change Set ID */
  changeSetId?: Maybe<Scalars['String']>;
  /** Constraints */
  constraints?: Maybe<SystemComponentConstraintsRequest>;
  /** Description */
  description?: Maybe<Scalars['String']>;
  /** Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** Name */
  name?: Maybe<Scalars['String']>;
  /** Properties */
  properties?: Maybe<SystemEntityPropertiesRequest>;
  /** Workspace ID */
  workspaceId?: Maybe<Scalars['String']>;
};

/** Delete Entity Reply */
export type SystemEntityDeleteReply = {
  __typename?: 'SystemEntityDeleteReply';
  /** system Item */
  item?: Maybe<SystemEntity>;
};

/** Delete Entity Request */
export type SystemEntityDeleteRequest = {
  /** Change Set ID */
  changeSetId?: Maybe<Scalars['String']>;
  /** systemEntity ID */
  id?: Maybe<Scalars['ID']>;
};

export type SystemEntityEvent = {
  __typename?: 'SystemEntityEvent';
  /** Action Name */
  actionName?: Maybe<Scalars['String']>;
  /** Creation Time */
  createTime?: Maybe<Scalars['String']>;
  /** Error Lines */
  errorLines?: Maybe<Array<Scalars['String']>>;
  /** Error Message */
  errorMessage?: Maybe<Scalars['String']>;
  /** Finalized */
  finalized?: Maybe<Scalars['Boolean']>;
  /** Final Time */
  finalTime?: Maybe<Scalars['String']>;
  /** A System Initiative System EntityEvent ID */
  id?: Maybe<Scalars['ID']>;
  /** Input Entity */
  inputEntity?: Maybe<SystemEntity>;
  /** Output Entity */
  outputEntity?: Maybe<SystemEntity>;
  /** Output Lines */
  outputLines?: Maybe<Array<Scalars['String']>>;
  /** Previous Entity */
  previousEntity?: Maybe<SystemEntity>;
  /** SI Properties */
  siProperties?: Maybe<EntityEventSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
  /** success */
  success?: Maybe<Scalars['Boolean']>;
  /** Updated Time */
  updatedTime?: Maybe<Scalars['String']>;
  /** User ID */
  userId?: Maybe<Scalars['String']>;
};

/** List A System Initiative System EntityEvent Reply */
export type SystemEntityEventListReply = {
  __typename?: 'SystemEntityEventListReply';
  /** Items */
  items?: Maybe<Array<SystemEntityEvent>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars['String']>;
  /** Total Count */
  totalCount?: Maybe<Scalars['String']>;
};

/** List A System Initiative System EntityEvent Request */
export type SystemEntityEventListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars['String']>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars['String']>;
  /** Page Token */
  pageToken?: Maybe<Scalars['String']>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars['String']>;
};

/** Get a A System Initiative System Entity Reply */
export type SystemEntityGetReply = {
  __typename?: 'SystemEntityGetReply';
  /** A System Initiative System Entity Item */
  item?: Maybe<SystemEntity>;
};

/** Get a A System Initiative System Entity Request */
export type SystemEntityGetRequest = {
  /** A System Initiative System Entity ID */
  id?: Maybe<Scalars['ID']>;
};

/** List A System Initiative System Entity Reply */
export type SystemEntityListReply = {
  __typename?: 'SystemEntityListReply';
  /** Items */
  items?: Maybe<Array<SystemEntity>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars['String']>;
  /** Total Count */
  totalCount?: Maybe<Scalars['String']>;
};

/** List A System Initiative System Entity Request */
export type SystemEntityListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars['String']>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars['String']>;
  /** Page Token */
  pageToken?: Maybe<Scalars['String']>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars['String']>;
};

/** Edit Phantom Property Reply */
export type SystemEntityPhantomEditReply = {
  __typename?: 'SystemEntityPhantomEditReply';
  /** Entity Event */
  item?: Maybe<SystemEntityEvent>;
};

/** Edit Phantom Property Request */
export type SystemEntityPhantomEditRequest = {
  /** Entity ID */
  id?: Maybe<Scalars['ID']>;
  /** The Phantom Data property value */
  property?: Maybe<Scalars['Boolean']>;
};

export type SystemEntityProperties = {
  __typename?: 'SystemEntityProperties';
  /** Phantom Data */
  phantom?: Maybe<Scalars['Boolean']>;
};

export type SystemEntityPropertiesRequest = {
  /** Phantom Data */
  phantom?: Maybe<Scalars['Boolean']>;
};

/** Sync State Reply */
export type SystemEntitySyncReply = {
  __typename?: 'SystemEntitySyncReply';
  /** Entity Event */
  item?: Maybe<SystemEntityEvent>;
};

/** Sync State Request */
export type SystemEntitySyncRequest = {
  /** Entity ID */
  id?: Maybe<Scalars['ID']>;
};

/** Update an Entity Reply */
export type SystemEntityUpdateReply = {
  __typename?: 'SystemEntityUpdateReply';
  /** system Item */
  item?: Maybe<SystemEntity>;
};

/** Update an Entity Request */
export type SystemEntityUpdateRequest = {
  /** Change Set ID */
  changeSetId?: Maybe<Scalars['String']>;
  /** systemEntity ID */
  id?: Maybe<Scalars['ID']>;
  /** system Item Update */
  update?: Maybe<SystemEntityUpdateRequestUpdateRequest>;
};

export type SystemEntityUpdateRequestUpdateRequest = {
  /** description */
  description?: Maybe<Scalars['String']>;
  /** displayName */
  displayName?: Maybe<Scalars['String']>;
  /** name */
  name?: Maybe<Scalars['String']>;
  /** properties */
  properties?: Maybe<SystemEntityPropertiesRequest>;
};

export type User = {
  __typename?: 'User';
  associations?: Maybe<UserAssociations>;
  /** Authorized capabilities for this user */
  capabilities?: Maybe<Capability>;
  /** A System Initiative User Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** A valid email address */
  email?: Maybe<Scalars['String']>;
  /** A System Initiative User ID */
  id?: Maybe<Scalars['ID']>;
  /** A System Initiative User Name */
  name?: Maybe<Scalars['String']>;
  /** SI Internal Properties */
  siProperties?: Maybe<UserSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
};

/** A System Initiative User Associations */
export type UserAssociations = {
  __typename?: 'UserAssociations';
  /** System Initiative Billing Account */
  billingAccount?: Maybe<BillingAccountGetReply>;
};

/** Create a User Reply */
export type UserCreateReply = {
  __typename?: 'UserCreateReply';
  /** A System Initiative User Item */
  item?: Maybe<User>;
};

/** Create a User Request */
export type UserCreateRequest = {
  /** User Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** Users email address */
  email?: Maybe<Scalars['String']>;
  /** User Name */
  name?: Maybe<Scalars['String']>;
  /** Users password */
  password?: Maybe<Scalars['String']>;
  /** The SI Properties for this User */
  siProperties?: Maybe<UserSiPropertiesRequest>;
};

/** Get a A System Initiative User Reply */
export type UserGetReply = {
  __typename?: 'UserGetReply';
  /** A System Initiative User Item */
  item?: Maybe<User>;
};

/** Get a A System Initiative User Request */
export type UserGetRequest = {
  /** A System Initiative User ID */
  id?: Maybe<Scalars['ID']>;
};

/** List A System Initiative User Reply */
export type UserListReply = {
  __typename?: 'UserListReply';
  /** Items */
  items?: Maybe<Array<User>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars['String']>;
  /** Total Count */
  totalCount?: Maybe<Scalars['String']>;
};

/** List A System Initiative User Request */
export type UserListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars['String']>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars['String']>;
  /** Page Token */
  pageToken?: Maybe<Scalars['String']>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars['String']>;
};

export type UserLoginReply = {
  __typename?: 'UserLoginReply';
  billingAccountId?: Maybe<Scalars['String']>;
  jwt?: Maybe<Scalars['String']>;
  userId?: Maybe<Scalars['String']>;
};

export type UserLoginRequest = {
  billingAccountName: Scalars['String'];
  email: Scalars['String'];
  password: Scalars['String'];
};

export type UserSiProperties = {
  __typename?: 'UserSiProperties';
  /** Billing Account ID */
  billingAccountId?: Maybe<Scalars['String']>;
};

export type UserSiPropertiesRequest = {
  /** Billing Account ID */
  billingAccountId?: Maybe<Scalars['String']>;
};

export type Workspace = {
  __typename?: 'Workspace';
  associations?: Maybe<WorkspaceAssociations>;
  /** A System Initiative Workspace Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** A System Initiative Workspace ID */
  id?: Maybe<Scalars['ID']>;
  /** A System Initiative Workspace Name */
  name?: Maybe<Scalars['String']>;
  /** SI Internal Properties */
  siProperties?: Maybe<WorkspaceSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
};

/** A System Initiative Workspace Associations */
export type WorkspaceAssociations = {
  __typename?: 'WorkspaceAssociations';
  /** System Initiative Billing Account */
  billingAccount?: Maybe<BillingAccountGetReply>;
  /** An instance of an integration with another system */
  integrationInstances?: Maybe<IntegrationInstanceListReply>;
  /** A System Initiative Organization */
  organization?: Maybe<OrganizationGetReply>;
};


/** A System Initiative Workspace Associations */
export type WorkspaceAssociationsIntegrationInstancesArgs = {
  input?: Maybe<IntegrationInstanceListRequest>;
};

/** Create an Organization Reply */
export type WorkspaceCreateReply = {
  __typename?: 'WorkspaceCreateReply';
  /** A System Initiative Workspace Item */
  item?: Maybe<Workspace>;
};

/** Create an Organization Request */
export type WorkspaceCreateRequest = {
  /** User Display Name */
  displayName?: Maybe<Scalars['String']>;
  /** User Name */
  name?: Maybe<Scalars['String']>;
  /** The SI Properties for this User */
  siProperties?: Maybe<WorkspaceSiPropertiesRequest>;
};

/** Get a A System Initiative Workspace Reply */
export type WorkspaceGetReply = {
  __typename?: 'WorkspaceGetReply';
  /** A System Initiative Workspace Item */
  item?: Maybe<Workspace>;
};

/** Get a A System Initiative Workspace Request */
export type WorkspaceGetRequest = {
  /** A System Initiative Workspace ID */
  id?: Maybe<Scalars['ID']>;
};

/** List A System Initiative Workspace Reply */
export type WorkspaceListReply = {
  __typename?: 'WorkspaceListReply';
  /** Items */
  items?: Maybe<Array<Workspace>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars['String']>;
  /** Total Count */
  totalCount?: Maybe<Scalars['String']>;
};

/** List A System Initiative Workspace Request */
export type WorkspaceListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars['String']>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars['String']>;
  /** Page Token */
  pageToken?: Maybe<Scalars['String']>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars['String']>;
};

export type WorkspaceSiProperties = {
  __typename?: 'WorkspaceSiProperties';
  /** Billing Account ID */
  billingAccountId?: Maybe<Scalars['String']>;
  /** Organization ID */
  organizationId?: Maybe<Scalars['String']>;
};

export type WorkspaceSiPropertiesRequest = {
  /** Billing Account ID */
  billingAccountId?: Maybe<Scalars['String']>;
  /** Organization ID */
  organizationId?: Maybe<Scalars['String']>;
};

