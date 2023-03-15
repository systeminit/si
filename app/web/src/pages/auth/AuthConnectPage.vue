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
import { onMounted } from "vue";
import { useRouter, useRoute } from "vue-router";
import { useHead } from "@vueuse/head";

import AppLayout from "@/components/layout/AppLayout.vue";
import Card from "@/ui-lib/Card.vue";
import ErrorMessage from "@/ui-lib/ErrorMessage.vue";

import Stack from "@/ui-lib/layout/Stack.vue";
import { useAuthStore } from "@/store/auth.store";

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

onMounted(async () => {
  // if no code in query, we just bail and an error will be displayed
  if (!connectCode) return;

  const connectReq = await authStore.AUTH_CONNECT({ code: connectCode });
  if (connectReq.result.success) {
    const workspacePk = connectReq.result.data.workspace.pk;

    // TODO: we probably want to allow passing in a more specific URL to redirect to
    // in case they tried to access that URL and were then redirected to login
    await router.push({ name: "workspace-single", params: { workspacePk } });
  }
});
</script>
