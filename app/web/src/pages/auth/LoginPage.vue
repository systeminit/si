<template>
  <AppLayout page-mode="modal" class="font-medium">
    <Stack spacing="lg" class="max-w-md">
      <AuthPageHeader title="Log In">
        Don't have an account?
        <RouterLink :to="{ name: 'signup' }">Create One!</RouterLink>
      </AuthPageHeader>

      <form @submit.prevent="tryPasswordLogin">
        <Card rounded>
          <Stack spacing="md">
            <ErrorMessage :request-status="loginReqStatus" />
            <VormInput
              v-model="loginPayload.workspaceName"
              label="Workspace Name"
              required
              placeholder="ex: initech"
              to-lower-case
              :regex="VALID_USERNAME_REGEX"
              regex-message="Invalid workspace name"
            />
            <VormInput
              v-model="loginPayload.userEmail"
              type="email"
              label="Email"
              required
            />
            <VormInput
              v-model="loginPayload.userPassword"
              type="password"
              label="Password"
              required
              autocomplete="current-password"
            />

            <VButton2
              :disabled="validationState.isError"
              :request-status="loginReqStatus"
              loading-text="Logging you in"
              submit
            >
              Log In
            </VButton2>
          </Stack>

          <template #footer>
            <Stack>
              <p class="text-center">Don't have an SI account?</p>
              <VButton2 link-to-named-route="signup" variant="ghost">
                Create An Account!
              </VButton2>
            </Stack>
          </template>
        </Card>
      </form>
    </Stack>
  </AppLayout>
</template>

<script setup lang="ts">
import { computed, reactive } from "vue";
import { useRouter, useRoute } from "vue-router";
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

const router = useRouter();
const route = useRoute();

useHead({ title: "Log in" });

const { validationState, validationMethods } = useValidatedInputGroup();

const devAutofill = true;

// local dev prefills signup and login
const loginPayload = reactive({
  workspaceName: import.meta.env.DEV && devAutofill ? "systeminit" : "",
  userEmail: import.meta.env.DEV && devAutofill ? "sally@systeminit.com" : "",
  userPassword: import.meta.env.DEV && devAutofill ? "Password123!" : "",
});

const authStore = useAuthStore();
const loginReqStatus = authStore.getRequestStatus("LOGIN");

async function tryPasswordLogin() {
  // touch all inputs, bail if we have any errors
  if (validationMethods.hasError()) return;

  const req = await authStore.LOGIN(loginPayload);
  if (req.result.success) onLoginSuccess();
}

const redirectAfterLogin = computed(() => route.query.redirect as string);

function onLoginSuccess() {
  router.push(redirectAfterLogin.value || { name: "home" });
}
</script>
