import { defineStore } from "pinia";
// import storage from "local-storage-fallback"; // drop-in storage polyfill which falls back to cookies/memory
import { useRouter } from "vue-router";
import { ApiRequest } from "@si/vue-lib";

// keys we use to store auth tokens in local storage
const AUTH_LOCAL_STORAGE_KEYS = {
  USER: "si-auth",
};

export type UserId = string;

// TODO: figure out good way to share this type with backend...
export type User = {
  id: UserId;
  externalId: string; // auth0 id - based on 3rd party
  externalDetails?: any; // json blob, just store auth0 details for now
  nickname: string;
  firstName?: string;
  lastName?: string;
  email?: string;
  emailVerified: boolean;
  pictureUrl?: string;
  needsTosUpdate?: boolean;
};

export type TosDetails = {
  id: string;
  pdfUrl: string;
  html: string;
};

export const useAuthStore = defineStore("auth", {
  state: () => ({
    user: null as User | null,
    tosDetails: null as TosDetails | null,
  }),
  getters: {
    // userIsLoggedIn: (state) => !!state.token,
    userIsLoggedIn: (state) => !!state.user,
    bestUserLabel: (state) => {
      if (!state.user) return "user";
      return (
        state.user.firstName ||
        state.user.nickname ||
        state.user.email ||
        "user"
      );
    },
  },
  actions: {
    // fetches user + billing account info - called on page refresh
    async CHECK_AUTH() {
      return new ApiRequest<{ user: User }>({
        url: "/whoami",
        onSuccess: (response) => {
          this.user = response.user;
        },
        onFail(e) {
          /* eslint-disable-next-line no-console */
          console.log("RESTORE AUTH FAILED!", e);
          // trigger logout?
        },
      });
    },

    async LOAD_TOS_DETAILS() {
      return new ApiRequest<TosDetails>({
        url: "/tos-details",
        onSuccess: (response) => {
          this.tosDetails = response;
        },
      });
    },
    async AGREE_TOS() {
      if (!this.tosDetails) throw new Error("TOS details not loaded");
      return new ApiRequest({
        method: "post",
        url: "/tos-agreement",
        params: {
          tosVersionId: this.tosDetails.id,
        },
        onSuccess: (response) => {
          if (!this.user) throw new Error("user not set");
          this.user.needsTosUpdate = false;
        },
      });
    },

    // SIGNUP
  },
});
