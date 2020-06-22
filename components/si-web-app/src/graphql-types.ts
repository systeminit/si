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
  associations?: Maybe<BillingAccountAssociations>;
  /** System Initiative Billing Account Display Name */
  displayName?: Maybe<Scalars["String"]>;
  /** System Initiative Billing Account ID */
  id?: Maybe<Scalars["ID"]>;
  /** System Initiative Billing Account Name */
  name?: Maybe<Scalars["String"]>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
};

/** System Initiative Billing Account Associations */
export type BillingAccountAssociations = {
  __typename?: "BillingAccountAssociations";
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
  __typename?: "BillingAccountGetReply";
  /** System Initiative Billing Account Item */
  item?: Maybe<BillingAccount>;
};

/** Get a System Initiative Billing Account Request */
export type BillingAccountGetRequest = {
  /** System Initiative Billing Account ID */
  id: Scalars["ID"];
};

/** List System Initiative Billing Account Reply */
export type BillingAccountListReply = {
  __typename?: "BillingAccountListReply";
  /** Items */
  items?: Maybe<Array<BillingAccount>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars["String"]>;
  /** Total Count */
  totalCount?: Maybe<Scalars["String"]>;
};

/** List System Initiative Billing Account Request */
export type BillingAccountListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars["String"]>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars["String"]>;
  /** Page Token */
  pageToken?: Maybe<Scalars["String"]>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars["String"]>;
};

/** Create a Billing Account and Administrative User Reply */
export type BillingAccountSignupReply = {
  __typename?: "BillingAccountSignupReply";
  /** Billing Account Object */
  billingAccount?: Maybe<BillingAccount>;
  /** User Object */
  user?: Maybe<User>;
};

/** Create a Billing Account and Administrative User Request */
export type BillingAccountSignupRequest = {
  /** Billing Account Information */
  billingAccount: BillingAccountSignupRequestBillingAccountRequest;
  /** User Information */
  user: BillingAccountSignupRequestUserRequest;
};

export type BillingAccountSignupRequestBillingAccountRequest = {
  /** Billing Account Display Name */
  displayName: Scalars["String"];
  /** Billing Account Name */
  name: Scalars["String"];
};

export type BillingAccountSignupRequestUserRequest = {
  /** User Display Name */
  displayName: Scalars["String"];
  /** A valid email address */
  email: Scalars["String"];
  /** User Name */
  name: Scalars["String"];
  /** The users password hash */
  password: Scalars["String"];
};

export type Capability = {
  __typename?: "Capability";
  /** The actions this capability allows */
  actions?: Maybe<Array<Scalars["String"]>>;
  /** The object the capability applies to */
  subject?: Maybe<Scalars["String"]>;
};

export type CapabilityRequest = {
  /** The actions this capability allows */
  actions: Array<Scalars["String"]>;
  /** The object the capability applies to */
  subject: Scalars["String"];
};

export type ChangeSet = {
  __typename?: "ChangeSet";
  associations?: Maybe<ChangeSetAssociations>;
  /** User ID who created this Change Set */
  createdByUserId?: Maybe<Scalars["String"]>;
  /** A change set for your system Display Name */
  displayName?: Maybe<Scalars["String"]>;
  /** Entry Count */
  entryCount?: Maybe<Scalars["String"]>;
  /** A change set for your system ID */
  id?: Maybe<Scalars["ID"]>;
  /** A change set for your system Name */
  name?: Maybe<Scalars["String"]>;
  /** Note */
  note?: Maybe<Scalars["String"]>;
  /** SI Internal Properties */
  siProperties?: Maybe<ChangeSetSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
  /** The status of this Change Set */
  status?: Maybe<ChangeSetStatus>;
};

/** A change set for your system Associations */
export type ChangeSetAssociations = {
  __typename?: "ChangeSetAssociations";
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
  __typename?: "ChangeSetCreateReply";
  /** A change set for your system Item */
  item?: Maybe<ChangeSet>;
};

/** Create a Change Set Request */
export type ChangeSetCreateRequest = {
  /** User ID who created this Change Set */
  createdByUserId: Scalars["String"];
  /** Display Name */
  displayName?: Maybe<Scalars["String"]>;
  /** Name */
  name: Scalars["String"];
  /** Note */
  note?: Maybe<Scalars["String"]>;
  /** Workspace ID */
  workspaceId: Scalars["String"];
};

/** Execute a Change Set Reply */
export type ChangeSetExecuteReply = {
  __typename?: "ChangeSetExecuteReply";
  /** ChangeSet Item */
  item?: Maybe<ChangeSet>;
};

/** Execute a Change Set Request */
export type ChangeSetExecuteRequest = {
  /** Change Set ID */
  id: Scalars["ID"];
};

/** Get a A change set for your system Reply */
export type ChangeSetGetReply = {
  __typename?: "ChangeSetGetReply";
  /** A change set for your system Item */
  item?: Maybe<ChangeSet>;
};

/** Get a A change set for your system Request */
export type ChangeSetGetRequest = {
  /** A change set for your system ID */
  id: Scalars["ID"];
};

/** List A change set for your system Reply */
export type ChangeSetListReply = {
  __typename?: "ChangeSetListReply";
  /** Items */
  items?: Maybe<Array<ChangeSet>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars["String"]>;
  /** Total Count */
  totalCount?: Maybe<Scalars["String"]>;
};

/** List A change set for your system Request */
export type ChangeSetListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars["String"]>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars["String"]>;
  /** Page Token */
  pageToken?: Maybe<Scalars["String"]>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars["String"]>;
};

export type ChangeSetSiProperties = {
  __typename?: "ChangeSetSiProperties";
  /** Billing Account ID */
  billingAccountId?: Maybe<Scalars["String"]>;
  /** Organization ID */
  organizationId?: Maybe<Scalars["String"]>;
  /** Organization ID */
  workspaceId?: Maybe<Scalars["String"]>;
};

/** The status of this Change Set */
export enum ChangeSetStatus {
  Abandoned = "ABANDONED",
  Closed = "CLOSED",
  Executing = "EXECUTING",
  Failed = "FAILED",
  Open = "OPEN",
  Unknown = "UNKNOWN",
}

export type ComponentSiProperties = {
  __typename?: "ComponentSiProperties";
  /** Integration Id */
  integrationId?: Maybe<Scalars["String"]>;
  /** Integration Service Id */
  integrationServiceId?: Maybe<Scalars["String"]>;
  /** Version */
  version?: Maybe<Scalars["Int"]>;
};

