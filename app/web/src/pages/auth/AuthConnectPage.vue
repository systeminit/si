<template>
  <AppLayout class="font-medium" pageMode="modal">
    <Stack class="max-w-md" spacing="lg">
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
            <ErrorMessage :requestStatus="authReqStatus" />
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

<script lang="ts" setup>
import { onMounted } from "vue";
import { useRouter, useRoute } from "vue-router";

import { Card, ErrorMessage, Stack, RichText } from "@si/vue-lib/design-system";
import AppLayout from "@/components/layout/AppLayout.vue";

import { useAuthStore } from "@/store/auth.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";

import AuthPageHeader from "./AuthPageHeader.vue";

const router = useRouter();
const route = useRoute();

const AUTH_PORTAL_URL = import.meta.env.VITE_AUTH_PORTAL_URL;
const LOGIN_URL = `${AUTH_PORTAL_URL}/login`;

const authStore = useAuthStore();
const featureFlagStore = useFeatureFlagsStore();
const authReqStatus = authStore.getRequestStatus("AUTH_CONNECT");

// grab the code from the URL, don't need to care about reactivity as it will not change while on the page
const connectCode = route.query.code as string;
const redirectPath = route.query.redirect as string;
const onDemandAssets = route.query.onDemandAssets === "true";

onMounted(async () => {
  // if no code in query, we just bail and an error will be displayed
  if (!connectCode) return;

  const connectReq = await authStore.AUTH_CONNECT({
    code: connectCode,
    onDemandAssets,
  });

  if (connectReq.result.success) {
    const workspacePk = connectReq.result.data.workspace.pk;

    const flagsLoaded = new Promise((resolve) => {
      const id = setInterval(() => {
        if (featureFlagStore.ENABLE_NEW_EXPERIENCE !== undefined) {
          clearInterval(id);
          resolve(null);
        }
      }, 50);
    });

    await flagsLoaded;
    let routeName = "workspace-single";
    if (featureFlagStore.ENABLE_NEW_EXPERIENCE) {
      routeName = "new-hotness-workspace";
    }

    const redirectObject = redirectPath
      ? { path: redirectPath }
      : { name: routeName, params: { workspacePk } };

    await router.replace(redirectObject);
  }
});
</script>
