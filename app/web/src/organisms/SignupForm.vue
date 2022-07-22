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
              required
              @error="setFieldInError('billingAccountName', $event)"
            />
          </div>

          <div class="sm:col-span-6">
            <SiTextBox2
              id="userName"
              v-model="form.userName"
              title="Full Name"
              description="Your full name."
              required
              @error="setFieldInError('userName', $event)"
            />
          </div>

          <div class="sm:col-span-6">
            <SiTextBox2
              id="userEmail"
              v-model="form.userEmail"
              title="Email"
              description="Your email address."
              required
              :validations="[
                {
                  id: 'email',
                  message: 'Must be a valid email address.',
                  check: validator.isEmail,
                },
              ]"
              @error="setFieldInError('userEmail', $event)"
            />
          </div>

          <div class="sm:col-span-6">
            <SiTextBox2
              id="userPassword"
              v-model="form.userPassword"
              title="Password"
              password
              description="Your password."
              required
              :validations="[
                {
                  id: 'strongPassword',
                  message:
                    'Must be > 8 characters and must have a mix of lowercase, uppercase, number and a symbol.',
                  check: validator.isStrongPassword,
                },
              ]"
              @error="setFieldInError('userPassword', $event)"
            />
          </div>

          <div class="sm:col-span-6">
            <SiTextBox2
              id="signupSecret"
              v-model="form.signupSecret"
              title="Agent Passphrase"
              password
              description="The secret agent passphrase provided to you by the Initiative."
              required
              @error="setFieldInError('signupSecret', $event)"
            />
          </div>

          <div class="sm:col-span-6">
            <button
              type="submit"
              data-test="signUp"
              aria-label="Sign Up"
              class="w-full flex justify-center py-2 px-4 border border-transparent rounded-sm shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 disabled:opacity-50"
              :disabled="formInError"
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
import validator from "validator";
import _ from "lodash";
import { useFieldErrors } from "@/composables/useFieldErrors";

const emit = defineEmits(["success", "back-to-login"]);

const form = ref<CreateAccountRequest>({
  billingAccountName: "",
  userName: "",
  userEmail: "",
  userPassword: "",
  signupSecret: "",
});

const errorMessage = ref<undefined | string>(undefined);

const { formInError, setFieldInError } = useFieldErrors();

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
