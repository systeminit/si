<template>
  <div class="min-h-full flex flex-col justify-center py-12 sm:px-6 lg:px-8">
    <div class="sm:mx-auto sm:max-w-md text-neutral-200 flex flex-row">
      <img
        class="mr-6 h-[4.25rem] w-auto"
        :src="siLogoWts"
        alt="System Initiative"
      />
      <div>
        <h2 class="text-left text-3xl font-extrabold">Sign Up</h2>
        <p class="mt-2 text-left text-md">
          Already have an account?
          <router-link
            :to="{ name: 'login' }"
            class="font-medium text-action-300 hover:text-action-400"
          >
            Log in!
          </router-link>
        </p>
      </div>
    </div>

    <div class="mt-8 sm:mx-auto sm:w-full sm:max-w-md">
      <div
        class="bg-neutral-900 pt-2 pb-4 px-4 shadow sm:rounded-t-md sm:px-10"
      >
        <div
          v-if="errorMessage"
          class="bg-destructive-500 text-white p-1 my-2 text-center text-sm font-medium rounded-sm"
        >
          Error: {{ errorMessage }}
        </div>
        <div v-else class="text-neutral-50 text-sm text-right py-0.5">
          Required Field <span class="text-destructive-500">*</span>
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
              placeholder="E.g. company name - you will need this to sign in"
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
              placeholder="First and last name - you can change this later"
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
              placeholder="The email you will use to sign in to your account"
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
              placeholder="A secure password you won't forget, 8 to 64 characters"
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
              placeholder="The secret agent phrase provided to you by the Initiative"
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
              Create Account
            </button>
          </div>
        </form>
      </div>

      <div
        class="border-t-2 border-black text-white text-center font-medium bg-neutral-900 pt-4 pb-8 px-4 shadow sm:rounded-b-md sm:px-10"
      >
        <div class="mb-4">Already have an account?</div>

        <router-link
          :to="{ name: 'login' }"
          class="w-full flex justify-center py-2 px-4 border border-action-500 rounded-sm shadow-sm text-sm font-bold text-action-500 hover:bg-action-50 hover:text-action-600 hover:border-action-600 focus:bg-action-100 focus:border-action-700 focus:text-action-700"
        >
          Log In!
        </router-link>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { CreateAccountRequest, SignupService } from "@/service/signup";
import siLogoWts from "@/assets/images/si-logo-wts.svg?url";
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

setFormSettings({
  hideRequiredLabel: false,
  requiredLabel: "*",
  requiredLabelClasses: "text-destructive-500",
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
