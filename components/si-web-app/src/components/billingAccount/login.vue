<template>
  <div :loading="$apollo.loading">
  
      <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded relative" role="alert" v-if="error">
        <strong class="font-bold">Sign in failed; try again?</strong>
        <span class="block sm:inline">({{ error }})</span>
      </div>

      <form class="bg-primary shadow-md rounded px-12 pt-6 pb-8 mb-4">

        <p class="text-center text-xl justify-center text-white mb-4">Sign In to The System Initiative</p>
        
        <div class="mb-4">

          <div v-for="field of objMethod.properties.attrs" v-bind:key="field.name">
            <div v-if="field.kind() == 'text'">
              
              <label class="block text-gray-200 text-sm font-bold mb-2" for="password">
                {{ field.label }}
              </label>

              <input class="block login-input-bg-color border border-gray-700 text-red-100 text-sm font-bold mb-2"
                v-model="objVariables[field.name]"
                :label="field.label"
                :name="field.name"
              >
            </div>

            <div v-else-if="field.kind() == 'password'">
              <label class="block border-gray-300 text-gray-200 text-sm font-bold mb-2" for="password">
                {{ field.label }}
              </label>

              <input class="block border login-input-bg-color border-gray-700 text-gray-700 text-sm font-bold mb-2"
                v-model="objVariables[field.name]"
                :label="field.label"
                :name="field.name"
                type="password"
              >
            </div>
          </div>

          <button class="logint-button-bg-color-dark hover:login-button-bg-color-light text-gray-200 font-bold py-1 px-4 focus:outline-none focus:shadow-outline" @click="checkLogin()" type="button">
            SignIn
          </button>

        </div>

      </form>
  </div>
</template>

<script lang="ts">
import Vue from "vue";

import { billingAccountList, auth } from "@/auth";

import { registry, PropMethod } from "si-registry";
const user = registry.get("user");

export default Vue.extend({
  name: "billingAccountLogin",
  data(): Record<string, any> {
    if (user == undefined) {
      throw "Registry object is undefined! Bug!";
    }
    let me = user.methods.getEntry("loginInternal") as PropMethod;
    return {
      obj: user,
      objVariables: user.graphql.variablesObject({
        methodName: "loginInternal",
      }),
      objMethod: me.request,
      error: undefined,
    };
  },
  methods: {
    async checkLogin() {
      try {
        const loginResult = await this.$apollo.query({
          query: user.graphql.query({
            methodName: "loginInternal",
            overrideName: "userLogin",
            overrideFields: "jwt, userId, billingAccountId",
          }),
          variables: this.objVariables,
        });
        let data = loginResult.data.userLogin;

        billingAccountList.addAccount({
          id: data.billingAccountId,
          shortName: this.billingAccountSelect,
        });
        this.billingAccounts = billingAccountList.getAccounts();

        await auth.login(data.jwt, data.userId);
      } catch (err) {
        this.error = err;
        console.log(err);
        return;
      }
      await this.$router.push("/");
    },
  },
});
</script>

<style scoped>

.login-input-bg-color {
  background-color: #2B2E2F;
}

.logint-button-bg-color-dark {
  background-color: #25788A;
}

.login-button-bg-color-light {
  background-color: #3FCBEA;
}

</style>
