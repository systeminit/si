<template>
  <AppLayout page-mode="modal" class="font-medium">
    <Stack spacing="lg" class="max-w-md">
      <AuthPageHeader title="Logging out"></AuthPageHeader>

      <Card rounded> See ya later! </Card>
    </Stack>
  </AppLayout>
</template>

<script setup lang="ts">
import { onMounted } from "vue";
import { useRouter, useRoute } from "vue-router";
import { useHead } from "@vueuse/head";

import { Card, ErrorMessage, Stack, RichText } from "@si/vue-lib/design-system";
import AppLayout from "@/components/layout/AppLayout.vue";

import { useAuthStore } from "@/store/auth.store";

import AuthPageHeader from "./AuthPageHeader.vue";

const AUTH_PORTAL_URL = import.meta.env.VITE_AUTH_PORTAL_URL;

useHead({ title: "Logging you out..." });

const authStore = useAuthStore();
</script>
