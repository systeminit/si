<template>
  <div
    class="flex items-center justify-center place-items-center mx-auto h-full"
  >
    <NavbarButton
      tooltipText="Model"
      :selected="route.name === 'workspace-compose'"
      :linkTo="{
        name: 'workspace-compose',
        params: { changeSetId: 'auto' },
      }"
    >
      <Icon name="diagram" />
    </NavbarButton>

    <NavbarButton
      tooltipText="Customize"
      :selected="route.matched.some((r) => r.name === 'workspace-lab')"
      :linkTo="{
        name: 'workspace-lab',
        params: { changeSetId: 'auto' },
      }"
    >
      <Icon name="beaker" />
    </NavbarButton>

    <!-- Vertical bar -->
    <div
      v-if="!featureFlagsStore.SINGLE_MODEL_SCREEN"
      class="w-0.5 h-8 self-center mx-xs bg-white"
    ></div>

    <NavbarButton
      v-if="!featureFlagsStore.SINGLE_MODEL_SCREEN"
      tooltipText="Apply"
      :selected="route.name === 'workspace-fix'"
      :linkTo="{ name: 'workspace-fix' }"
    >
      <Icon name="tools" />
    </NavbarButton>

    <NavbarButton
      v-if="!featureFlagsStore.SINGLE_MODEL_SCREEN"
      tooltipText="Analyze"
      :selected="route.name === 'workspace-view'"
      :linkTo="{ name: 'workspace-view' }"
    >
      <Icon name="eye" />
    </NavbarButton>
  </div>
</template>

<script setup lang="ts">
import { useRoute } from "vue-router";
import { Icon } from "@si/vue-lib/design-system";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import NavbarButton from "./NavbarButton.vue";

const featureFlagsStore = useFeatureFlagsStore();

const route = useRoute();
</script>