export type DataPageToken = {
  __typename?: "DataPageToken";
  /** Contained Within */
  containedWithin?: Maybe<Scalars["String"]>;
  /** Item ID */
  itemId?: Maybe<Scalars["String"]>;
  /** Order by */
  orderBy?: Maybe<Scalars["String"]>;
  /** Order by direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars["String"]>;
  /** Query */
  query?: Maybe<DataQuery>;
};

/** Order by direction */
export enum DataPageTokenOrderByDirection {
  Asc = "ASC",
  Desc = "DESC",
  Unknown = "UNKNOWN",
}

export type DataQuery = {
  __typename?: "DataQuery";
  /** Query Boolean Logic */
  booleanTerm?: Maybe<DataQueryBooleanTerm>;
  /** Filter by Change Set ID */
  changeSetId?: Maybe<Scalars["String"]>;
  /** Is Not */
  isNot?: Maybe<Scalars["Boolean"]>;
  /** Expression Option */
  items?: Maybe<Array<DataQueryItems>>;
  /** Filter by View Context Tag */
  viewContext?: Maybe<Scalars["String"]>;
};

/** Query Boolean Logic */
export enum DataQueryBooleanTerm {
  And = "AND",
  Or = "OR",
  Unknown = "UNKNOWN",
}

export type DataQueryItems = {
  __typename?: "DataQueryItems";
  /** Query Expression */
  expression?: Maybe<DataQueryItemsExpression>;
  /** Query */
  query?: Maybe<DataQuery>;
};

export type DataQueryItemsExpression = {
  __typename?: "DataQueryItemsExpression";
  /** Query Comparison */
  comparison?: Maybe<DataQueryItemsExpressionComparison>;
  /** Field */
  field?: Maybe<Scalars["String"]>;
  /** Query Field Type */
  fieldType?: Maybe<DataQueryItemsExpressionFieldType>;
  /** Value */
  value?: Maybe<Scalars["String"]>;
};

/** Query Comparison */
export enum DataQueryItemsExpressionComparison {
  Contains = "CONTAINS",
  Equals = "EQUALS",
  Like = "LIKE",
  NotEquals = "NOT_EQUALS",
  NotLike = "NOT_LIKE",
  Unknown = "UNKNOWN",
}

/** Query Field Type */
export enum DataQueryItemsExpressionFieldType {
  Int = "INT",
  String = "STRING",
  Unknown = "UNKNOWN",
}

export type DataQueryItemsExpressionRequest = {
  /** Query Comparison */
  comparison?: Maybe<DataQueryItemsExpressionComparison>;
  /** Field */
  field: Scalars["String"];
  /** Query Field Type */
  fieldType?: Maybe<DataQueryItemsExpressionFieldType>;
  /** Value */
  value: Scalars["String"];
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
  changeSetId?: Maybe<Scalars["String"]>;
  /** Is Not */
  isNot?: Maybe<Scalars["Boolean"]>;
  /** Expression Option */
  items?: Maybe<Array<DataQueryItemsRequest>>;
  /** Filter by View Context Tag */
  viewContext?: Maybe<Scalars["String"]>;
};

export type DataStorable = {
  __typename?: "DataStorable";
  /** Order for the Change Set Entry */
  changeSetEntryCount?: Maybe<Scalars["String"]>;
  /** The Change Set event type */
  changeSetEventType?: Maybe<DataStorableChangeSetEventType>;
  /** has this been executed */
  changeSetExecuted?: Maybe<Scalars["Boolean"]>;
  /** The Change Set ID for this item */
  changeSetId?: Maybe<Scalars["String"]>;
  /** has this been deleted */
  deleted?: Maybe<Scalars["Boolean"]>;
  /** The canonical ID for this item */
  itemId?: Maybe<Scalars["String"]>;
  /** Natural Key */
  naturalKey?: Maybe<Scalars["String"]>;
  /** Tenant IDs */
  tenantIds?: Maybe<Array<Scalars["String"]>>;
  /** Type Name */
  typeName?: Maybe<Scalars["String"]>;
  /** View context tags */
  viewContext?: Maybe<Scalars["String"]>;
};

/** The Change Set event type */
export enum DataStorableChangeSetEventType {
  Action = "ACTION",
  Create = "CREATE",
  Delete = "DELETE",
  Unknown = "UNKNOWN",
  Update = "UPDATE",
}

export type EntityEventSiProperties = {
  __typename?: "EntityEventSiProperties";
  /** Billing Account ID */
  billingAccountId?: Maybe<Scalars["String"]>;
  /** Component Id */
  componentId?: Maybe<Scalars["String"]>;
  /** Entity Id */
  entityId?: Maybe<Scalars["String"]>;
  /** Integration Id */
  integrationId?: Maybe<Scalars["String"]>;
  /** Integration Service Id */
  integrationServiceId?: Maybe<Scalars["String"]>;
  /** Organization ID */
  organizationId?: Maybe<Scalars["String"]>;
  /** Workspace ID */
  workspaceId?: Maybe<Scalars["String"]>;
};

export type EntitySiProperties = {
  __typename?: "EntitySiProperties";
  /** Billing Account ID */
  billingAccountId?: Maybe<Scalars["String"]>;
  /** Change Set ID */
  changeSetId?: Maybe<Scalars["String"]>;
  /** Component Id */
  componentId?: Maybe<Scalars["String"]>;
  /** Entity State */
  entityState?: Maybe<EntitySiPropertiesEntityState>;
  /** Integration Id */
  integrationId?: Maybe<Scalars["String"]>;
  /** Integration Service Id */
  integrationServiceId?: Maybe<Scalars["String"]>;
  /** Organization ID */
  organizationId?: Maybe<Scalars["String"]>;
  /** Version */
  version?: Maybe<Scalars["Int"]>;
  /** Workspace ID */
  workspaceId?: Maybe<Scalars["String"]>;
};

/** Entity State */
export enum EntitySiPropertiesEntityState {
  Error = "ERROR",
  Ok = "OK",
  Transition = "TRANSITION",
  Unknown = "UNKNOWN",
}

