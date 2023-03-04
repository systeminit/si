<template>
  <AppLayout page-mode="modal" class="font-medium">
    <Stack spacing="lg" class="max-w-md">
      <AuthPageHeader title="Sign Up">
        Already have an account?
        <RouterLink :to="{ name: 'login' }">Log in!</RouterLink>
      </AuthPageHeader>

      <form @submit.prevent="trySignup">
        <Card rounded>
          <Stack spacing="md">
            <ErrorMessage :request-status="signupReqStatus" />
            <VormInput
              v-model="signupPayload.workspaceName"
              label="Workspace Name"
              required
              placeholder="ex: acmecorp"
              instructions="An org-level username - you will need this when you sign in"
              to-lower-case
              :regex="VALID_USERNAME_REGEX"
              regex-message="Please use only lowercase letters, digits, and _-."
            />
            <VormInput
              v-model="signupPayload.userName"
              type="text"
              label="Your Full Name"
              required
              placeholder="ex: John Smith"
            />

            <VormInput
              v-model="signupPayload.userEmail"
              type="email"
              label="Your Email"
              required
              placeholder="ex: john.smith@acmecorp.co"
              instructions="You'll use this to sign in"
            />
            <VormInput
              v-model="signupPayload.userPassword"
              type="password"
              label="Password"
              required
              allow-show-password
              placeholder="Select a new password"
              autocomplete="new-password"
              instructions="8-64 characters, must include lowercase, uppercase, number, and symbol"
              :min-length="8"
              :max-length="64"
            />

            <VormInput
              ref="agentInputRef"
              v-model="signupPayload.signupSecret"
              type="password"
              label="Agent Passphrase"
              required
              placeholder="provided to you by SI"
              @focus="checkDuplicatePassword"
            />

            <VButton2
              :disabled="validationState.isError"
              :request-status="signupReqStatus"
              loading-text="Creating your account"
              submit
            >
              Create Account
            </VButton2>
          </Stack>

          <template #footer>
            <Stack>
              <p class="text-center">Already have an account?</p>
              <VButton2 link-to-named-route="login" variant="ghost">
                Log In!
              </VButton2>
            </Stack>
          </template>
        </Card>
      </form>
    </Stack>
  </AppLayout>
</template>

<script setup lang="ts">
import { reactive, ref } from "vue";
import { useRouter } from "vue-router";
import { useHead } from "@vueuse/head";

import AppLayout from "@/components/layout/AppLayout.vue";
import Card from "@/ui-lib/Card.vue";
import VormInput from "@/ui-lib/forms/VormInput.vue";
import { useValidatedInputGroup } from "@/ui-lib/forms/helpers/form-validation";
import ErrorMessage from "@/ui-lib/ErrorMessage.vue";
import VButton2 from "@/ui-lib/VButton2.vue";
import Stack from "@/ui-lib/layout/Stack.vue";
import { useAuthStore } from "@/store/auth.store";
import { VALID_USERNAME_REGEX } from "@/utils/input_validations";
import AuthPageHeader from "./AuthPageHeader.vue";

useHead({ title: "Sign Up" });

const router = useRouter();

const { validationState, validationMethods } = useValidatedInputGroup();

const devAutofill = true;

// local dev prefills signup and login
const signupPayload = reactive({
  workspaceName: import.meta.env.DEV && devAutofill ? "systeminit" : "",
  userName: import.meta.env.DEV && devAutofill ? "Signup Sally" : "",
  userEmail: import.meta.env.DEV && devAutofill ? "sally@systeminit.com" : "",
  userPassword: import.meta.env.DEV && devAutofill ? "Password123!" : "",
  signupSecret: import.meta.env.DEV && devAutofill ? "cool-steam" : "",
});

// Check if a password manager put the password in both fields
const agentInputRef = ref<InstanceType<typeof VormInput>>();
const checkDuplicatePassword = () => {
  if (signupPayload.userPassword === signupPayload.signupSecret) {
    signupPayload.signupSecret = "";

    // this removes the 1password highlight from the Agent Passphrase field
    agentInputRef.value?.inputRef?.removeAttribute(
      "data-com-onepassword-filled",
    );
  }
};

const authStore = useAuthStore();
const signupReqStatus = authStore.getRequestStatus("SIGNUP");

async function trySignup() {
  // touch all inputs, bail if we have any errors
  if (validationMethods.hasError()) return;

  const req = await authStore.SIGNUP(signupPayload);
  if (req.result.success) router.push({ name: "login" });
}
</script>
