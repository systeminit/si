<template>
  <div class="min-h-full flex flex-col justify-center py-12 sm:px-6 lg:px-8">
    <div class="sm:mx-auto sm:w-full sm:max-w-md">
      <img
        class="mx-auto h-12 w-auto"
        :src="siLogoWts"
        alt="System Initiative"
      />
      <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-200">
        Sign in to your account
      </h2>
      <p class="mt-2 text-center text-sm text-gray-200">
        Or
        {{ " " }}
        <router-link
          :to="{ name: 'signup' }"
          class="font-medium text-indigo-300 hover:text-indigo-400"
        >
          sign up for a free trial
        </router-link>
      </p>
    </div>

    <div class="mt-8 sm:mx-auto sm:w-full sm:max-w-md">
      <div class="bg-gray-800 py-8 px-4 shadow sm:rounded-sm sm:px-10">
        <div
          v-if="errorMessage"
          data-testid="error-message"
          class="text-white bg-red-500"
        >
          Error: {{ errorMessage }}
        </div>

        <div class="space-y-6">
          <div>
            <SiTextBox2
              id="billingAccountName"
              v-model="form.billingAccountName"
              title="Billing Account Name"
              required
              @error="setFieldInError('billingAccountName', $event)"
            />
          </div>

          <div>
            <SiTextBox2
              id="userEmail"
              v-model="form.userEmail"
              title="Email Address"
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

          <div>
            <SiTextBox2
              id="userPassword"
              v-model="form.userPassword"
              title="Password"
              password
              description="Your password."
              required
              autocomplete="current-password"
              @error="setFieldInError('userPassword', $event)"
            />
          </div>

          <div>
            <button
              type="submit"
              data-test="login"
              aria-label="Sign In"
              class="w-full flex justify-center py-2 px-4 border border-transparent rounded-sm shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 disabled:opacity-50"
              :disabled="formInError"
              @click.prevent="login"
            >
              Sign in
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { SessionService } from "@/service/session";
import siLogoWts from "@/assets/images/si-logo-wts.svg";
import SiTextBox2 from "@/atoms/SiTextBox2.vue";
import { useFieldErrors } from "@/composables/useFieldErrors";
import validator from "validator";

const form = ref({
  billingAccountName: "",
  userEmail: "",
  userPassword: "",
});
const errorMessage = ref<string | null>(null);

const { formInError, setFieldInError } = useFieldErrors();

const emit = defineEmits(["signup", "success"]);

const login = () => {
  SessionService.login(form.value).subscribe((response) => {
    if (!response) {
      errorMessage.value = "Login error; please try again!";
    } else if (response.error) {
      errorMessage.value = "Login error; please try again!";
    } else {
      emit("success");
    }
  });
};
</script>