export type Group = {
  __typename?: "Group";
  /** Authorized capabilities for this user */
  capabilities?: Maybe<Array<Capability>>;
  /** A System Initiative User Group Display Name */
  displayName?: Maybe<Scalars["String"]>;
  /** A System Initiative User Group ID */
  id?: Maybe<Scalars["ID"]>;
  /** A System Initiative User Group Name */
  name?: Maybe<Scalars["String"]>;
  /** SI Internal Properties */
  siProperties?: Maybe<GroupSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
  /** User IDs of our groups members */
  userIds?: Maybe<Array<Scalars["String"]>>;
};

/** Create a Group Reply */
export type GroupCreateReply = {
  __typename?: "GroupCreateReply";
  /** A System Initiative User Group Item */
  item?: Maybe<Group>;
};

/** Create a Group Request */
export type GroupCreateRequest = {
  /** Authorized capabilities for this user */
  capabilities?: Maybe<Array<CapabilityRequest>>;
  /** Group Display Name */
  displayName: Scalars["String"];
  /** Group Name */
  name: Scalars["String"];
  /** The SI Properties for this User */
  siProperties: GroupSiPropertiesRequest;
  /** Group user IDs */
  userIds?: Maybe<Array<Scalars["String"]>>;
};

/** Get a A System Initiative User Group Reply */
export type GroupGetReply = {
  __typename?: "GroupGetReply";
  /** A System Initiative User Group Item */
  item?: Maybe<Group>;
};

/** Get a A System Initiative User Group Request */
export type GroupGetRequest = {
  /** A System Initiative User Group ID */
  id: Scalars["ID"];
};

/** List A System Initiative User Group Reply */
export type GroupListReply = {
  __typename?: "GroupListReply";
  /** Items */
  items?: Maybe<Array<Group>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars["String"]>;
  /** Total Count */
  totalCount?: Maybe<Scalars["String"]>;
};

/** List A System Initiative User Group Request */
export type GroupListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars["String"]>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars["String"]>;
  /** Page Token */
  pageToken?: Maybe<Scalars["String"]>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars["String"]>;
};

export type GroupSiProperties = {
  __typename?: "GroupSiProperties";
  /** Billing Account ID */
  billingAccountId?: Maybe<Scalars["String"]>;
};

export type GroupSiPropertiesRequest = {
  /** Billing Account ID */
  billingAccountId: Scalars["String"];
};

export type Integration = {
  __typename?: "Integration";
  associations?: Maybe<IntegrationAssociations>;
  /** An integration with another system Display Name */
  displayName?: Maybe<Scalars["String"]>;
  /** An integration with another system ID */
  id?: Maybe<Scalars["ID"]>;
  /** An integration with another system Name */
  name?: Maybe<Scalars["String"]>;
  /** Options for this Integration */
  options?: Maybe<Array<IntegrationOptions>>;
  /** SI Internal Properties */
  siProperties?: Maybe<IntegrationSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
};

/** An integration with another system Associations */
export type IntegrationAssociations = {
  __typename?: "IntegrationAssociations";
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
  __typename?: "IntegrationGetReply";
  /** An integration with another system Item */
  item?: Maybe<Integration>;
};

/** Get a An integration with another system Request */
export type IntegrationGetRequest = {
  /** An integration with another system ID */
  id: Scalars["ID"];
};

export type IntegrationInstance = {
  __typename?: "IntegrationInstance";
  associations?: Maybe<IntegrationInstanceAssociations>;
  /** An instance of an integration with another system Display Name */
  displayName?: Maybe<Scalars["String"]>;
  /** An instance of an integration with another system ID */
  id?: Maybe<Scalars["ID"]>;
  /** An instance of an integration with another system Name */
  name?: Maybe<Scalars["String"]>;
  /** Options for this Integration */
  optionValues?: Maybe<Array<IntegrationInstanceOptionValues>>;
  /** SI Internal Properties */
  siProperties?: Maybe<IntegrationInstanceSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
};

/** An instance of an integration with another system Associations */
export type IntegrationInstanceAssociations = {
  __typename?: "IntegrationInstanceAssociations";
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
  __typename?: "IntegrationInstanceGetReply";
  /** An instance of an integration with another system Item */
  item?: Maybe<IntegrationInstance>;
};

/** Get a An instance of an integration with another system Request */
export type IntegrationInstanceGetRequest = {
  /** An instance of an integration with another system ID */
  id: Scalars["ID"];
};

/** List An instance of an integration with another system Reply */
export type IntegrationInstanceListReply = {
  __typename?: "IntegrationInstanceListReply";
  /** Items */
  items?: Maybe<Array<IntegrationInstance>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars["String"]>;
  /** Total Count */
  totalCount?: Maybe<Scalars["String"]>;
};

/** List An instance of an integration with another system Request */
export type IntegrationInstanceListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars["String"]>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars["String"]>;
  /** Page Token */
  pageToken?: Maybe<Scalars["String"]>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars["String"]>;
};

export type IntegrationInstanceOptionValues = {
  __typename?: "IntegrationInstanceOptionValues";
  /** The name for this option */
  name?: Maybe<Scalars["String"]>;
  /** The type of option */
  optionType?: Maybe<IntegrationOptionsOptionType>;
  /** The value for this option */
  value?: Maybe<Scalars["String"]>;
};

export type IntegrationInstanceSiProperties = {
  __typename?: "IntegrationInstanceSiProperties";
  /** Billing Account ID */
  billingAccountId?: Maybe<Scalars["String"]>;
  /** List of organization id's this integration instance is enabled on */
  enabledOrganizationIdList?: Maybe<Array<Scalars["String"]>>;
  /** List of workspace id's this integration instance is enabled on */
  enabledWorkspaceIdList?: Maybe<Array<Scalars["String"]>>;
  /** Integration ID */
  integrationId?: Maybe<Scalars["String"]>;
};

/** List An integration with another system Reply */
export type IntegrationListReply = {
  __typename?: "IntegrationListReply";
  /** Items */
  items?: Maybe<Array<Integration>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars["String"]>;
  /** Total Count */
  totalCount?: Maybe<Scalars["String"]>;
};

/** List An integration with another system Request */
export type IntegrationListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars["String"]>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars["String"]>;
  /** Page Token */
  pageToken?: Maybe<Scalars["String"]>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars["String"]>;
};

export type IntegrationOptions = {
  __typename?: "IntegrationOptions";
  /** The display name for this option */
  displayName?: Maybe<Scalars["String"]>;
  /** The name for this option */
  name?: Maybe<Scalars["String"]>;
  /** The type of option */
  optionType?: Maybe<IntegrationOptionsOptionType>;
};

