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
  Equals = "EQUALS",
  Notequals = "NOTEQUALS",
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
};

export type MutationCreateAccountArgs = {
  input?: Maybe<CreateAccountRequest>;
};

export type Organization = {
  __typename?: "Organization";
  /** billingAccountId */
  billingAccount?: Maybe<BillingAccount>;
  /** billingAccountId */
  billingAccountId?: Maybe<Scalars["String"]>;
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

export type OrganizationWorkspacesArgs = {
  input?: Maybe<ListWorkspacesRequest>;
};

export type Query = {
  __typename?: "Query";
  getBillingAccount?: Maybe<GetBillingAccountReply>;
  getUser?: Maybe<GetUserReply>;
  listOrganizations?: Maybe<ListOrganizationsReply>;
  listUsers?: Maybe<ListUsersReply>;
  listWorkspaces?: Maybe<ListWorkspacesReply>;
  login?: Maybe<LoginReply>;
};

export type QueryGetBillingAccountArgs = {
  input?: Maybe<GetBillingAccountRequest>;
};

export type QueryGetUserArgs = {
  input?: Maybe<GetUserRequest>;
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
