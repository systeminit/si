import { defineStore } from "pinia";
import storage from "local-storage-fallback"; // drop-in storage polyfill which falls back to cookies/memory
import * as _ from "lodash-es";
import jwtDecode from "jwt-decode";
import { ApiRequest } from "@si/vue-lib/pinia";
import { posthog } from "@/utils/posthog";
import { User } from "@/api/sdf/dal/user";
import { Workspace } from "@/api/sdf/dal/workspace";
import { useWorkspacesStore } from "./workspaces.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import { AuthApiRequest } from ".";

export type UserId = string;

const AUTH_PORTAL_URL = import.meta.env.VITE_AUTH_PORTAL_URL;

// keys we use to store auth tokens in local storage
const AUTH_LOCAL_STORAGE_KEYS = {
  USER_TOKENS: "si-auth",
};

// Cookie name for tracking accessed workspaces across subdomains
const WORKSPACE_TRACKING_COOKIE = "si-workspaces";

// Utility functions for managing workspace tracking cookie
function getAccessedWorkspaces(): string[] {
  const cookie = document.cookie
    .split("; ")
    .find((row) => row.startsWith(`${WORKSPACE_TRACKING_COOKIE}=`));
  if (!cookie) return [];
  const value = cookie.split("=")[1];
  if (!value) return [];
  try {
    return JSON.parse(decodeURIComponent(value));
  } catch {
    return [];
  }
}

function addAccessedWorkspace(workspaceId: string) {
  const workspaces = getAccessedWorkspaces();
  if (!workspaces.includes(workspaceId)) {
    workspaces.push(workspaceId);
  }
  // Set cookie with domain=.systeminit.com so it's accessible across subdomains
  // Extract parent domain from current hostname (e.g., app.systeminit.com -> .systeminit.com)
  const hostname = window.location.hostname;
  const parts = hostname.split(".");
  const domain = parts.length >= 2 ? `.${parts.slice(-2).join(".")}` : hostname;

  document.cookie = `${WORKSPACE_TRACKING_COOKIE}=${encodeURIComponent(JSON.stringify(workspaces))}; domain=${domain}; path=/; max-age=31536000; SameSite=Lax`;
}

// V1 token format (legacy)
type TokenDataV1 = {
  user_pk: string;
  workspace_pk: string;
};

// V2 token format (new secure tokens)
type TokenDataV2 = {
  version: "2";
  userId: string;
  workspaceId: string;
  role: string;
  jti: string;
};

type TokenData = TokenDataV1 | TokenDataV2;

interface LoginResponse {
  user: User;
  workspace: Workspace;
  token: string;
  userWorkspaceFlags: Record<string, boolean>;
}

export interface WorkspaceUser {
  id: string;
  name: string;
  email: string;
}

// Helper function to normalize token data from V1 or V2 format
function normalizeTokenData(token: TokenData): {
  userPk: string;
  workspacePk: string;
} {
  if ("version" in token && token.version === "2") {
    // V2 token format
    return {
      userPk: token.userId,
      workspacePk: token.workspaceId,
    };
  }

  // V1 token format (explicit cast since TypeScript doesn't narrow automatically)
  const v1Token = token as TokenDataV1;
  return {
    userPk: v1Token.user_pk,
    workspacePk: v1Token.workspace_pk,
  };
}

