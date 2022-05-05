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
        <div v-if="errorMessage" class="text-white bg-red-500">
          Error: {{ errorMessage }}
        </div>

        <div class="grid grid-cols-1 gap-y-6 gap-x-4 sm:grid-cols-6">
          <div class="sm:col-span-6">
            <label
              for="billingAccountName"
              class="block text-sm font-medium text-gray-200"
            >
              Billing Account Name
            </label>
            <div class="mt-1 w-full">
              <input
                id="billingAccountName"
                v-model="form.billingAccountName"
                data-test="billingAccountName"
                type="text"
                name="billingAccountName"
                autocomplete="billingAccountName"
                required
                class="appearance-none block bg-gray-900 text-gray-100 w-full px-3 py-2 border border-gray-600 rounded-sm shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-200 focus:border-indigo-200 sm:text-sm"
              />
            </div>
            <p class="mt-2 text-xs text-gray-300">
              A name for your account. A company name is a good idea. You can
              change it later. (You'll need this to sign in!)
            </p>
          </div>

          <div class="sm:col-span-6">
            <label
              for="userName"
              class="block text-sm font-medium text-gray-200"
            >
              Full Name
            </label>
            <div class="mt-1 w-full">
              <input
                id="userName"
                v-model="form.userName"
                data-test="userName"
                type="text"
                name="userName"
                autocomplete="userName"
                required
                class="appearance-none block bg-gray-900 text-gray-100 w-full px-3 py-2 border border-gray-600 rounded-sm shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-200 focus:border-indigo-200 sm:text-sm"
              />
            </div>
            <p class="mt-2 text-xs text-gray-300">Your full name.</p>
          </div>

          <div class="sm:col-span-6">
            <label
              for="userEmail"
              class="block text-sm font-medium text-gray-200"
            >
              Email
            </label>
            <div class="mt-1 w-full">
              <input
                id="userEmail"
                v-model="form.userEmail"
                data-test="userEmail"
                type="email"
                name="email"
                autocomplete="email"
                required
                class="appearance-none block bg-gray-900 text-gray-100 w-full px-3 py-2 border border-gray-600 rounded-sm shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-200 focus:border-indigo-200 sm:text-sm"
              />
            </div>
            <p class="mt-2 text-xs text-gray-300">Your email address.</p>
          </div>

          <div class="sm:col-span-6">
            <label
              for="userPassword"
              class="block text-sm font-medium text-gray-200"
            >
              Password
            </label>
            <div class="mt-1 w-full">
              <input
                id="userPassword"
                v-model="form.userPassword"
                data-test="userPassword"
                type="password"
                name="password"
                required
                class="appearance-none block bg-gray-900 text-gray-100 w-full px-3 py-2 border border-gray-600 rounded-sm shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-200 focus:border-indigo-200 sm:text-sm"
              />
            </div>
            <p class="mt-2 text-xs text-gray-300">Your password.</p>
          </div>

          <div class="sm:col-span-6">
            <label
              for="signupSecret"
              class="block text-sm font-medium text-gray-200"
            >
              Agent Passphrase
            </label>
            <div class="mt-1 w-full">
              <input
                id="signupSecret"
                v-model="form.signupSecret"
                data-test="signupSecret"
                type="password"
                name="password"
                required
                class="appearance-none block bg-gray-900 text-gray-100 w-full px-3 py-2 border border-gray-600 rounded-sm shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-200 focus:border-indigo-200 sm:text-sm"
              />
            </div>
            <p class="mt-2 text-xs text-gray-300">
              The secret agent passphrase provided to you by the Initiative.
            </p>
          </div>
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
