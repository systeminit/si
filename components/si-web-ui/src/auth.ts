import { Query, BillingAccount, User, Workspace } from "@/graphql-types";
import * as jwtLib from "jsonwebtoken";
import { onLogin, onLogout, ExtendedApolloClient } from "@/vue-apollo";
import { ApolloQueryResult } from "apollo-client";
import VueApollo from "vue-apollo";

import getProfile from "@/graphql/queries/getProfile.gql";

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

  public getFirstAccountShortName(): string {
    const accounts = this.getAccounts();
    if (accounts.length == 0) {
      return "";
    } else {
      return accounts[0].shortName ? accounts[0].shortName : "";
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

class Authentication {
  loggedIn: boolean = false;
  profile: User = {};
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
          let user = JSON.parse(profileJson) as User;
          this.profile = user;
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
    let userReply: ApolloQueryResult<Query> = await client.query({
      query: getProfile,
      variables: { userId },
    });
    if (
      userReply.data &&
      userReply.data.getUser &&
      userReply.data.getUser.user
    ) {
      this.profile = userReply.data.getUser.user;
      this.loggedIn = true;
      localStorage.setItem("profile", JSON.stringify(this.profile));
    } else {
      throw "User query did not return data for the user profile";
    }
  }

  async logout(): Promise<void> {
    this.profile = {};
    this.loggedIn = false;
    if (this.apollo) {
      await onLogout(this.apollo.defaultClient as ExtendedApolloClient);
    }
    localStorage.removeItem("profile");
  }

  getProfile(): User {
    return this.profile;
  }

  getCurrentWorkspace(): Workspace {
    const organization = (this.profile &&
      this.profile.billingAccount &&
      this.profile.billingAccount.organizations &&
      this.profile.billingAccount.organizations.items &&
      this.profile.billingAccount.organizations.items[0]) || { name: "busted" };
    const workspace = (organization &&
      organization.workspaces &&
      organization.workspaces.items &&
      organization.workspaces.items[0]) || { name: "busted" };
    return workspace;
  }
}

export const auth = new Authentication();
