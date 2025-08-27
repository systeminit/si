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
      v-if="!onHead"
      tooltipText="Review"
      icon="eye"
      :linkTo="{
        path: `/n/${changeSetStore.selectedWorkspacePk}/${changeSetStore.selectedChangeSetId}/h/r`,
      }"
    />
    <NavbarButton
      v-else
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
import { computed } from "vue";
import { useChangeSetsStore } from "@/store/change_sets.store";
import NavbarButton from "./NavbarButton.vue";

const route = useRoute();
const changeSetStore = useChangeSetsStore();

const onHead = computed(() => changeSetStore.headSelected);

const modelingLink = () => {
  if (changeSetStore.selectedChangeSetId) {
    return {
      name: "new-hotness",
      params: { changeSetId: changeSetStore.selectedChangeSetId },
    };
  }
  return {
    name: "new-hotness",
    params: { changeSetId: "auto" },
  };
};
</script>
