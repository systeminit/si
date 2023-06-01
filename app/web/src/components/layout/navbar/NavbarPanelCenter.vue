<template>
  <div
    class="flex items-center justify-center place-items-center mx-auto h-full"
  >
    <NavbarButton
      tooltip-text="Model"
      :selected="route.name === 'workspace-compose'"
      :link-to="{
        name: 'workspace-compose',
        params: { changeSetId: 'auto' },
      }"
    >
      <Icon name="diagram" />
    </NavbarButton>

    <NavbarButton
      tooltip-text="Customize"
      :selected="route.matched.some((r) => r.name === 'workspace-lab')"
      :link-to="{
        name: 'workspace-lab',
        params: { changeSetId: 'auto' },
      }"
    >
      <Icon name="beaker" />
    </NavbarButton>

    <!-- Vertical bar -->
    <div
      v-if="!SINGLE_MODEL_SCREEN_FF"
      class="w-0.5 h-8 self-center mx-xs bg-white"
    ></div>

    <NavbarButton
      v-if="!SINGLE_MODEL_SCREEN_FF"
      tooltip-text="Apply"
      :selected="route.name === 'workspace-fix'"
      :link-to="{ name: 'workspace-fix' }"
    >
      <Icon name="tools" />
    </NavbarButton>

    <NavbarButton
      v-if="!SINGLE_MODEL_SCREEN_FF"
      tooltip-text="Analyze"
      :selected="route.name === 'workspace-view'"
      :link-to="{ name: 'workspace-view' }"
    >
      <Icon name="eye" />
    </NavbarButton>
  </div>
</template>

<script setup lang="ts">
import { useRoute } from "vue-router";
import { Icon } from "@si/vue-lib/design-system";
import { SINGLE_MODEL_SCREEN_FF } from "@/utils/feature_flags";
import NavbarButton from "./NavbarButton.vue";

const route = useRoute();
</script>
