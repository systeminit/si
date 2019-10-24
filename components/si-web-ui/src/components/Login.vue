<template>
  <v-card class="elevation-12" :loading="loading">
    <v-toolbar color="primary" dark flat>
      <v-toolbar-title>Sign In to IRA</v-toolbar-title>
      <v-spacer />
      <v-btn to="signup" color="accent">Sign Up</v-btn>
    </v-toolbar>
    <v-card-text>
      <v-alert type="warning" v-if="error">
        Sign in failed; try again?
      </v-alert>
      <v-form>
        <v-combobox
          v-model="billingAccountSelect"
          name="billingAccount"
          :items="billingAccounts"
          item-value="shortName"
          item-text="shortName"
          :return-object="false"
          prepend-icon="mdi-account-group"
          label="Billing Account Name"
        />

        <v-text-field
          label="Email Address"
          name="email"
          prepend-icon="mdi-account-circle"
          type="text"
          v-model="email"
        />

        <v-text-field
          id="password"
          label="Password"
          name="password"
          prepend-icon="mdi-lock"
          type="password"
          v-model="password"
        />
      </v-form>
    </v-card-text>
    <v-card-actions>
      <v-spacer />
      <v-btn class="info" @click="checkLogin()">Login</v-btn>
    </v-card-actions>
  </v-card>
</template>

<script lang="ts">
import Vue from "vue";

import {
  BillingAccount,
  LoginQueryVariables,
  LoginReply,
  LoginQuery,
} from "@/graphql-types";

import { billingAccountList, auth } from "@/auth";
import { onLogin, ExtendedApolloClient } from "@/vue-apollo";

import login from "@/graphql/queries/login.gql";

interface Data {
  error: boolean;
  billingAccountSelect: string;
  billingAccounts: BillingAccount[];
  email: string;
  password: string;
  loading: false | "secondary";
}

export default Vue.extend({
  name: "login",
  data(): Data {
    let billingAccounts = billingAccountList.getAccounts();
    let billingAccountSelect = billingAccountList.getFirstAccountShortName();
    return {
      billingAccountSelect,
      billingAccounts,
      email: "",
      password: "",
      loading: false,
      error: false,
    };
  },
  methods: {
    async checkLogin() {
      this.loading = "secondary";
      try {
        const loginResult = await this.$apollo.query({
          query: login,
          variables: {
            email: this.email,
            password: this.password,
            billingAccountShortName: this.billingAccountSelect,
          },
        });
        let data: LoginQuery = loginResult.data;
        if (!data.login) {
          this.error = true;
          throw "response incomplete";
        } else {
          if (
            !data.login.billingAccountId ||
            !data.login.userId ||
            !data.login.jwt
          ) {
            this.error = true;
            throw "response incomplete; missing a field";
          }

          // Add and set the billing account lists
          billingAccountList.addAccount({
            id: data.login.billingAccountId,
            shortName: this.billingAccountSelect,
          });
          this.billingAccounts = billingAccountList.getAccounts();

          await auth.login(data.login.jwt, data.login.userId);
        }
      } catch (err) {
        this.error = true;
        this.loading = false;
        return;
      }
      this.error = false;
      this.loading = false;
      await this.$router.push("/");
    },
  },
});
</script>