/** The type of option */
export enum IntegrationOptionsOptionType {
  Secret = "SECRET",
  String = "STRING",
  Unknown = "UNKNOWN",
}

export type IntegrationService = {
  __typename?: "IntegrationService";
  associations?: Maybe<IntegrationServiceAssociations>;
  /** An service within an integration Display Name */
  displayName?: Maybe<Scalars["String"]>;
  /** An service within an integration ID */
  id?: Maybe<Scalars["ID"]>;
  /** An service within an integration Name */
  name?: Maybe<Scalars["String"]>;
  /** SI Internal Properties */
  siProperties?: Maybe<IntegrationServiceSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
};

/** An service within an integration Associations */
export type IntegrationServiceAssociations = {
  __typename?: "IntegrationServiceAssociations";
  /** An integration with another system */
  integration?: Maybe<IntegrationGetReply>;
};

/** Get a An service within an integration Reply */
export type IntegrationServiceGetReply = {
  __typename?: "IntegrationServiceGetReply";
  /** An service within an integration Item */
  item?: Maybe<IntegrationService>;
};

/** Get a An service within an integration Request */
export type IntegrationServiceGetRequest = {
  /** An service within an integration ID */
  id: Scalars["ID"];
};

export type IntegrationServiceSiProperties = {
  __typename?: "IntegrationServiceSiProperties";
  /** Integration ID */
  integrationId?: Maybe<Scalars["String"]>;
  /** The version of this integration */
  version?: Maybe<Scalars["Int"]>;
};

export type IntegrationSiProperties = {
  __typename?: "IntegrationSiProperties";
  /** The version of this integration */
  version?: Maybe<Scalars["Int"]>;
};

export type Item = {
  __typename?: "Item";
  associations?: Maybe<ItemAssociations>;
  /** An item Display Name */
  displayName?: Maybe<Scalars["String"]>;
  /** An item ID */
  id?: Maybe<Scalars["ID"]>;
  /** An item Name */
  name?: Maybe<Scalars["String"]>;
  /** SI Internal Properties */
  siProperties?: Maybe<ItemSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
};

/** An item Associations */
export type ItemAssociations = {
  __typename?: "ItemAssociations";
  /** System Initiative Billing Account */
  billingAccount?: Maybe<BillingAccountGetReply>;
  /** A System Initiative Organization */
  organization?: Maybe<OrganizationGetReply>;
  /** A System Initiative Workspace */
  workspace?: Maybe<WorkspaceGetReply>;
};

/** Get an Item Reply */
export type ItemGetReply = {
  __typename?: "ItemGetReply";
  /** The Item */
  item?: Maybe<Item>;
};

/** Get an Item Request */
export type ItemGetRequest = {
  /** Item ID */
  id: Scalars["ID"];
};

/** List Items Reply */
export type ItemListReply = {
  __typename?: "ItemListReply";
  /** Items */
  items?: Maybe<Array<Item>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars["String"]>;
  /** Total Count */
  totalCount?: Maybe<Scalars["String"]>;
};

/** List Items Request */
export type ItemListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars["String"]>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars["String"]>;
  /** Page Token */
  pageToken?: Maybe<Scalars["String"]>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars["String"]>;
};

export type ItemSiProperties = {
  __typename?: "ItemSiProperties";
  /** Billing Account ID */
  billingAccountId?: Maybe<Scalars["String"]>;
  /** Organization ID */
  organizationId?: Maybe<Scalars["String"]>;
  /** Organization ID */
  workspaceId?: Maybe<Scalars["String"]>;
};

export type KubernetesContainer = {
  __typename?: "KubernetesContainer";
  /** Image */
  image?: Maybe<Scalars["String"]>;
  /** Name */
  name?: Maybe<Scalars["String"]>;
  /** Ports */
  ports?: Maybe<Array<KubernetesContainerPort>>;
};

export type KubernetesContainerPort = {
  __typename?: "KubernetesContainerPort";
  /** Container Port */
  containerPort?: Maybe<Scalars["Int"]>;
  /** Host IP */
  hostIp?: Maybe<Scalars["String"]>;
  /** Name */
  name?: Maybe<Scalars["String"]>;
  /** Protocol */
  protocol?: Maybe<Scalars["String"]>;
};

export type KubernetesContainerPortRequest = {
  /** Container Port */
  containerPort?: Maybe<Scalars["Int"]>;
  /** Host IP */
  hostIp?: Maybe<Scalars["String"]>;
  /** Name */
  name?: Maybe<Scalars["String"]>;
  /** Protocol */
  protocol?: Maybe<Scalars["String"]>;
};

export type KubernetesContainerRequest = {
  /** Image */
  image?: Maybe<Scalars["String"]>;
  /** Name */
  name?: Maybe<Scalars["String"]>;
  /** Ports */
  ports?: Maybe<Array<KubernetesContainerPortRequest>>;
};

export type KubernetesDeploymentComponent = {
  __typename?: "KubernetesDeploymentComponent";
  /** Component Constraints */
  constraints?: Maybe<KubernetesDeploymentComponentConstraints>;
  /** Component Description */
  description?: Maybe<Scalars["String"]>;
  /** Kubernetes Deployment Object Component Display Name */
  displayName?: Maybe<Scalars["String"]>;
  /** Kubernetes Deployment Object Component ID */
  id?: Maybe<Scalars["ID"]>;
  /** Kubernetes Deployment Object Component Name */
  name?: Maybe<Scalars["String"]>;
  /** SI Properties */
  siProperties?: Maybe<ComponentSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
};

export type KubernetesDeploymentComponentConstraints = {
  __typename?: "KubernetesDeploymentComponentConstraints";
  /** Component Display Name */
  componentDisplayName?: Maybe<Scalars["String"]>;
  /** Component Name */
  componentName?: Maybe<Scalars["String"]>;
  /** Kubernetes Version */
  kubernetesVersion?: Maybe<
    KubernetesDeploymentComponentConstraintsKubernetesVersion
  >;
};

/** Kubernetes Version */
export enum KubernetesDeploymentComponentConstraintsKubernetesVersion {
  Unknown = "UNKNOWN",
  V1_12 = "V1_12",
  V1_13 = "V1_13",
  V1_14 = "V1_14",
  V1_15 = "V1_15",
}

