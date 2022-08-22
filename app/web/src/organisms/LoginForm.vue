<template>
  <div class="min-h-full flex flex-col justify-center py-12 sm:px-6 lg:px-8">
    <div class="sm:mx-auto sm:w-full sm:max-w-md text-neutral-200">
      <img
        class="mx-auto h-12 w-auto"
        :src="siLogoWts"
        alt="System Initiative"
      />
      <h2 class="mt-6 text-center text-3xl font-extrabold">
        Sign in to an existing account
      </h2>
      <p class="mt-2 text-center text-md">
        Or
        {{ " " }}
        <router-link
          :to="{ name: 'signup' }"
          class="font-medium text-action-300 hover:text-action-400"
        >
          create a new account to start your free trial
        </router-link>
      </p>
    </div>

    <div class="mt-8 sm:mx-auto sm:w-full sm:max-w-md">
      <div class="bg-neutral-900 py-8 px-4 shadow sm:rounded-sm sm:px-10">
        <div
          v-if="errorMessage"
          data-testid="error-message"
          class="text-white bg-destructive-500"
        >
          Error: {{ errorMessage }}
        </div>

        <form class="space-y-6">
          <div>
            <SiTextBox
              id="billingAccountName"
              v-model="form.billingAccountName"
              title="Billing Account Name"
              login-mode
              required
              @error="setFieldInError('billingAccountName', $event)"
            />
          </div>

          <div>
            <SiTextBox
              id="userEmail"
              v-model="form.userEmail"
              title="Email Address"
              login-mode
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
            <SiTextBox
              id="userPassword"
              v-model="form.userPassword"
              title="Password"
              password
              login-mode
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
              class="w-full flex justify-center py-2 px-4 border border-transparent rounded-sm shadow-sm text-sm font-medium text-white bg-blue-500 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 disabled:opacity-50"
              :disabled="formInError"
              @click.prevent="login"
            >
              Sign in
            </button>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { SessionService } from "@/service/session";
import siLogoWts from "@/assets/images/si-logo-wts.svg";
import SiTextBox from "@/atoms/SiTextBox.vue";
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
