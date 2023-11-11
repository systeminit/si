<template>
  <div class="flex justify-center mx-auto flex-none">
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
  </div>
</template>

<script setup lang="ts">
import { useRoute } from "vue-router";
import { Icon } from "@si/vue-lib/design-system";
import NavbarButton from "./NavbarButton.vue";

const route = useRoute();
</script>
