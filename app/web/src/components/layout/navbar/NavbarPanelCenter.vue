<template>
  <div
    class="flex flex-row flex-none items-center h-full justify-center place-items-center mx-auto overflow-hidden"
  >
    <NavbarButton
      tooltipText="Model"
      icon="grid"
      :selected="
        ['workspace-compose', 'workspace-compose-view'].includes(
          route.name as string,
        )
      "
      :linkTo="modelingLink()"
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
import { useViewsStore } from "@/store/views.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import NavbarButton from "./NavbarButton.vue";

const route = useRoute();

const modelingLink = () => {
  const viewsStore = useViewsStore();
  const changeSetStore = useChangeSetsStore();
  const ffStore = useFeatureFlagsStore();
  const prefix = ffStore.ENABLE_NEW_EXPERIENCE
    ? "new-hotness"
    : "workspace-compose";
  if (changeSetStore.selectedChangeSetId) {
    if (viewsStore.selectedViewId) {
      return {
        name: `${prefix}-view`,
        params: {
          changeSetId: changeSetStore.selectedChangeSetId,
          viewId: viewsStore.selectedViewId,
        },
      };
    } else {
      return {
        name: prefix,
        params: { changeSetId: changeSetStore.selectedChangeSetId },
      };
    }
  }
  return {
    name: prefix,
    params: { changeSetId: "auto" },
  };
};
</script>
