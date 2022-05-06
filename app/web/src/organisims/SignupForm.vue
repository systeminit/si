<template>
  <div class="min-h-full flex flex-col justify-center py-12 sm:px-6 lg:px-8">
    <div class="sm:mx-auto sm:w-full sm:max-w-md">
      <img
        class="mx-auto h-12 w-auto"
        :src="siLogoWts"
        alt="System Initiative"
      />
      <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-200">
        Sign up for a free trial
      </h2>
      <p class="mt-2 text-center text-sm text-gray-200">
        Or
        {{ " " }}
        <router-link
          :to="{ name: 'login' }"
          class="font-medium text-indigo-300 hover:text-indigo-400"
        >
          sign in to your account
        </router-link>
      </p>
    </div>

    <div class="mt-8 sm:mx-auto sm:w-full sm:max-w-md">
      <div class="bg-gray-800 py-8 px-4 shadow sm:rounded-sm sm:px-10">
        <div
          v-if="errorMessage"
          class="bg-red-600 text-white p-1 mb-6 text-center text-sm font-medium"
        >
          Error: {{ errorMessage }}
        </div>

        <div class="grid grid-cols-1 gap-y-6 gap-x-4 sm:grid-cols-6">
          <div class="sm:col-span-6">
            <SiTextBox2
              id="billingAccountName"
              v-model="form.billingAccountName"
              title="Billing Account Name"
              description="A name for your account. A company name is a good idea. You can change it later. (You'll need this to sign in!)"
            />
            <!--
            :error="form.billingAccountName === ''"
            error-id="billing-account-name-error"
            error-message="Billing account name cannot be an empty string or whitespace."
          --></div>

          <div class="sm:col-span-6">
            <SiTextBox2
              id="userName"
              v-model="form.userName"
              title="Full Name"
              description="Your full name."
            />
            <!--
            :error="false"
            error-id="name-error"
            error-message="Full name cannot be an empty string or whitespace."
          --></div>

          <div class="sm:col-span-6">
            <SiTextBox2
              id="userEmail"
              v-model="form.userEmail"
              title="Email"
              description="Your email address."
            />
            <!--
            :error="false"
            error-id="email-error"
            error-message="Email address must include the '@' character."
          --></div>

          <div class="sm:col-span-6">
            <SiTextBox2
              id="userPassword"
              v-model="form.userPassword"
              title="Password"
              :password="true"
              description="Your password."
            />
            <!--
            :error="false"
            error-id="password-error"
            error-message="Password cannot be an empty string or whitespace."
          --></div>

          <div class="sm:col-span-6">
            <SiTextBox2
              id="signupSecret"
              v-model="form.signupSecret"
              title="Agent Passphrase"
              :password="true"
              description="The secret agent passphrase provided to you by the Initiative."
            />
            <!--
            :error="false"
            error-id="agent-passphrase-error"
            error-message="Agent passphrase cannot be an empty string or whitespace."
          --></div>

          <div class="sm:col-span-6">
            <button
              type="submit"
              data-test="signUp"
              aria-label="Sign Up"
              class="w-full flex justify-center py-2 px-4 border border-transparent rounded-sm shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-400"
              @click="createAccount"
            >
              Sign up
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { CreateAccountRequest, SignupService } from "@/service/signup";
import siLogoWts from "@/assets/images/si-logo-wts.svg";
import SiTextBox2 from "@/atoms/SiTextBox2.vue";

const emit = defineEmits(["success", "back-to-login"]);

const form = ref<CreateAccountRequest>({
  billingAccountName: "",
  userName: "",
  userEmail: "",
  userPassword: "",
  signupSecret: "",
});

const errorMessage = ref<undefined | string>(undefined);

const createAccount = () => {
  SignupService.createAccount(form.value).subscribe((response) => {
    if (response.error) {
      errorMessage.value = response.error.message;
    } else {
      emit("success");
    }
  });
};
</script>
