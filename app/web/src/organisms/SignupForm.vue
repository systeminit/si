<template>
  <div class="min-h-full flex flex-col justify-center py-12 sm:px-6 lg:px-8">
    <div class="sm:mx-auto sm:w-full sm:max-w-md text-neutral-200">
      <img
        class="mx-auto h-12 w-auto"
        :src="siLogoWts"
        alt="System Initiative"
      />
      <h2 class="mt-6 text-center text-3xl font-extrabold">
        Sign up for your free trial
      </h2>
      <p class="mt-2 text-center text-md">
        Or
        {{ " " }}
        <router-link
          :to="{ name: 'login' }"
          class="font-medium text-action-300 hover:text-action-400"
        >
          sign in to an existing account
        </router-link>
      </p>
    </div>

    <div class="mt-8 sm:mx-auto sm:w-full sm:max-w-md">
      <div class="bg-neutral-800 py-8 px-4 shadow sm:rounded-sm sm:px-10">
        <div
          v-if="errorMessage"
          class="bg-destructive-500 text-white p-1 mb-6 text-center text-sm font-medium"
        >
          Error: {{ errorMessage }}
        </div>

        <form
          class="grid grid-cols-1 gap-y-6 gap-x-4 sm:grid-cols-6"
          @submit.prevent="createAccount"
        >
          <div class="sm:col-span-6">
            <SiTextBox
              id="billingAccountName"
              v-model="form.billingAccountName"
              title="Billing Account Name"
              description="A name for your account. A company name is a good idea. You can change it later. (You'll need this to sign in!)"
              required
              login-mode
              @error="setFieldInError('billingAccountName', $event)"
            />
          </div>

          <div class="sm:col-span-6">
            <SiTextBox
              id="userName"
              v-model="form.userName"
              title="Full Name"
              description="Your full name."
              required
              login-mode
              @error="setFieldInError('userName', $event)"
            />
          </div>

          <div class="sm:col-span-6">
            <SiTextBox
              id="userEmail"
              v-model="form.userEmail"
              title="Email"
              description="Your email address."
              required
              login-mode
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
            <SiTextBox
              id="userPassword"
              v-model="form.userPassword"
              title="Password"
              password
              description="Your password."
              required
              login-mode
              :min-password-length="8"
              :max-password-length="64"
              :validations="[
                {
                  id: 'strongPassword',
                  message:
                    'Password must be between 8 and 64 characters and include lowercase, uppercase, number, and symbol.',
                  check: validator.isStrongPassword,
                },
              ]"
              @error="setFieldInError('userPassword', $event)"
            />
          </div>

          <div class="sm:col-span-6">
            <SiTextBox
              id="signupSecret"
              v-model="form.signupSecret"
              login-mode
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
            >
              Sign up
            </button>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { CreateAccountRequest, SignupService } from "@/service/signup";
import siLogoWts from "@/assets/images/si-logo-wts.svg";
import SiTextBox from "@/atoms/SiTextBox.vue";
import validator from "validator";
import _ from "lodash";
import { useFieldErrors } from "@/composables/useFieldErrors";
import { setFormSettings } from "@/composables/formSettings";

const emit = defineEmits(["success", "back-to-login"]);

const form = ref<CreateAccountRequest>({
  billingAccountName: "",
  userName: "",
  userEmail: "",
  userPassword: "",
  signupSecret: "",
});

setFormSettings({ hideRequiredLabel: true });

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
