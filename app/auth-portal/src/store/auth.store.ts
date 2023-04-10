import { defineStore } from "pinia";
// import storage from "local-storage-fallback"; // drop-in storage polyfill which falls back to cookies/memory
import { ApiRequest } from "@si/vue-lib/pinia";
import { posthog } from "posthog-js";

export type UserId = string;

// TODO: figure out good way to share this type with backend...
export type User = {
  id: UserId;
  externalId: string; // auth0 id - based on 3rd party
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  externalDetails?: any; // json blob, just store auth0 details for now
  nickname: string;
  firstName: string | null;
  lastName: string | null;
  email?: string;
  emailVerified: boolean;
  pictureUrl: string | null;
  needsTosUpdate?: boolean;
  githubUsername?: string;
  discordUsername?: string;
};

export const useAuthStore = defineStore("auth", {
  state: () => ({
    user: null as User | null,
    waitingForAccess: false,
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
    // useful to keep this logic in one place
    needsProfileUpdate: (state) =>
      !state.user?.githubUsername || !state.user?.discordUsername,
  },
  actions: {
    // fetches user + billing account info - called on page refresh
    // split from LOAD_USER since it will likely change
    // and because this request loading blocks the whole page/app
    async CHECK_AUTH() {
      return new ApiRequest<{ user: User }>({
        url: "/whoami",
        onSuccess: (response) => {
          this.user = response.user;
          posthog.identify(this.user.id);
          if (this.user.email) {
            posthog.alias(this.user.id, this.user.email);
          }
        },
        onFail(e) {
          /* eslint-disable-next-line no-console */
          console.log("RESTORE AUTH FAILED!", e);
          // trigger logout?
        },
      });
    },

    async LOAD_USER() {
      return new ApiRequest<{ user: User }>({
        url: "/whoami",
        onSuccess: (response) => {
          this.user = response.user;
        },
      });
    },
    async UPDATE_USER(user: Partial<User>) {
      if (!this.user) throw new Error("User not loaded");
      return new ApiRequest<{ user: User }>({
        method: "patch",
        url: `/users/${this.user.id}`,
        params: user,
        onSuccess: (response) => {
          this.user = response.user;
        },
      });
    },

    async AGREE_TOS(tosVersionId: string) {
      return new ApiRequest({
        method: "post",
        url: "/tos-agreement",
        params: {
          tosVersionId,
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
