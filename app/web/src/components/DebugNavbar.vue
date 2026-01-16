<template>
  <div class="w-full bg-neutral-900 text-shade-0 flex flex-row justify-end items-center h-[60px]">
    <SiLogo class="block h-[44px] w-[44px] ml-[12px] mr-[12px] flex-none" />
    <div class="flex-grow px-sm font-bold text-lg">SVG DEBUG PAGE</div>
    <SiThemeSwitcher />
    <NavbarButton :linkTo="{ name: 'workspace-dev-dashboard' }" tooltipText="Back to Dev Dashboard">
      <Icon name="cat" />
    </NavbarButton>
  </div>
</template>

<script setup lang="ts">
import { Icon, useThemeContainer } from "@si/vue-lib/design-system";
import SiLogo from "@si/vue-lib/brand-assets/si-logo-symbol.svg?component";
import SiThemeSwitcher from "@/components/layout/navbar/NavbarThemeSwitcher.vue";
import NavbarButton from "./layout/navbar/NavbarButton.vue";

useThemeContainer("dark");
</script>
