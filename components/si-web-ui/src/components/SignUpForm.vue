<template>
  <v-card class="elevation-12" :loading="loading">
    <v-toolbar color="primary" dark flat>
      <v-toolbar-title>Sign Up for IRA OS!</v-toolbar-title>
      <v-spacer />
      <v-btn to="sigin" color="accent">Sign In</v-btn>
    </v-toolbar>
    <v-card-text>
      <v-alert type="warning" v-if="error">
        Sign up failed; try again?
      </v-alert>
      <v-form>
        <v-text-field
          label="Account Name"
          name="billingAccountShortName"
          v-model="billingAccountShortName"
          :rules="billingAccountShortNameRules"
          type="text"
          prepend-icon="mdi-account-box-multiple"
        />
        <v-text-field
          label="Account Display Name"
          name="billingAccountDisplayName"
          v-model="billingAccountDisplayName"
          :rules="billingAccountDisplayNameRules"
          type="text"
          prepend-icon="mdi-account-details"
        />
        <v-text-field
          label="Given Name"
          name="userGivenName"
          v-model="userGivenName"
          :rules="userGivenNameRules"
          type="text"
          prepend-icon="mdi-alpha-g"
        />
        <v-text-field
          label="Family Name"
          name="userFamilyName"
          v-model="userFamilyName"
          :rules="userFamilyNameRules"
          type="text"
          prepend-icon="mdi-alpha-f"
        />
        <v-text-field
          label="Display Name"
          name="userDisplayName"
          v-model="userDisplayName"
          :rules="userDisplayNameRules"
          type="text"
          prepend-icon="mdi-alpha-d"
        />
        <v-text-field
          label="Email Address"
          name="email"
          prepend-icon="mdi-at"
          type="text"
          v-model="userEmail"
          :rules="userEmailRules"
        />
        <v-text-field
          id="password"
          label="Password"
          name="password"
          prepend-icon="mdi-lock"
          type="password"
          v-model="userPassword"
          :counter="150"
          :rules="userPasswordRules"
        />
      </v-form>
    </v-card-text>
    <v-card-actions>
      <v-spacer />
      <v-btn class="info" @click="createAccount()">Create Account</v-btn>
    </v-card-actions>
  </v-card>
</template>

<script lang="ts">
import Vue from "vue";

import createAccount from "@/graphql/mutations/createAccount.gql";

type VueValidationArray = Array<(v: string) => true | string>;

interface Data {
  billingAccountShortName: string;
  billingAccountShortNameRules: VueValidationArray;
  billingAccountDisplayName: string;
  billingAccountDisplayNameRules: VueValidationArray;
  userDisplayName: string;
  userDisplayNameRules: VueValidationArray;
  userGivenName: string;
  userGivenNameRules: VueValidationArray;
  userFamilyName: string;
  userFamilyNameRules: VueValidationArray;
  userEmail: string;
  userEmailRules: VueValidationArray;
  userPassword: string;
  userPasswordRules: VueValidationArray;
  error: boolean;
  loading: false | "secondary";
}

export default Vue.extend({
  name: "SignUpForm",
  data(): Data {
    return {
      billingAccountShortName: "",
      billingAccountShortNameRules: [
        (v: string) => !!v || "Account Name is required",
        (v: string) =>
          /^[\w-]+$/.test(v) ||
          "Account Name may only contain A-Z, a-z, 0-9, _, or -",
      ],
      billingAccountDisplayName: "",
      billingAccountDisplayNameRules: [
        (v: string) => !!v || "Account Display Name is required",
      ],
      userDisplayName: "",
      userDisplayNameRules: [(v: string) => !!v || "Display Name is required"],
      userGivenName: "",
      userGivenNameRules: [(v: string) => !!v || "Given Name is required"],
      userFamilyName: "",
      userFamilyNameRules: [(v: string) => !!v || "Family Name is required"],
      userEmail: "",
      userEmailRules: [
        (v: string) => !!v || "Email is required",
        (v: string) => /.+@.+\..+/.test(v) || "E-mail must be valid",
      ],
      userPassword: "",
      userPasswordRules: [
        (v: string) => !!v || "Password is required",
        (v: string) =>
          v.length >= 10 || "Password must be 10 characters or longer",
      ],
      error: false,
      loading: false,
    };
  },
  methods: {
    async createAccount() {
      this.loading = "secondary";
      try {
        await this.$apollo.mutate({
          mutation: createAccount,
          variables: {
            billingAccountShortName: this.billingAccountShortName,
            billingAccountDisplayName: this.billingAccountDisplayName,
            userDisplayName: this.userDisplayName,
            userGivenName: this.userGivenName,
            userFamilyName: this.userFamilyName,
            userEmail: this.userEmail,
            userPassword: this.userPassword,
          },
          update: (_store, _createData) => {
            this.$router.push({ name: "signin" });
          },
        });
      } catch (err) {
        this.error = true;
        this.loading = false;
      }
    },
  },
});
</script>
