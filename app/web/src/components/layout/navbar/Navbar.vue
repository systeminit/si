<template>
  <nav
    class="navbar bg-neutral-900 text-white relative shadow-[0_4px_4px_0_rgba(0,0,0,0.15)] z-90 h-[60px] overflow-hidden shrink-0 flex gap-sm"
  >
    <!-- Left side -->
    <NavbarPanelLeft />

    <!-- Center -->
    <NavbarPanelCenter />

    <!-- Right -->
    <NavbarPanelRight />
  </nav>
</template>

<script setup lang="ts">
import { useThemeContainer } from "@si/vue-lib/design-system";
import NavbarPanelCenter from "./NavbarPanelCenter.vue";
import NavbarPanelRight from "./NavbarPanelRight.vue";
import NavbarPanelLeft from "./NavbarPanelLeft.vue";

// top bar is always dark, so this keeps the workspace/changeset dropdowns looking correct
useThemeContainer("dark");
</script>

<style lang="less">
.navbar {
  &:after {
    content: "";
    position: absolute;
    bottom: 0px;
    width: 100%;
    height: 1px;
    background: black;
  }
}
</style>
