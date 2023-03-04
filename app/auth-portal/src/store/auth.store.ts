import { defineStore } from "pinia";
// import storage from "local-storage-fallback"; // drop-in storage polyfill which falls back to cookies/memory
import { useRouter } from "vue-router";
import { ApiRequest } from "@si/vue-lib";

// keys we use to store auth tokens in local storage
const AUTH_LOCAL_STORAGE_KEYS = {
  USER: "si-auth",
};

export type UserId = string;

export const useAuthStore = defineStore("auth", {
  state: () => ({
    userId: null as string | null,
    userEmail: null as string | null,
  }),
  getters: {
    // userIsLoggedIn: (state) => !!state.token,
    userIsLoggedIn: (state) => !!state.userId,
  },
  actions: {
    // fetches user + billing account info - called on page refresh
    async CHECK_AUTH() {
      return new ApiRequest<{ user: { id: string; email: string } }>({
        url: "/whoami",
        onSuccess: (response) => {
          this.userId = response.user.id;
          this.userEmail = response.user.email;
        },
        onFail(e) {
          /* eslint-disable-next-line no-console */
          console.log("RESTORE AUTH FAILED!", e);
          // trigger logout?
        },
      });
    },

    // SIGNUP
  },
});
