<template>
  <AppLayout page-mode="modal" class="font-medium">
    <Stack spacing="l" class="max-w-md">
      <div class="text-neutral-200 flex flex-row gap-m px-l">
        <img class="h-16" :src="siLogoWts" alt="System Initiative" />
        <Stack spacing="m">
          <h2 class="text-4xl font-extrabold capsize">Log In</h2>
          <RichText class="capsize">
            Don't have an account?
            <router-link :to="{ name: 'signup' }">Create one!</router-link>
          </RichText>
        </Stack>
      </div>

      <form @submit.prevent="onSubmit">
        <Card rounded>
          <Stack spacing="m">
            <ErrorMessage :message="errorMessage" />
            <VormInput
              v-model="loginPayload.billingAccountName"
              label="Billing Account Name"
              required
              placeholder="ex: initech"
              to-lower-case
              :regex="/^[a-z0-9._-]+$/i"
              regex-message="Invalid billing account name"
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

            <VButton2 :disabled="validationState.isError" submit>
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
import { reactive, ref } from "vue";
import { useRouter, RouterLink } from "vue-router";
import { useHead } from "@vueuse/head";

import { SessionService } from "@/service/session";
import siLogoWts from "@/assets/images/si-logo-wts.svg?url";
import AppLayout from "@/templates/AppLayout.vue";
import Card from "@/ui-lib/Card.vue";
import VormInput from "@/ui-lib/forms/VormInput.vue";
import { useValidatedInputGroup } from "@/ui-lib/forms/helpers/form-validation";
import ErrorMessage from "@/ui-lib/ErrorMessage.vue";
import VButton2 from "@/ui-lib/VButton2.vue";
import Stack from "@/ui-lib/layout/Stack.vue";
import RichText from "@/ui-lib/RichText.vue";

useHead({ title: "Login" });

const router = useRouter();

const { validationState, validationMethods } = useValidatedInputGroup();

// local dev prefills signup and login
const loginPayload = reactive({
  billingAccountName: import.meta.env.DEV ? "systeminit" : "",
  userEmail: import.meta.env.DEV ? "example@systeminit.com" : "",
  userPassword: import.meta.env.DEV ? "Password123!" : "",
});

const errorMessage = ref<string>();

function onSubmit() {
  // touch all inputs, bail if we have any errors
  if (validationMethods.hasError()) return;

  SessionService.login(loginPayload).subscribe((response) => {
    if (!response || response.error) {
      errorMessage.value = "Login error; please try again!";
    } else {
      router.push({ name: "home" });
    }
  });
}
</script>
