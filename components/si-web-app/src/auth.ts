import { Query, BillingAccount, User, Workspace } from "@/graphql-types";
import * as jwtLib from "jsonwebtoken";
import { onLogin, onLogout, ExtendedApolloClient } from "@/vue-apollo";
import { ApolloQueryResult } from "apollo-client";
import VueApollo from "vue-apollo";

import { registry } from "si-registry";

/**
 * BillingAccountList handles the list of BillingAccounts that
 * have ever been logged in to by this browser.
 *
 * This list is stored in localStorage - so it will live forever
 * as data the browser carries around for our website.
 **/
class BillingAccountList {
  public getAccounts(): BillingAccount[] {
    let billingAccountsJson = localStorage.getItem("billingAccounts");
    if (billingAccountsJson) {
      let billingAccounts = JSON.parse(billingAccountsJson) as BillingAccount[];
      return billingAccounts;
    } else {
      return [];
    }
  }

  public getFirstAccountName(): string {
    const accounts = this.getAccounts();
    if (accounts.length == 0 || accounts[0].name == undefined) {
      return "";
    } else {
      return accounts[0].name;
    }
  }

  public addAccount(billingAccount: BillingAccount) {
    const accounts = this.getAccounts();
    if (accounts.length == 0) {
      accounts.push(billingAccount);
    } else {
      for (const account of accounts) {
        if (account.id == billingAccount.id) {
          return;
        }
      }
      accounts.push(billingAccount);
    }
    localStorage.setItem("billingAccounts", JSON.stringify(accounts));
  }
}

export const billingAccountList = new BillingAccountList();

interface ProfileConstructor {
  user: User;
  billingAccount: BillingAccount;
  workspaces: Workspace[];
}

class Profile {
  user: User;
  billingAccount: BillingAccount;
  workspaces: Workspace[];
  workspaceDefault: Workspace;

  constructor(args: ProfileConstructor) {
    this.user = args.user;
    this.billingAccount = args.billingAccount;
    this.workspaces = args.workspaces;
    this.workspaceDefault = args.workspaces[0];
  }
}

class Authentication {
  loggedIn: boolean = false;
  profile: Profile | undefined;
  apollo: null | VueApollo = null;

  setApollo(apolloProvider: VueApollo): void {
    this.apollo = apolloProvider;
  }

  async isAuthenticated(): Promise<boolean> {
    let apolloToken = localStorage.getItem("apollo-token");
    if (apolloToken && this.apollo) {
      let currentTime = Math.floor(Date.now() / 1000);
      let decodedToken = jwtLib.decode(apolloToken, { complete: true }) as any;
      if (decodedToken && currentTime >= decodedToken["payload"]["exp"]) {
        await this.logout();
        return false;
      }
      // If this is false, it means we have an apolloToken, but we aren't actually
      // in the right state. Rehydrate.
      if (this.loggedIn == false) {
        let profileJson = localStorage.getItem("profile");
        if (profileJson) {
          let profile = JSON.parse(profileJson) as Profile;
          this.profile = profile;
          this.loggedIn = true;
        }
      }
    }
    return this.loggedIn;
  }

  async login(jwt: string, userId: string): Promise<void> {
    if (this.apollo) {
      await onLogin(this.apollo.defaultClient as ExtendedApolloClient, jwt);
    } else {
      throw "Authentication not initialized";
    }
    let client = this.apollo.defaultClient;
    let user = registry.get("user");
    let userQuery = user.graphql.query({
      methodName: "get",
      associations: {
        user: ["billingAccount"],
        billingAccount: ["organizations"],
        organization: ["workspaces"],
      },
    });
    const userReply: ApolloQueryResult<Query> = await client.query({
      query: userQuery,
      variables: { id: userId },
    });
    const data = user.graphql.validateResult({
      methodName: "get",
      data: userReply,
    });
    this.profile = new Profile({
      user: data.item,
      billingAccount: data.item.associations.billingAccount.item,
      workspaces:
        data.item.associations.billingAccount.item.associations.organizations
          .items[0].associations.workspaces.items,
    });
    this.loggedIn = true;
    localStorage.setItem("profile", JSON.stringify(this.profile));
  }

  async logout(): Promise<void> {
    this.profile = undefined;
    this.loggedIn = false;
    if (this.apollo) {
      await onLogout(this.apollo.defaultClient as ExtendedApolloClient);
    }
    localStorage.removeItem("profile");
  }

  getProfile(): Profile {
    if (this.profile == undefined) {
      throw "Cannot get profile; user is not logged in!";
    }
    return this.profile;
  }

  // - Alex -
  // Property 'workspaceDefault' does not exist on type 'User'.
  //
  getCurrentWorkspace(): Workspace {
    return this.getProfile().workspaceDefault;
  }
}

export const auth = new Authentication();
