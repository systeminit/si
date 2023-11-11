<template>
  <nav
    class="bg-neutral-900 text-white relative border-b border-shade-100 shadow-[0_4px_4px_0_rgba(0,0,0,0.15)] z-90 pl-sm flex gap-sm"
  >
    <NavbarPanelLeft />
    <NavbarPanelCenter />
    <NavbarPanelRight />
  </nav>
</template>

<script setup lang="ts">
import NavbarPanelCenter from "./NavbarPanelCenter.vue";
import NavbarPanelRight from "./NavbarPanelRight.vue";
import NavbarPanelLeft from "./NavbarPanelLeft.vue";
</script>
