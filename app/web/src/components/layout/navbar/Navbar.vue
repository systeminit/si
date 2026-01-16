<template>
  <nav
    :class="
      clsx(
        'navbar bg-neutral-900 text-white relative shadow-[0_4px_4px_0_rgba(0,0,0,0.15)]',
        'z-90 h-[60px] overflow-hidden shrink-0 flex flex-row justify-between select-none',
        windowWidth > 740 && 'gap-sm',
      )
    "
  >
    <!-- Left side -->
    <NavbarPanelLeft :invalidWorkspace="invalidWorkspace" />

    <!-- Center -->
    <NavbarPanelCenter v-if="!invalidWorkspace" />

    <!-- Right -->
    <NavbarPanelRight :invalidWorkspace="invalidWorkspace" />
  </nav>
</template>

<script setup lang="ts">
import { useThemeContainer } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { useWorkspacesStore } from "@/store/workspaces.store";
import NavbarPanelCenter from "./NavbarPanelCenter.vue";
import NavbarPanelRight from "./NavbarPanelRight.vue";
import NavbarPanelLeft from "./NavbarPanelLeft.vue";

const workspacesStore = useWorkspacesStore();

const invalidWorkspace = computed(
  () => !!(workspacesStore.urlSelectedWorkspaceId && !workspacesStore.selectedWorkspace),
);

// top bar is always dark, so this keeps the workspace and change set dropdowns looking correct
useThemeContainer("dark");

const windowWidth = ref(window.innerWidth);

const windowResizeHandler = () => {
  windowWidth.value = window.innerWidth;
};

onMounted(() => {
  windowResizeHandler();
  window.addEventListener("resize", windowResizeHandler);
});
onBeforeUnmount(() => {
  window.removeEventListener("resize", windowResizeHandler);
});
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