export type KubernetesDeploymentComponentConstraintsRequest = {
  /** Component Display Name */
  componentDisplayName?: Maybe<Scalars["String"]>;
  /** Component Name */
  componentName?: Maybe<Scalars["String"]>;
  /** Kubernetes Version */
  kubernetesVersion?: Maybe<
    KubernetesDeploymentComponentConstraintsKubernetesVersion
  >;
};

/** Get a Kubernetes Deployment Object Component Reply */
export type KubernetesDeploymentComponentGetReply = {
  __typename?: "KubernetesDeploymentComponentGetReply";
  /** Kubernetes Deployment Object Component Item */
  item?: Maybe<KubernetesDeploymentComponent>;
};

/** Get a Kubernetes Deployment Object Component Request */
export type KubernetesDeploymentComponentGetRequest = {
  /** Kubernetes Deployment Object Component ID */
  id: Scalars["ID"];
};

/** List Kubernetes Deployment Object Component Reply */
export type KubernetesDeploymentComponentListReply = {
  __typename?: "KubernetesDeploymentComponentListReply";
  /** Items */
  items?: Maybe<Array<KubernetesDeploymentComponent>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars["String"]>;
  /** Total Count */
  totalCount?: Maybe<Scalars["String"]>;
};

/** List Kubernetes Deployment Object Component Request */
export type KubernetesDeploymentComponentListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars["String"]>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars["String"]>;
  /** Page Token */
  pageToken?: Maybe<Scalars["String"]>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars["String"]>;
};

/** Pick Component Reply */
export type KubernetesDeploymentComponentPickReply = {
  __typename?: "KubernetesDeploymentComponentPickReply";
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
  __typename?: "KubernetesDeploymentEntity";
  associations?: Maybe<KubernetesDeploymentEntityAssociations>;
  /** Constraints */
  constraints?: Maybe<KubernetesDeploymentComponentConstraints>;
  /** Entity Description */
  description?: Maybe<Scalars["String"]>;
  /** Kubernetes Deployment Object Entity Display Name */
  displayName?: Maybe<Scalars["String"]>;
  /** Kubernetes Deployment Object Entity ID */
  id?: Maybe<Scalars["ID"]>;
  /** Implicit Constraints */
  implicitConstraints?: Maybe<KubernetesDeploymentComponentConstraints>;
  /** Kubernetes Deployment Object Entity Name */
  name?: Maybe<Scalars["String"]>;
  /** Properties */
  properties?: Maybe<KubernetesDeploymentEntityProperties>;
  /** SI Properties */
  siProperties?: Maybe<EntitySiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
};

/** Apply Reply */
export type KubernetesDeploymentEntityApplyReply = {
  __typename?: "KubernetesDeploymentEntityApplyReply";
  /** Entity Event */
  item?: Maybe<KubernetesDeploymentEntityEvent>;
};

/** Apply Request */
export type KubernetesDeploymentEntityApplyRequest = {
  /** Entity ID */
  id: Scalars["ID"];
};

/** Kubernetes Deployment Object Entity Associations */
export type KubernetesDeploymentEntityAssociations = {
  __typename?: "KubernetesDeploymentEntityAssociations";
  /** System Initiative Billing Account */
  billingAccount?: Maybe<BillingAccountGetReply>;
};

/** Create Entity Reply */
export type KubernetesDeploymentEntityCreateReply = {
  __typename?: "KubernetesDeploymentEntityCreateReply";
  /** kubernetesDeploymentEntity Item */
  item?: Maybe<KubernetesDeploymentEntity>;
};

/** Create Entity Request */
export type KubernetesDeploymentEntityCreateRequest = {
  /** Change Set ID */
  changeSetId: Scalars["String"];
  /** Constraints */
  constraints?: Maybe<KubernetesDeploymentComponentConstraintsRequest>;
  /** Description */
  description: Scalars["String"];
  /** Display Name */
  displayName: Scalars["String"];
  /** Name */
  name: Scalars["String"];
  /** Properties */
  properties?: Maybe<KubernetesDeploymentEntityPropertiesRequest>;
  /** Workspace ID */
  workspaceId: Scalars["String"];
};

/** Delete Entity Reply */
export type KubernetesDeploymentEntityDeleteReply = {
  __typename?: "KubernetesDeploymentEntityDeleteReply";
  /** kubernetesDeployment Item */
  item?: Maybe<KubernetesDeploymentEntity>;
};

/** Delete Entity Request */
export type KubernetesDeploymentEntityDeleteRequest = {
  /** Change Set ID */
  changeSetId: Scalars["String"];
  /** kubernetesDeploymentEntity ID */
  id: Scalars["ID"];
};

export type KubernetesDeploymentEntityEvent = {
  __typename?: "KubernetesDeploymentEntityEvent";
  /** Action Name */
  actionName?: Maybe<Scalars["String"]>;
  /** Creation Time */
  createTime?: Maybe<Scalars["String"]>;
  /** Error Lines */
  errorLines?: Maybe<Array<Scalars["String"]>>;
  /** Error Message */
  errorMessage?: Maybe<Scalars["String"]>;
  /** Finalized */
  finalized?: Maybe<Scalars["Boolean"]>;
  /** Final Time */
  finalTime?: Maybe<Scalars["String"]>;
  /** Kubernetes Deployment Object EntityEvent ID */
  id?: Maybe<Scalars["ID"]>;
  /** Input Entity */
  inputEntity?: Maybe<KubernetesDeploymentEntity>;
  /** Output Entity */
  outputEntity?: Maybe<KubernetesDeploymentEntity>;
  /** Output Lines */
  outputLines?: Maybe<Array<Scalars["String"]>>;
  /** Previous Entity */
  previousEntity?: Maybe<KubernetesDeploymentEntity>;
  /** SI Properties */
  siProperties?: Maybe<EntityEventSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
  /** success */
  success?: Maybe<Scalars["Boolean"]>;
  /** Updated Time */
  updatedTime?: Maybe<Scalars["String"]>;
  /** User ID */
  userId?: Maybe<Scalars["String"]>;
};

/** List Kubernetes Deployment Object EntityEvent Reply */
export type KubernetesDeploymentEntityEventListReply = {
  __typename?: "KubernetesDeploymentEntityEventListReply";
  /** Items */
  items?: Maybe<Array<KubernetesDeploymentEntityEvent>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars["String"]>;
  /** Total Count */
  totalCount?: Maybe<Scalars["String"]>;
};

/** List Kubernetes Deployment Object EntityEvent Request */
export type KubernetesDeploymentEntityEventListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars["String"]>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars["String"]>;
  /** Page Token */
  pageToken?: Maybe<Scalars["String"]>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars["String"]>;
};

