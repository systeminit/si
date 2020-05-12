<template>
  <div>
    <v-card :loading="$apollo.loading">
      <v-card-title>Sign In to IRA</v-card-title>
      <v-alert type="warning" v-if="error">Sign in failed; try again? ({{ error }})</v-alert>
      <v-form>
        <div v-for="field of objMethod.properties.attrs" v-bind:key="field.name">
          <div v-if="field.kind() == 'text'">
            <v-text-field
              v-model="objVariables[field.name]"
              :label="field.label"
              :name="field.name"
            ></v-text-field>
          </div>
          <div v-else-if="field.kind() == 'password'">
            <v-text-field
              v-model="objVariables[field.name]"
              :label="field.label"
              :name="field.name"
              type="password"
            ></v-text-field>
          </div>
        </div>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="checkLogin()">Submit</v-btn>
        </v-card-actions>
      </v-form>
    </v-card>
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
