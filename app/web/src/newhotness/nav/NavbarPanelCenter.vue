<template>
  <div
    class="flex flex-row flex-none items-center h-full justify-center place-items-center mx-auto overflow-hidden"
  >
    <template v-if="isConnected">
      <NavbarButton
        tooltipText="Compose"
        icon="grid"
        :selected="
          route.name?.toString().startsWith('new-hotness') &&
          route.name?.toString() !== 'new-hotness-review'
        "
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
        v-if="!ctx.onHead.value"
        tooltipText="Review"
        icon="eye"
        :selected="route.matched.some((r) => r.name === 'new-hotness-review')"
        :linkTo="{
          path: `/n/${workspaceId}/${changeSetId}/h/r`,
        }"
      />
      <NavbarButton
        v-else
        tooltipText="Audit"
        icon="eye"
        :selected="route.matched.some((r) => r.name === 'workspace-audit')"
        :linkTo="{
          path: `/w/${workspaceId}/${changeSetId}/a`,
        }"
      />
    </template>
    <div
      v-else
      :class="
        clsx(
          'p-md text-destructive-500 flex flex-row gap-sm items-center',
          'animate-[pulse_2s_infinite]',
          themeClasses(
            'bg-destructive-100 text-destructive-900',
            'bg-destructive-900 text-destructive-100',
          ),
        )
      "
    >
      <Icon name="alert-square" size="md" /> Lost Connection, retrying...
    </div>
  </div>
</template>

<script setup lang="ts">
import { clsx } from "clsx";
import { computed } from "vue";
import { useRoute } from "vue-router";
import { themeClasses, Icon } from "@si/vue-lib/design-system";
import NavbarButton from "@/components/layout/navbar/NavbarButton.vue";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { useContext } from "../logic_composables/context";

const ctx = useContext();
const route = useRoute();
const ffStore = useFeatureFlagsStore();

const isConnected = computed(() => {
  if (ffStore.SHOW_WS_DISCONNECT) return props.connected;
  return true;
});

const props = defineProps<{
  workspaceId: string;
  changeSetId: string;
  componentId?: string;
  viewId?: string;
  connected: boolean;
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