/** Get a Kubernetes Deployment Object Entity Reply */
export type KubernetesDeploymentEntityGetReply = {
  __typename?: "KubernetesDeploymentEntityGetReply";
  /** Kubernetes Deployment Object Entity Item */
  item?: Maybe<KubernetesDeploymentEntity>;
};

/** Get a Kubernetes Deployment Object Entity Request */
export type KubernetesDeploymentEntityGetRequest = {
  /** Kubernetes Deployment Object Entity ID */
  id: Scalars["ID"];
};

/** Edit kubernetesDeploymentEntityPropertiesKubernetesObject Property Reply */
export type KubernetesDeploymentEntityKubernetesObjectEditReply = {
  __typename?: "KubernetesDeploymentEntityKubernetesObjectEditReply";
  /** Entity Event */
  item?: Maybe<KubernetesDeploymentEntityEvent>;
};

/** Edit kubernetesDeploymentEntityPropertiesKubernetesObject Property Request */
export type KubernetesDeploymentEntityKubernetesObjectEditRequest = {
  /** Entity ID */
  id: Scalars["ID"];
  /** The Kubernetes Object property value */
  property?: Maybe<KubernetesDeploymentEntityPropertiesKubernetesObjectRequest>;
};

/** Edit KubernetesObjectYaml Property Reply */
export type KubernetesDeploymentEntityKubernetesObjectYamlEditReply = {
  __typename?: "KubernetesDeploymentEntityKubernetesObjectYamlEditReply";
  /** Entity Event */
  item?: Maybe<KubernetesDeploymentEntityEvent>;
};

/** Edit KubernetesObjectYaml Property Request */
export type KubernetesDeploymentEntityKubernetesObjectYamlEditRequest = {
  /** Entity ID */
  id: Scalars["ID"];
  /** The Kubernetes Object YAML property value */
  property?: Maybe<Scalars["String"]>;
};

/** List Kubernetes Deployment Object Entity Reply */
export type KubernetesDeploymentEntityListReply = {
  __typename?: "KubernetesDeploymentEntityListReply";
  /** Items */
  items?: Maybe<Array<KubernetesDeploymentEntity>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars["String"]>;
  /** Total Count */
  totalCount?: Maybe<Scalars["String"]>;
};

/** List Kubernetes Deployment Object Entity Request */
export type KubernetesDeploymentEntityListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars["String"]>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars["String"]>;
  /** Page Token */
  pageToken?: Maybe<Scalars["String"]>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars["String"]>;
};

export type KubernetesDeploymentEntityProperties = {
  __typename?: "KubernetesDeploymentEntityProperties";
  /** Kubernetes Object */
  kubernetesObject?: Maybe<
    KubernetesDeploymentEntityPropertiesKubernetesObject
  >;
  /** Kubernetes Object YAML */
  kubernetesObjectYaml?: Maybe<Scalars["String"]>;
};

export type KubernetesDeploymentEntityPropertiesKubernetesObject = {
  __typename?: "KubernetesDeploymentEntityPropertiesKubernetesObject";
  /** API Version */
  apiVersion?: Maybe<Scalars["String"]>;
  /** Kind */
  kind?: Maybe<Scalars["String"]>;
  /** Metadata */
  metadata?: Maybe<KubernetesMetadata>;
  /** Deployment Spec */
  spec?: Maybe<KubernetesDeploymentEntityPropertiesKubernetesObjectSpec>;
};

export type KubernetesDeploymentEntityPropertiesKubernetesObjectRequest = {
  /** API Version */
  apiVersion: Scalars["String"];
  /** Kind */
  kind: Scalars["String"];
  /** Metadata */
  metadata?: Maybe<KubernetesMetadataRequest>;
  /** Deployment Spec */
  spec?: Maybe<KubernetesDeploymentEntityPropertiesKubernetesObjectSpecRequest>;
};

export type KubernetesDeploymentEntityPropertiesKubernetesObjectSpec = {
  __typename?: "KubernetesDeploymentEntityPropertiesKubernetesObjectSpec";
  /** Replicas */
  replicas?: Maybe<Scalars["Int"]>;
  /** Selector */
  selector?: Maybe<KubernetesSelector>;
  /** Pod Template Spec */
  template?: Maybe<KubernetesPodTemplateSpec>;
};

export type KubernetesDeploymentEntityPropertiesKubernetesObjectSpecRequest = {
  /** Replicas */
  replicas?: Maybe<Scalars["Int"]>;
  /** Selector */
  selector?: Maybe<KubernetesSelectorRequest>;
  /** Pod Template Spec */
  template?: Maybe<KubernetesPodTemplateSpecRequest>;
};

export type KubernetesDeploymentEntityPropertiesRequest = {
  /** Kubernetes Object */
  kubernetesObject?: Maybe<
    KubernetesDeploymentEntityPropertiesKubernetesObjectRequest
  >;
  /** Kubernetes Object YAML */
  kubernetesObjectYaml?: Maybe<Scalars["String"]>;
};

/** Sync State Reply */
export type KubernetesDeploymentEntitySyncReply = {
  __typename?: "KubernetesDeploymentEntitySyncReply";
  /** Entity Event */
  item?: Maybe<KubernetesDeploymentEntityEvent>;
};

/** Sync State Request */
export type KubernetesDeploymentEntitySyncRequest = {
  /** Entity ID */
  id: Scalars["ID"];
};

/** Update an Entity Reply */
export type KubernetesDeploymentEntityUpdateReply = {
  __typename?: "KubernetesDeploymentEntityUpdateReply";
  /** kubernetesDeployment Item */
  item?: Maybe<KubernetesDeploymentEntity>;
};

/** Update an Entity Request */
export type KubernetesDeploymentEntityUpdateRequest = {
  /** Change Set ID */
  changeSetId: Scalars["String"];
  /** kubernetesDeploymentEntity ID */
  id: Scalars["ID"];
  /** kubernetesDeployment Item Update */
  update?: Maybe<KubernetesDeploymentEntityUpdateRequestUpdateRequest>;
};

export type KubernetesDeploymentEntityUpdateRequestUpdateRequest = {
  /** description */
  description?: Maybe<Scalars["String"]>;
  /** name */
  name?: Maybe<Scalars["String"]>;
  /** properties */
  properties?: Maybe<KubernetesDeploymentEntityPropertiesRequest>;
};