export const useAuthStore = () => {
  const WORKSPACE_API_PREFIX = ["v2", "workspaces"];

  const realtimeStore = useRealtimeStore();

  return defineStore("auth", {
    state: () => ({
      tokens: {} as Record<string, string>,
      userPk: null as string | null,

      user: null as User | null,
      // TODO - Users will not be in this list if they have NEVER logged into the workspace
      workspaceUsers: {} as Record<string, WorkspaceUser>,
      userWorkspaceFlags: {} as Record<string, boolean>,
    }),
    getters: {
      // previously we checked only for the token existing
      // but when the DB is reset, the token is still set but the backend DB is empty
      // so we must wait for the backend to be re-initialized
      userIsLoggedIn: (state) => !_.isEmpty(state.tokens),
      userIsLoggedInAndInitialized: (state) => !_.isEmpty(state.tokens) && state.user?.pk,
      selectedWorkspaceToken: (state) => {
        const workspacesStore = useWorkspacesStore();
        if (workspacesStore.urlSelectedWorkspaceId) {
          // this case works in most scenarios except if we are asking for the
          // selectedWorkspaceToken before useRouterStore is ready (like on page refresh)
          return state.tokens[workspacesStore.urlSelectedWorkspaceId];
        } else {
          // make sure that if we have a selected workspace token we populate it properly
          // even if the workspaces store does not have the urlSelectedWorkspaceId ready yet
          const path = window.location.pathname;

          if (path.includes("auth-connect")) {
            // this case handles when we are arriving from the /go endpoint of the auth api
            // currently just parsing the string manually to avoid using URLSearchParams
            // TODO(Wendy) - replace this with URLSearchParams when we can
            const queryString = window.location.search.replace("?", ""); // grab the whole query string, removing the starting ?
            const queryParts = queryString.split("&"); // split each of the individual parts
            const workspacePartString = queryParts.find((part) => part.includes("workspaceId=")); // find the part we care about
            if (workspacePartString) {
              const workspaceId = workspacePartString.replace("workspaceId=", ""); // strip out unnecessary data
              return state.tokens[workspaceId];
            }
          } else if (path.startsWith("/n/") || path.startsWith("/w/")) {
            // this case attempts to handle standard workspace urls for the old and new ui
            const pathParts = path.split("/");
            const workspaceId = pathParts[2];
            if (workspaceId) {
              return state.tokens[workspaceId];
            }
          }
        }
        // we don't have a token!
        return undefined;
      },
      selectedOrDefaultAuthToken(): string | undefined {
        // console.log("TOKEN: ", this.selectedWorkspaceToken);
        return this.selectedWorkspaceToken || _.values(this.tokens)[0];
      },
      workspaceHasOneUser(): boolean {
        return Object.keys(this.workspaceUsers).length === 1;
      },
    },
    actions: {
      // NOTE(nick): this probably needs a new home for users eventually.
      async LIST_WORKSPACE_USERS(workspaceId: string) {
        return new ApiRequest<{ users: WorkspaceUser[] }>({
          method: "get",
          url: WORKSPACE_API_PREFIX.concat([workspaceId, "users"]),
          onSuccess: (response) => {
            this.workspaceUsers = {};
            response.users.forEach((u) => {
              this.workspaceUsers[u.id] = u;
            });
          },
        });
      },

      // fetches user + workspace info from SDF - called on page refresh
      async RESTORE_AUTH() {
        return new ApiRequest<Omit<LoginResponse, "jwt">>({
          url: "/session/restore_authentication",
          onSuccess: (response) => {
            this.user = response.user;
            // Currently restore auth is not loading the correct workspace
            this.userWorkspaceFlags = response.userWorkspaceFlags;
          },
          onFail(e) {
            /* eslint-disable-next-line no-console */
            console.log("RESTORE AUTH FAILED!", e);
          },
        });
      },

      // exchanges a code from the auth portal/api to auth with sdf
      // and initializes workspace/user if necessary
      async AUTH_CONNECT(payload: { code: string; onDemandAssets: boolean }) {
        return new ApiRequest<LoginResponse, { code: string; onDemandAssets: boolean }>({
          method: "post",
          url: "/session/connect",
          params: { ...payload },
          onSuccess: (response) => {
            this.finishUserLogin(response);
          },
          onFail: (response) => {
            const errMessage = response?.error?.message || "";
            if (errMessage.includes("relation") && errMessage.includes("does not exist")) {
              /* eslint-disable no-console, no-alert */
              console.log("db needs migrations");
              // TODO: probably show a better error than an alert
              alert("Looks like your database needs migrations - please restart SDF");
            }
          },
        });
      },

      async CHECK_FIRST_MODAL(userPk: string) {
        return new AuthApiRequest<boolean>({
          url: ["users", { userPk }, "firstTimeModal"],
        });
      },

      async DISMISS_FIRST_TIME_MODAL(userPk: string) {
        return new AuthApiRequest<boolean>({
          method: "post",
          url: ["users", { userPk }, "dismissFirstTimeModal"],
        });
      },

      // uses existing auth token (jwt) to re-fetch and initialize workspace/user from auth api
      // this is needed if user is still logged inbut the running SI instance DB is empty
      // for example after updating containers via the launcher
      async AUTH_RECONNECT() {
        return new ApiRequest<Omit<LoginResponse, "jwt">>({
          url: "/session/reconnect",
          onSuccess: (response) => {
            this.user = response.user;
            this.userWorkspaceFlags = response.userWorkspaceFlags;
          },
          onFail(e) {
            console.log("AUTH RECONNECT FAILED!", e);
            // trigger logout?
          },
        });
      },

      initTokens() {
        let tokensByWorkspacePk: Record<string, string> = {};
        try {
          const parsed = JSON.parse(storage.getItem(AUTH_LOCAL_STORAGE_KEYS.USER_TOKENS) || "{}");
          tokensByWorkspacePk = parsed;
        } catch {
          /* empty */
        }

        const tokens = _.values(tokensByWorkspacePk);
        if (!tokens.length) return [];

        // token contains user pk and workspace pk (normalize V1/V2 format)

        const decodedToken = jwtDecode<TokenData>(tokens[0]!);
        const { userPk } = normalizeTokenData(decodedToken);

        this.$patch({
          tokens: tokensByWorkspacePk,
          userPk,
        });

        // Track all workspace IDs in shared cookie for logout
        Object.keys(tokensByWorkspacePk).forEach((workspaceId) => {
          addAccessedWorkspace(workspaceId);
        });

        return tokens;
      },

      // OTHER ACTIONS ///////////////////////////////////////////////////////////////////
      async initFromStorage() {
        // check regular user token (we will likely have a different token for admin auth later)
        const tokens = this.initTokens();
        if (!tokens.length) return;

        // this endpoint re-fetches the user and workspace
        // dont think it's 100% necessary at the moment and not quite the right shape, but can fix later
        const restoreAuthReq = await this.RESTORE_AUTH();
        if (!restoreAuthReq.result.success) {
          const errMessage: string | undefined = restoreAuthReq.result.errBody?.error?.message;
          const errCode: string | undefined = restoreAuthReq.result.errBody?.error?.code;

          if (errCode === "WORKSPACE_NOT_INITIALIZED") {
            // db is migrated, but workspace does not exist, probably because it has been reset
            const _reconnectReq = await this.AUTH_RECONNECT();
            // WHAT HAD HAPPENED WAS...
            // in a local scenario where prd auth portal has a workspace, and local does not, the first attempt this will hit an infinite loop trying to find the workspace, totally stuck
            // on a second attempt, it appears, the workspace gets created and user exits the loop
            // TODO: react to failure here?
          } else if (
            // db is totally empty and needs migrations to run
            // TODO: can we catch this more broadly in the backend and return a specific error code?
            errMessage &&
            errMessage.includes("relation") &&
            errMessage.includes("does not exist")
          ) {
            /* eslint-disable no-console, no-alert */
            console.log("db needs migrations");
            // TODO: probably show a better error than an alert
            alert("Looks like your database needs migrations - please restart SDF");
          } else {
            this.localLogout();
          }
        }
      },
      async logout() {
        // Call backend to revoke token
        // Use fetch directly with the workspace token from localStorage
        try {
          const workspaceToken = this.selectedWorkspaceToken;
          if (workspaceToken) {
            const AUTH_API_URL = import.meta.env.VITE_AUTH_API_URL;
            await fetch(`${AUTH_API_URL}/session/logout`, {
              method: "POST",
              headers: {
                Authorization: `Bearer ${workspaceToken}`,
                "Content-Type": "application/json",
              },
            });
          }
        } catch (error) {
          // Silently fail - logout will still clear local state
        }

        // Clear local state and redirect
        this.localLogout(true);
      },

      localLogout(redirectToAuthPortal = true) {
        storage.removeItem(AUTH_LOCAL_STORAGE_KEYS.USER_TOKENS);
        this.$patch({
          tokens: {},
          userPk: null,
        });
        posthog.reset();

        if (window && redirectToAuthPortal) {
          window.location.href = `${AUTH_PORTAL_URL}/dashboard`;
        }
      },

      // split out so we can reuse for different login methods (password, oauth, magic link, signup, etc)
      finishUserLogin(loginResponse: LoginResponse) {
        const decodedJwt = jwtDecode<TokenData>(loginResponse.token);
        const { userPk, workspacePk } = normalizeTokenData(decodedJwt);

        this.$patch({
          userPk,
          tokens: {
            ...this.tokens,
            [workspacePk]: loginResponse.token,
          },
          user: loginResponse.user,
          userWorkspaceFlags: loginResponse.userWorkspaceFlags,
        });
        // store the tokens in localstorage
        storage.setItem(AUTH_LOCAL_STORAGE_KEYS.USER_TOKENS, JSON.stringify(this.tokens));
        // track this workspace access in shared cookie for logout
        addAccessedWorkspace(workspacePk);
        // identify the user in posthog
        posthog.identify(loginResponse.user.pk, {
          email: loginResponse.user.email,
        });
      },

      updateFlags(flags: Record<string, boolean>) {
        this.userWorkspaceFlags = flags;
      },

      async FORCE_REFRESH_MEMBERS(workspaceId: string) {
        return new ApiRequest({
          method: "post",
          url: "/session/refresh_workspace_members",
          params: {
            workspaceId,
          },
        });
      },

      registerRequestsBegin(requestUlid: string, actionName: string) {
        realtimeStore.inflightRequests.set(requestUlid, actionName);
      },
      registerRequestsEnd(requestUlid: string) {
        realtimeStore.inflightRequests.delete(requestUlid);
      },
    },
    // THIS STORE ISN'T WRAPPED IN addStoreHooks SO THIS DOES NOT RUN
    // websocket event listener for authStore events is in App.vue
    onActivated() {},
  })();
};
