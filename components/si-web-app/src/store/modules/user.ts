import { Module } from "vuex";
import {
  Query,
  BillingAccount,
  User,
  Workspace,
  Organization,
} from "@/types/graphql-types";
import * as jwtLib from "jsonwebtoken";
import { onLogin, onLogout, apollo } from "@/api/apollo";
import { ApolloQueryResult } from "apollo-client";

import { registry } from "si-registry";

interface ProfileConstructor {
  user: User;
  billingAccount: BillingAccount;
  organization: Organization;
  workspaces: Workspace[];
}

class Profile {
  user: User;
  billingAccount: BillingAccount;
  organization: Organization;
  workspaces: Workspace[];
  workspaceDefault: Workspace;

  constructor(args: ProfileConstructor) {
    this.user = args.user;
    this.billingAccount = args.billingAccount;
    this.workspaces = args.workspaces;
    this.organization = args.organization;
    this.workspaceDefault = args.workspaces[0];
  }
}

export class Authentication {
  loggedIn: boolean = false;
  profile: Profile | undefined;
}

export interface UserStore {
  auth: Authentication;
}

export const auth = new Authentication();

export const user: Module<UserStore, any> = {
  namespaced: true,
  state: {
    auth,
  },
  mutations: {
    loggedIn(state, loginValue: boolean) {
      state.auth.loggedIn = loginValue;
    },
    profile(state, profileValue: Profile | undefined) {
      state.auth.profile = profileValue;
    },
  },
  getters: {
    profile(state): Profile {
      if (state.auth.profile == undefined) {
        throw new Error("Cannot get profile; user is not logged in!");
      }
      return state.auth.profile;
    },
    userId(state): string {
      if (state.auth.profile?.user.id == undefined) {
        throw new Error("Cannot get userId; user is not logged in!");
      }
      return state.auth.profile.user.id;
    },
    currentWorkspace(state): Workspace {
      if (state.auth.profile == undefined) {
        throw new Error(
          "Cannot get profile; user is not logged in. So cannot get the current workspace, either!",
        );
      }
      return state.auth.profile.workspaceDefault;
    },
    currentWorkspaceId(state): string {
      if (state.auth.profile?.workspaceDefault?.id == undefined) {
        throw new Error(
          "Cannot get profile; user is not logged in. So cannot get the current workspace, either!",
        );
      }
      return state.auth.profile.workspaceDefault.id;
    },
  },
  actions: {
    async isAuthenticated({ dispatch, state, commit }): Promise<boolean> {
      let apolloToken = localStorage.getItem("apollo-token");
      if (apolloToken) {
        let currentTime = Math.floor(Date.now() / 1000);
        let decodedToken = jwtLib.decode(apolloToken, {
          complete: true,
        }) as any;
        if (decodedToken && currentTime >= decodedToken["payload"]["exp"]) {
          await dispatch("logout");
          return false;
        }
        // If this is false, it means we have an apolloToken, but we aren't actually
        // in the right state. Rehydrate.
        if (state.auth.loggedIn == false) {
          let profileJson = localStorage.getItem("profile");
          if (profileJson) {
            let profile = JSON.parse(profileJson) as Profile;
            commit("profile", profile);
            commit("loggedIn", true);
          }
        }
      }
      return true;
    },

    async login(
      { commit, dispatch },
      {
        email,
        password,
        billingAccountName,
      }: { email: string; password: string; billingAccountName: string },
    ): Promise<void> {
      // Log in the user
      const userRegistry = registry.get("user");
      const loginResult = await apollo.query({
        query: userRegistry.graphql.query({
          methodName: "loginInternal",
          overrideName: "userLogin",
          overrideFields: "jwt, userId, billingAccountId",
        }),
        variables: { email, password, billingAccountName },
      });
      const loginData = loginResult.data.userLogin;

      await onLogin(loginData.jwt);

      // Populate their profile
      let userQuery = userRegistry.graphql.query({
        methodName: "get",
        associations: {
          user: ["billingAccount"],
          billingAccount: ["organizations"],
          organization: ["workspaces"],
        },
      });
      const userReply: ApolloQueryResult<Query> = await apollo.query({
        query: userQuery,
        variables: { id: loginData.userId },
      });
      const data = userRegistry.graphql.validateResult({
        methodName: "get",
        data: userReply,
      });
      let profile = new Profile({
        user: data.item,
        billingAccount: data.item.associations.billingAccount.item,
        organization:
          data.item.associations.billingAccount.item.associations.organizations
            .items[0],
        workspaces:
          data.item.associations.billingAccount.item.associations.organizations
            .items[0].associations.workspaces.items,
      });
      commit("profile", profile);
      commit("loggedIn", true);
      dispatch("entity/load", { root: true });
      localStorage.setItem("profile", JSON.stringify(profile));
    },

    async logout({ commit }): Promise<void> {
      commit("profile", undefined);
      commit("loggedIn", false);
      await onLogout();
      localStorage.removeItem("profile");
    },
  },
};