export type KubernetesMetadata = {
  __typename?: "KubernetesMetadata";
  /** Labels */
  labels?: Maybe<Array<Labels>>;
  /** Name */
  name?: Maybe<Scalars["String"]>;
};

export type KubernetesMetadataRequest = {
  /** Labels */
  labels?: Maybe<Array<LabelsRequest>>;
  /** Name */
  name: Scalars["String"];
};

export type KubernetesPodSpec = {
  __typename?: "KubernetesPodSpec";
  /** Containers */
  containers?: Maybe<Array<KubernetesContainer>>;
};

export type KubernetesPodSpecRequest = {
  /** Containers */
  containers?: Maybe<Array<KubernetesContainerRequest>>;
};

export type KubernetesPodTemplateSpec = {
  __typename?: "KubernetesPodTemplateSpec";
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
  __typename?: "KubernetesSelector";
  /** Match Labels */
  matchLabels?: Maybe<Array<MatchLabels>>;
};

export type KubernetesSelectorRequest = {
  /** Match Labels */
  matchLabels?: Maybe<Array<MatchLabelsRequest>>;
};

/** Labels */
export type Labels = {
  __typename?: "Labels";
  key?: Maybe<Scalars["String"]>;
  value?: Maybe<Scalars["String"]>;
};

/** Labels */
export type LabelsRequest = {
  key?: Maybe<Scalars["String"]>;
  value?: Maybe<Scalars["String"]>;
};

/** Match Labels */
export type MatchLabels = {
  __typename?: "MatchLabels";
  key?: Maybe<Scalars["String"]>;
  value?: Maybe<Scalars["String"]>;
};

/** Match Labels */
export type MatchLabelsRequest = {
  key?: Maybe<Scalars["String"]>;
  value?: Maybe<Scalars["String"]>;
};

export type Mutation = {
  __typename?: "Mutation";
  billingAccountSignup?: Maybe<BillingAccountSignupReply>;
  changeSetCreate?: Maybe<ChangeSetCreateReply>;
  changeSetExecute?: Maybe<ChangeSetExecuteReply>;
  groupCreate?: Maybe<GroupCreateReply>;
  kubernetesDeploymentEntityApply?: Maybe<KubernetesDeploymentEntityApplyReply>;
  kubernetesDeploymentEntityCreate?: Maybe<
    KubernetesDeploymentEntityCreateReply
  >;
  kubernetesDeploymentEntityDelete?: Maybe<
    KubernetesDeploymentEntityDeleteReply
  >;
  kubernetesDeploymentEntityKubernetesObjectEdit?: Maybe<
    KubernetesDeploymentEntityKubernetesObjectEditReply
  >;
  kubernetesDeploymentEntityKubernetesObjectYamlEdit?: Maybe<
    KubernetesDeploymentEntityKubernetesObjectYamlEditReply
  >;
  kubernetesDeploymentEntitySync?: Maybe<KubernetesDeploymentEntitySyncReply>;
  kubernetesDeploymentEntityUpdate?: Maybe<
    KubernetesDeploymentEntityUpdateReply
  >;
  organizationCreate?: Maybe<OrganizationCreateReply>;
  userCreate?: Maybe<UserCreateReply>;
  workspaceCreate?: Maybe<WorkspaceCreateReply>;
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

export type MutationOrganizationCreateArgs = {
  input?: Maybe<OrganizationCreateRequest>;
};

export type MutationUserCreateArgs = {
  input?: Maybe<UserCreateRequest>;
};

export type MutationWorkspaceCreateArgs = {
  input?: Maybe<WorkspaceCreateRequest>;
};

export type Organization = {
  __typename?: "Organization";
  associations?: Maybe<OrganizationAssociations>;
  /** A System Initiative Organization Display Name */
  displayName?: Maybe<Scalars["String"]>;
  /** A System Initiative Organization ID */
  id?: Maybe<Scalars["ID"]>;
  /** A System Initiative Organization Name */
  name?: Maybe<Scalars["String"]>;
  /** SI Internal Properties */
  siProperties?: Maybe<OrganizationSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
};

/** A System Initiative Organization Associations */
export type OrganizationAssociations = {
  __typename?: "OrganizationAssociations";
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
  __typename?: "OrganizationCreateReply";
  /** A System Initiative Organization Item */
  item?: Maybe<Organization>;
};

/** Create an Organization Request */
export type OrganizationCreateRequest = {
  /** User Display Name */
  displayName: Scalars["String"];
  /** User Name */
  name: Scalars["String"];
  /** The SI Properties for this User */
  siProperties: OrganizationSiPropertiesRequest;
};

/** Get a A System Initiative Organization Reply */
export type OrganizationGetReply = {
  __typename?: "OrganizationGetReply";
  /** A System Initiative Organization Item */
  item?: Maybe<Organization>;
};

/** Get a A System Initiative Organization Request */
export type OrganizationGetRequest = {
  /** A System Initiative Organization ID */
  id: Scalars["ID"];
};

/** List A System Initiative Organization Reply */
export type OrganizationListReply = {
  __typename?: "OrganizationListReply";
  /** Items */
  items?: Maybe<Array<Organization>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars["String"]>;
  /** Total Count */
  totalCount?: Maybe<Scalars["String"]>;
};

/** List A System Initiative Organization Request */
export type OrganizationListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars["String"]>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars["String"]>;
  /** Page Token */
  pageToken?: Maybe<Scalars["String"]>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars["String"]>;
};

export type OrganizationSiProperties = {
  __typename?: "OrganizationSiProperties";
  /** Billing Account ID */
  billingAccountId?: Maybe<Scalars["String"]>;
};

export type OrganizationSiPropertiesRequest = {
  /** Billing Account ID */
  billingAccountId: Scalars["String"];
};

