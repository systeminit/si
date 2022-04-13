import { defineStore } from "pinia";

export const useSecretStore = defineStore("secret", {
  state: () => {
    return {
      secretAgent: false,
    };
  },
  actions: {
    authenticateSecretAgent(passphrase: String) {
      if (passphrase === "seethefnord") {
        this.secretAgent = true;
      } else {
        this.secretAgent = false;
      }
    },
    leaveInitiative() {
      this.secretAgent = false;
    },
  },
  persist: true,
});
