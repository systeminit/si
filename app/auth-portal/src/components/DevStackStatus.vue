<template>
  <div>
    <Icon :name="onboardingStore.devFrontendOnline ? 'check' : 'loader'" />
    <Icon :name="onboardingStore.devBackendOnline ? 'check' : 'loader'" />
  </div>
</template>

<script setup lang="ts">
import { Icon } from "@si/vue-lib/design-system";

import { useOnboardingStore } from "@/store/onboarding.store";

const onboardingStore = useOnboardingStore();
</script>
