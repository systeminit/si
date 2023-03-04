<template>
  <AppLayout page-mode="modal" class="font-medium">
    <Stack spacing="lg" class="max-w-md">
      <AuthPageHeader title="Authenticating">
        Validating your credentials
      </AuthPageHeader>

      <Card rounded>
        <RichText>
          <template v-if="!connectCode">
            <ErrorMessage>Invalid auth attempt</ErrorMessage>
            <a :href="LOGIN_URL">Return to login</a>
          </template>
          <template v-else-if="authReqStatus.isError">
            <ErrorMessage :request-status="authReqStatus" />
            <a :href="LOGIN_URL">Return to login</a>
          </template>
          <template v-else-if="authReqStatus.isPending">
            Checking credentials...
          </template>
          <template v-else-if="authReqStatus.isSuccess">
            Hooray you logged in!
          </template>
        </RichText>
      </Card>
    </Stack>
  </AppLayout>
</template>

<script setup lang="ts">
import { computed, onBeforeMount } from "vue";
import { useRouter, useRoute } from "vue-router";
import { useHead } from "@vueuse/head";

import AppLayout from "@/components/layout/AppLayout.vue";
import Card from "@/ui-lib/Card.vue";
import VormInput from "@/ui-lib/forms/VormInput.vue";
import ErrorMessage from "@/ui-lib/ErrorMessage.vue";
import VButton2 from "@/ui-lib/VButton2.vue";
import Stack from "@/ui-lib/layout/Stack.vue";
import { useAuthStore } from "@/store/auth.store";
import { VALID_USERNAME_REGEX } from "@/utils/input_validations";
import RichText from "@/ui-lib/RichText.vue";
import AuthPageHeader from "./AuthPageHeader.vue";

const router = useRouter();
const route = useRoute();

const AUTH_PORTAL_URL = import.meta.env.VITE_AUTH_PORTAL_URL;
const LOGIN_URL = `${AUTH_PORTAL_URL}/login`;

useHead({ title: "Authenticating..." });

const authStore = useAuthStore();
const authReqStatus = authStore.getRequestStatus("AUTH_CONNECT");

// grab the code from the URL, dont need to care about reactivity as it will not change while on the page
const connectCode = route.query.code as string;

onBeforeMount(() => {
  if (connectCode) authStore.AUTH_CONNECT({ code: connectCode });
});

// const redirectAfterLogin = computed(() => route.query.redirect as string);

// function onLoginSuccess() {
//   router.push(redirectAfterLogin.value || { name: "home" });
// }
</script>
