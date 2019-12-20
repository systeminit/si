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
  /** shortName */
  shortName?: Maybe<Scalars["String"]>;
  /** tenantIds */
  tenantIds?: Maybe<Array<Scalars["String"]>>;
  /** typeName */
  typeName?: Maybe<Scalars["String"]>;
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

export type GetBillingAccountReply = {
  __typename?: "GetBillingAccountReply";
  /** billingAccount */
  billingAccount?: Maybe<BillingAccount>;
};

export type GetBillingAccountRequest = {
  /** billingAccountId */
  billingAccountId?: Maybe<Scalars["String"]>;
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

export type Query = {
  __typename?: "Query";
  getBillingAccount?: Maybe<GetBillingAccountReply>;
  getUser?: Maybe<GetUserReply>;
  login?: Maybe<LoginReply>;
};

export type QueryGetBillingAccountArgs = {
  input?: Maybe<GetBillingAccountRequest>;
};

export type QueryGetUserArgs = {
  input?: Maybe<GetUserRequest>;
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
              >
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
