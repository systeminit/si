import { defineStore } from "pinia";
import storage from "local-storage-fallback"; // drop-in storage polyfill which falls back to cookies/memory
import jwtDecode from "jwt-decode";
import { useRouter } from "vue-router";
import { ApiRequest } from "@/store/lib/pinia_api_tools";

import { User } from "@/api/sdf/dal/user";
import { BillingAccount } from "@/api/sdf/dal/billing_account";

// keys we use to store auth tokens in local storage
const AUTH_LOCAL_STORAGE_KEYS = {
  USER: "si-auth",
};

type TokenData = {
  user_id: string;
  billing_account_id: string;
  // isImpersonating?: boolean;
};

interface LoginResponse {
  user: User;
  billingAccount: BillingAccount;
  jwt: string;
}

export const useAuthStore = defineStore("auth", {
  state: () => ({
    token: null as string | null,
    billingAccountId: null as string | null,
    userId: null as string | null,
    adminIsImpersonatingUser: false,

    // TODO: these maybe should live in another module related to the user/org/groups/etc
    user: null as User | null,
    billingAccount: null as BillingAccount | null,
  }),
  getters: {
    userIsLoggedIn: (state) => !!state.token,
  },
  actions: {
    async LOGIN(payload: {
      billingAccountName: string;
      userEmail: string;
      userPassword: string;
    }) {
      return new ApiRequest<LoginResponse>({
        method: "post",
        url: "/session/login",
        params: payload,
        onSuccess: (response) => {
          // finish login is split out because we'll likely add other login methods or trigger login after signup
          // (ex: oauth, magic link)
          this.finishUserLogin(response);
        },
      });
    },
    // fetches user + billing account info - called on page refresh
    async RESTORE_AUTH() {
      return new ApiRequest<Omit<LoginResponse, "jwt">>({
        url: "/session/restore_authentication",
        onSuccess: (response) => {
          this.user = response.user;
          this.billingAccount = response.billingAccount;
        },
        onFail(e) {
          /* eslint-disable-next-line no-console */
          console.log("RESTORE AUTH FAILED!", e);
          // trigger logout?
        },
      });
    },

    // TODO: set up these actions / routes
    /*
    // usually not used, this causes api to log out *all* sessions for the user
    LOGOUT_ALL_SESSIONS() {
      return new ApiRequest({
        method: "post",
        url: "/auth/logout",
        onSuccess: (response: any) => {
          this.localLogout();
        },
        onFail: () => {
          // still want to log out (clear local storage) even if api request fails
          // but may want to alert the user
          this.localLogout();
        },
      });
    },

    REQUEST_PASSWORD_RESET(email: string) {
      return new ApiRequest({
        method: "post",
        url: "/auth/request-password-reset",
        params: { email },
      });
    },
    COMPLETE_PASSWORD_RESET(payload: {
      billingAccountName: string;
      email: string;
      resetToken: string;
      newPassword: string;
    }) {
      return new ApiRequest({
        method: "post",
        url: "/auth/password-reset",
        params: payload,
      });
    },

    REQUEST_MAGIC_LINK(payload: { email: string; redirectUrl?: string }) {
      return new ApiRequest({
        method: "post",
        url: "/auth/request-magic-link",
        params: payload,
      });
    },
    USE_MAGIC_LINK(token: string) {
      return new ApiRequest({
        method: "post",
        url: "/auth/use-magic-link",
        params: { token },
        onSuccess: (response: any) => {
          this.finishUserLogin(response);
        },
      });
    },
    */

    // SIGNUP
    async SIGNUP(payload: {
      billingAccountName: string;
      userEmail: string;
      userPassword: string;
      userName: string;
      signupSecret: string;
    }) {
      return new ApiRequest({
        method: "post",
        url: "/signup/create_account",
        params: payload,
        // TODO: we could return an auth token and log the user in?
        // onSuccess: (response) => {},
      });
    },

    // OTHER ACTIONS ///////////////////////////////////////////////////////////////////
    async initFromStorage() {
      // check regular user token (we will likely have a different token for admin auth later)
      const token = storage.getItem(AUTH_LOCAL_STORAGE_KEYS.USER);
      if (!token) return;

      // token contains user id and biling account id
      const { user_id: userId, billing_account_id: billingAccountId } =
        jwtDecode<TokenData>(token);
      this.$patch({
        token,
        userId,
        billingAccountId,
        // adminIsImpersonatingUser: isImpersonating,
      });

      // this endpoint re-fetches the user and billing account
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
        userId: null,
        billingAccountId: null,
        adminIsImpersonatingUser: false,
      });
    },

    // split out so we can reuse for different login methods (password, oauth, magic link, signup, etc)
    finishUserLogin(loginResponse: LoginResponse) {
      const decodedJwt = jwtDecode<TokenData>(loginResponse.jwt);
      this.$patch({
        userId: decodedJwt.user_id,
        billingAccountId: decodedJwt.billing_account_id,
        token: loginResponse.jwt,
        user: loginResponse.user,
        billingAccount: loginResponse.billingAccount,
      });

      // store the token in localstorage
      storage.setItem(AUTH_LOCAL_STORAGE_KEYS.USER, loginResponse.jwt);

      // pass along user/company data to those stores so we dont have to load again
      // const usersStore = useUsersStore();
      // const companyStore = useCompanyStore();
      // usersStore.setCurrentUser(loginResponse.user);
      // companyStore.setCurrentCompany(loginResponse.company);
    },
  },
});