export type Query = {
  __typename?: "Query";
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
  kubernetesDeploymentComponentGet?: Maybe<
    KubernetesDeploymentComponentGetReply
  >;
  kubernetesDeploymentComponentList?: Maybe<
    KubernetesDeploymentComponentListReply
  >;
  kubernetesDeploymentComponentPick?: Maybe<
    KubernetesDeploymentComponentPickReply
  >;
  kubernetesDeploymentEntityEventList?: Maybe<
    KubernetesDeploymentEntityEventListReply
  >;
  kubernetesDeploymentEntityGet?: Maybe<KubernetesDeploymentEntityGetReply>;
  kubernetesDeploymentEntityList?: Maybe<KubernetesDeploymentEntityListReply>;
  organizationGet?: Maybe<OrganizationGetReply>;
  organizationList?: Maybe<OrganizationListReply>;
  userGet?: Maybe<UserGetReply>;
  userList?: Maybe<UserListReply>;
  userLogin?: Maybe<UserLoginReply>;
  workspaceGet?: Maybe<WorkspaceGetReply>;
  workspaceList?: Maybe<WorkspaceListReply>;
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

export type QueryOrganizationGetArgs = {
  input?: Maybe<OrganizationGetRequest>;
};

export type QueryOrganizationListArgs = {
  input?: Maybe<OrganizationListRequest>;
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

export type User = {
  __typename?: "User";
  associations?: Maybe<UserAssociations>;
  /** Authorized capabilities for this user */
  capabilities?: Maybe<Capability>;
  /** A System Initiative User Display Name */
  displayName?: Maybe<Scalars["String"]>;
  /** A valid email address */
  email?: Maybe<Scalars["String"]>;
  /** A System Initiative User ID */
  id?: Maybe<Scalars["ID"]>;
  /** A System Initiative User Name */
  name?: Maybe<Scalars["String"]>;
  /** SI Internal Properties */
  siProperties?: Maybe<UserSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
};

/** A System Initiative User Associations */
export type UserAssociations = {
  __typename?: "UserAssociations";
  /** System Initiative Billing Account */
  billingAccount?: Maybe<BillingAccountGetReply>;
};

/** Create a User Reply */
export type UserCreateReply = {
  __typename?: "UserCreateReply";
  /** A System Initiative User Item */
  item?: Maybe<User>;
};

/** Create a User Request */
export type UserCreateRequest = {
  /** User Display Name */
  displayName: Scalars["String"];
  /** Users email address */
  email: Scalars["String"];
  /** User Name */
  name: Scalars["String"];
  /** Users password */
  password: Scalars["String"];
  /** The SI Properties for this User */
  siProperties: UserSiPropertiesRequest;
};

/** Get a A System Initiative User Reply */
export type UserGetReply = {
  __typename?: "UserGetReply";
  /** A System Initiative User Item */
  item?: Maybe<User>;
};

/** Get a A System Initiative User Request */
export type UserGetRequest = {
  /** A System Initiative User ID */
  id: Scalars["ID"];
};

/** List A System Initiative User Reply */
export type UserListReply = {
  __typename?: "UserListReply";
  /** Items */
  items?: Maybe<Array<User>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars["String"]>;
  /** Total Count */
  totalCount?: Maybe<Scalars["String"]>;
};

/** List A System Initiative User Request */
export type UserListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars["String"]>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars["String"]>;
  /** Page Token */
  pageToken?: Maybe<Scalars["String"]>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars["String"]>;
};

export type UserLoginReply = {
  __typename?: "UserLoginReply";
  billingAccountId?: Maybe<Scalars["String"]>;
  jwt?: Maybe<Scalars["String"]>;
  userId?: Maybe<Scalars["String"]>;
};

export type UserLoginRequest = {
  billingAccountName: Scalars["String"];
  email: Scalars["String"];
  password: Scalars["String"];
};

export type UserSiProperties = {
  __typename?: "UserSiProperties";
  /** Billing Account ID */
  billingAccountId?: Maybe<Scalars["String"]>;
};

export type UserSiPropertiesRequest = {
  /** Billing Account ID */
  billingAccountId: Scalars["String"];
};

export type Workspace = {
  __typename?: "Workspace";
  associations?: Maybe<WorkspaceAssociations>;
  /** A System Initiative Workspace Display Name */
  displayName?: Maybe<Scalars["String"]>;
  /** A System Initiative Workspace ID */
  id?: Maybe<Scalars["ID"]>;
  /** A System Initiative Workspace Name */
  name?: Maybe<Scalars["String"]>;
  /** SI Internal Properties */
  siProperties?: Maybe<WorkspaceSiProperties>;
  /** SI Storable */
  siStorable?: Maybe<DataStorable>;
};

/** A System Initiative Workspace Associations */
export type WorkspaceAssociations = {
  __typename?: "WorkspaceAssociations";
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
  __typename?: "WorkspaceCreateReply";
  /** A System Initiative Workspace Item */
  item?: Maybe<Workspace>;
};

/** Create an Organization Request */
export type WorkspaceCreateRequest = {
  /** User Display Name */
  displayName: Scalars["String"];
  /** User Name */
  name: Scalars["String"];
  /** The SI Properties for this User */
  siProperties: WorkspaceSiPropertiesRequest;
};

/** Get a A System Initiative Workspace Reply */
export type WorkspaceGetReply = {
  __typename?: "WorkspaceGetReply";
  /** A System Initiative Workspace Item */
  item?: Maybe<Workspace>;
};

/** Get a A System Initiative Workspace Request */
export type WorkspaceGetRequest = {
  /** A System Initiative Workspace ID */
  id: Scalars["ID"];
};

/** List A System Initiative Workspace Reply */
export type WorkspaceListReply = {
  __typename?: "WorkspaceListReply";
  /** Items */
  items?: Maybe<Array<Workspace>>;
  /** Next Page Token */
  nextPageToken?: Maybe<Scalars["String"]>;
  /** Total Count */
  totalCount?: Maybe<Scalars["String"]>;
};

/** List A System Initiative Workspace Request */
export type WorkspaceListRequest = {
  /** Order By */
  orderBy?: Maybe<Scalars["String"]>;
  /** Order By Direction */
  orderByDirection?: Maybe<DataPageTokenOrderByDirection>;
  /** Page Size */
  pageSize?: Maybe<Scalars["String"]>;
  /** Page Token */
  pageToken?: Maybe<Scalars["String"]>;
  /** Query */
  query?: Maybe<DataQueryRequest>;
  /** Scope By Tenant ID */
  scopeByTenantId?: Maybe<Scalars["String"]>;
};

export type WorkspaceSiProperties = {
  __typename?: "WorkspaceSiProperties";
  /** Billing Account ID */
  billingAccountId?: Maybe<Scalars["String"]>;
  /** Organization ID */
  organizationId?: Maybe<Scalars["String"]>;
};

export type WorkspaceSiPropertiesRequest = {
  /** Billing Account ID */
  billingAccountId: Scalars["String"];
  /** Organization ID */
  organizationId: Scalars["String"];
};
