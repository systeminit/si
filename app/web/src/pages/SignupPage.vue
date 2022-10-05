<template>
  <AppLayout page-mode="modal" class="font-medium">
    <Stack spacing="lg" class="max-w-md">
      <div class="text-neutral-200 flex flex-row gap-md px-lg">
        <img class="h-16" :src="siLogoWts" alt="System Initiative" />
        <Stack spacing="md">
          <h2 class="text-4xl font-extrabold capsize">Sign Up</h2>
          <RichText class="capsize">
            Already have an account?
            <router-link :to="{ name: 'login' }">Log in!</router-link>
          </RichText>
        </Stack>
      </div>

      <form @submit.prevent="trySignup">
        <Card rounded>
          <Stack spacing="md">
            <ErrorMessage :request-status="signupReqStatus" />
            <VormInput
              v-model="signupPayload.billingAccountName"
              label="Billing Account Name"
              required
              placeholder="ex: acmecorp"
              instructions="An org-level username - you will need this when you sign in"
              to-lower-case
              :regex="/^[a-z0-9._-]+$/i"
              regex-message="only letters, digits, and ._-"
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
              placeholder="Select a new password"
              autocomplete="new-password"
              instructions="8-64 characters, must include lowercase, uppercase, number, and symbol"
              :min-password-length="8"
              :max-password-length="64"
            />

            <VormInput
              v-model="signupPayload.signupSecret"
              type="password"
              label="Agent Passphrase"
              required
              placeholder="provided to you by SI"
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
import { reactive } from "vue";
import { useRouter, RouterLink } from "vue-router";
import { useHead } from "@vueuse/head";

import siLogoWts from "@/assets/images/si-logo-wts.svg?url";
import AppLayout from "@/templates/AppLayout.vue";
import Card from "@/ui-lib/Card.vue";
import VormInput from "@/ui-lib/forms/VormInput.vue";
import { useValidatedInputGroup } from "@/ui-lib/forms/helpers/form-validation";
import ErrorMessage from "@/ui-lib/ErrorMessage.vue";
import VButton2 from "@/ui-lib/VButton2.vue";
import Stack from "@/ui-lib/layout/Stack.vue";
import RichText from "@/ui-lib/RichText.vue";
import { useAuthStore } from "@/store/auth.store";

useHead({ title: "Sign Up" });

const router = useRouter();

const { validationState, validationMethods } = useValidatedInputGroup();

// local dev prefills signup and login
const signupPayload = reactive({
  billingAccountName: import.meta.env.DEV ? "systeminit" : "",
  userName: import.meta.env.DEV ? "System Init" : "",
  userEmail: import.meta.env.DEV ? "example@systeminit.com" : "",
  userPassword: import.meta.env.DEV ? "Password123!" : "",
  signupSecret: import.meta.env.DEV ? "cool-steam" : "",
});

const authStore = useAuthStore();
const signupReqStatus = authStore.getRequestStatus("SIGNUP");

async function trySignup() {
  // touch all inputs, bail if we have any errors
  if (validationMethods.hasError()) return;

  const req = await authStore.SIGNUP(signupPayload);
  if (req.result.success) router.push({ name: "login" });
}
</script>
