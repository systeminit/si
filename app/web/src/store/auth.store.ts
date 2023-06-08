import { defineStore } from "pinia";
import storage from "local-storage-fallback"; // drop-in storage polyfill which falls back to cookies/memory
import jwtDecode from "jwt-decode";
import { useRouter } from "vue-router";
import { ApiRequest } from "@si/vue-lib/pinia";
import { posthog } from "@/utils/posthog";

import { User } from "@/api/sdf/dal/user";
import { Workspace } from "@/api/sdf/dal/workspace";

// keys we use to store auth tokens in local storage
const AUTH_LOCAL_STORAGE_KEYS = {
  USER: "si-auth",
};

type TokenData = {
  user_pk: string;
  workspace_pk: string;
  // isImpersonating?: boolean;
};

interface LoginResponse {
  user: User;
  workspace: Workspace;
  token: string;
}

export const useAuthStore = defineStore("auth", {
  state: () => ({
    token: null as string | null,
    workspacePk: null as string | null,
    userPk: null as string | null,
    adminIsImpersonatingUser: false,

    // TODO: these maybe should live in another module related to the user/org/groups/etc
    user: null as User | null,
    workspace: null as Workspace | null,
  }),
  getters: {
    userIsLoggedIn: (state) => !!state.token,
  },
  actions: {
    // fetches user + workspace info - called on page refresh
    async RESTORE_AUTH() {
      return new ApiRequest<Omit<LoginResponse, "jwt">>({
        url: "/session/restore_authentication",
        onSuccess: (response) => {
          this.user = response.user;
          this.workspace = response.workspace;
        },
        onFail(e) {
          /* eslint-disable-next-line no-console */
          console.log("RESTORE AUTH FAILED!", e);
          // trigger logout?
        },
      });
    },

    async AUTH_CONNECT(payload: { code: string }) {
      return new ApiRequest<LoginResponse>({
        method: "post",
        url: "/session/connect",
        params: payload,
        onSuccess: (response) => {
          this.finishUserLogin(response);
        },
      });
    },

    // OTHER ACTIONS ///////////////////////////////////////////////////////////////////
    async initFromStorage() {
      // check regular user token (we will likely have a different token for admin auth later)
      const token = storage.getItem(AUTH_LOCAL_STORAGE_KEYS.USER);
      if (!token) return;

      // token contains user pk and biling account pk
      const { user_pk: userPk, workspace_pk: workspacePk } =
        jwtDecode<TokenData>(token);
      this.$patch({
        token,
        userPk,
        workspacePk,
        // adminIsImpersonatingUser: isImpersonating,
      });

      // this endpoint re-fetches the user and workspace
      // dont think it's 100% necessary at the moment and not quite the right shape, but can fix later
      const restoreAuthReq = await this.RESTORE_AUTH();
      if (!restoreAuthReq.result.success) {
        this.localLogout();

        // not sure this is where we want to do this, but it's fine for now
        const router = useRouter();
        router.push({ name: "login" });
      }
    },
    localLogout() {
      storage.removeItem(AUTH_LOCAL_STORAGE_KEYS.USER);
      this.$patch({
        token: null,
        userPk: null,
        workspacePk: null,
        adminIsImpersonatingUser: false,
      });
      posthog.reset();
    },

    // split out so we can reuse for different login methods (password, oauth, magic link, signup, etc)
    finishUserLogin(loginResponse: LoginResponse) {
      const decodedJwt = jwtDecode<TokenData>(loginResponse.token);
      this.$patch({
        userPk: decodedJwt.user_pk,
        workspacePk: decodedJwt.workspace_pk,
        token: loginResponse.token,
        user: loginResponse.user,
        workspace: loginResponse.workspace,
      });
      // store the token in localstorage
      storage.setItem(AUTH_LOCAL_STORAGE_KEYS.USER, loginResponse.token);
      // identify the user in posthog
      posthog.identify(loginResponse.user.pk, {
        email: loginResponse.user.email,
      });
    },
  },
});
