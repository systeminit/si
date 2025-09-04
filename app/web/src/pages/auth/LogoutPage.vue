<template>
  <AppLayout pageMode="modal" class="font-medium">
    <Stack spacing="lg" class="max-w-md">
      <AuthPageHeader title="Logging out"></AuthPageHeader>
      <Card rounded> See ya later!</Card>
    </Stack>
  </AppLayout>
</template>

<script setup lang="ts">
import { Card, Stack } from "@si/vue-lib/design-system";
import AppLayout from "@/components/layout/AppLayout.vue";

import AuthPageHeader from "./AuthPageHeader.vue";
</script>
