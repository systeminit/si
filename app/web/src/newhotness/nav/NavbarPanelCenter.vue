<template>
  <div
    class="flex flex-row flex-none items-center h-full justify-center place-items-center mx-auto overflow-hidden"
  >
    <NavbarButton
      tooltipText="Compose"
      icon="grid"
      :selected="route.name?.toString().startsWith('new-hotness')"
      :linkTo="compositionLink"
    />

    <NavbarButton
      tooltipText="Customize"
      icon="beaker"
      :selected="route.matched.some((r) => r.name === 'workspace-lab')"
      :linkTo="{
        path: `/w/${workspaceId}/${changeSetId}/l`,
      }"
    />

    <NavbarButton
      tooltipText="Audit"
      icon="eye"
      :selected="route.matched.some((r) => r.name === 'workspace-audit')"
      :linkTo="{
        path: `/w/${workspaceId}/${changeSetId}/a`,
      }"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useRoute } from "vue-router";
import NavbarButton from "@/components/layout/navbar/NavbarButton.vue";

const route = useRoute();

const props = defineProps<{
  workspaceId: string;
  changeSetId: string;
  componentId?: string;
  viewId?: string;
}>();

const compositionLink = computed(() => {
  // eslint-disable-next-line no-nested-ternary
  const name = props.componentId
    ? "new-hotness-component"
    : props.viewId
    ? "new-hotness-view"
    : "new-hotness";
  return {
    name,
    params: props,
  };
});
</script>
