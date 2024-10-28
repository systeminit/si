<template>
  <div
    class="flex items-center justify-center place-items-center mx-auto h-full flex-none"
  >
    <NavbarButton
      tooltipText="Model"
      icon="diagram"
      :selected="route.name === 'workspace-compose'"
      :linkTo="{
        name: 'workspace-compose',
        params: { changeSetId: 'auto' },
      }"
    />

    <NavbarButton
      tooltipText="Customize"
      icon="beaker"
      :selected="route.matched.some((r) => r.name === 'workspace-lab')"
      :linkTo="{
        name: 'workspace-lab',
        params: { changeSetId: 'auto' },
      }"
    />

    <NavbarButton
      v-if="featureFlagsStore.AUDIT_PAGE"
      tooltipText="Audit"
      icon="eye"
      :selected="route.matched.some((r) => r.name === 'workspace-audit')"
      :linkTo="{
        name: 'workspace-audit',
        params: { changeSetId: 'auto' },
      }"
    />
  </div>
</template>

<script setup lang="ts">
import { useRoute } from "vue-router";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import NavbarButton from "./NavbarButton.vue";

const route = useRoute();
const featureFlagsStore = useFeatureFlagsStore();
</script>
