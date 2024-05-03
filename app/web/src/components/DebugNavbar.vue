<template>
  <div
    class="w-full bg-neutral-900 text-white flex flex-row justify-end items-center"
  >
    <div class="flex-grow px-sm font-bold text-lg">SVG DEBUG PAGE</div>
    <SiThemeSwitcher />
    <NavbarButton
      :linkTo="{ name: 'workspace-dev-dashboard' }"
      tooltipText="Back to Dev Dashboard"
    >
      <Icon name="cat" />
    </NavbarButton>
  </div>
</template>

<script setup lang="ts">
import { Icon, useThemeContainer } from "@si/vue-lib/design-system";
import SiThemeSwitcher from "@/components/layout/navbar/NavbarThemeSwitcher.vue";
import NavbarButton from "./layout/navbar/NavbarButton.vue";

useThemeContainer("dark");
</script>
