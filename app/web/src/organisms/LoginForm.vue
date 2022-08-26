<template>
  <div class="min-h-full flex flex-col justify-center py-12 sm:px-6 lg:px-8">
    <div class="sm:mx-auto sm:max-w-md text-neutral-200 flex flex-row">
      <img
        class="mr-6 h-[4.25rem] w-auto"
        :src="siLogoWts"
        alt="System Initiative"
      />
      <div>
        <h2 class="text-left text-3xl font-extrabold">Log In</h2>
        <p class="mt-2 text-left text-md">
          Don't have an account?
          <router-link
            :to="{ name: 'signup' }"
            class="font-medium underline text-action-300 hover:text-action-400"
          >
            Create one!
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

        <form class="space-y-6" @submit.prevent="login">
          <SiTextBox
            id="billingAccountName"
            v-model="form.billingAccountName"
            title="Billing Account Name"
            login-mode
            required
            placeholder="Your billing account"
            @error="setFieldInError('billingAccountName', $event)"
          />

          <SiTextBox
            id="userEmail"
            v-model="form.userEmail"
            title="Email Address"
            login-mode
            required
            placeholder="Your email"
            :validations="[
              {
                id: 'email',
                message: 'Must be a valid email address.',
                check: validator.isEmail,
              },
            ]"
            @error="setFieldInError('userEmail', $event)"
          />

          <SiTextBox
            id="userPassword"
            v-model="form.userPassword"
            title="Password"
            password
            login-mode
            required
            placeholder="Your password"
            autocomplete="current-password"
            @error="setFieldInError('userPassword', $event)"
          />

          <button
            type="submit"
            data-test="login"
            aria-label="Sign In"
            class="w-full flex justify-center py-2 px-4 border border-transparent rounded-sm shadow-sm text-sm font-medium text-white bg-action-500 hover:bg-action-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-action-400 disabled:opacity-50"
            :disabled="formInError"
          >
            Log In
          </button>
        </form>
      </div>

      <div
        class="border-t-2 border-black text-white text-center font-medium bg-neutral-900 pt-4 pb-8 px-4 shadow sm:rounded-b-md sm:px-10"
      >
        <div class="mb-4">Don't have an SI account?</div>

        <router-link
          :to="{ name: 'signup' }"
          class="w-full flex justify-center py-2 px-4 border border-action-500 rounded-sm shadow-sm text-sm font-bold text-action-500 hover:bg-action-50 hover:text-action-600 hover:border-action-600 focus:bg-action-100 focus:border-action-700 focus:text-action-700"
        >
          Create An Account!
        </router-link>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import validator from "validator";
import { SessionService } from "@/service/session";
import siLogoWts from "@/assets/images/si-logo-wts.svg?url";
import SiTextBox from "@/atoms/SiTextBox.vue";
import { useFieldErrors } from "@/composables/useFieldErrors";
import { setFormSettings } from "@/composables/formSettings";

const form = ref({
  billingAccountName: "",
  userEmail: "",
  userPassword: "",
});

setFormSettings({
  hideRequiredLabel: false,
  requiredLabel: "*",
  requiredLabelClasses: "text-destructive-500",
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
