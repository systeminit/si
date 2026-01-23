<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <template v-if="isConnected">
    <div v-if="ffStore.SHOW_POLICIES" class="flex flex-row items-end gap-xs p-xs">
      <NewButton
        tone="navFlat"
        @click="() => router.push(compositionLink)"
        class="py-3xs"
        :active="onSim"
        :label="onHead ? 'Model' : 'Simulation'"
      />
      <NewButton
        v-if="ffStore.SHOW_AUTHORING_NAV"
        tone="navFlat"
        class="py-3xs"
        :active="onAuthor"
        @click="
          () => {
            router.push({
              path: `/w/${workspaceId}/${changeSetId}/l`,
            });
          }
        "
        label="Author"
      />
      <NewButton
        v-if="onHead"
        tone="navFlat"
        class="py-3xs"
        :active="onAudit"
        @click="
          () => {
            router.push({
              path: `/w/${workspaceId}/${changeSetId}/a`,
            });
          }
        "
        label="Audit"
      />
      <NewButton
        v-else
        tone="navFlat"
        class="py-3xs"
        :active="onReview"
        @click="
          () => {
            router.push({
              path: `/n/${workspaceId}/${changeSetId}/h/r`,
            });
          }
        "
        label="Review"
      />
      <NewButton
        tone="navFlat"
        class="py-3xs"
        :active="onPolicy"
        @click="
          () => {
            router.push({
              path: `/n/${workspaceId}/${changeSetId}/h/p`,
            });
          }
        "
        label="Policy"
      />
    </div>
    <template v-else>
      <div
        class="flex flex-row flex-none items-center h-full justify-center place-items-center mx-auto overflow-hidden"
      >
        <NavbarButton
          tooltipText="Compose"
          icon="grid"
          :selected="
            route.name?.toString().startsWith('new-hotness') && route.name?.toString() !== 'new-hotness-review'
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
          v-if="!onHead"
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
      </div>
    </template>
  </template>
  <div
    v-else
    :class="
      clsx(
        'p-md text-destructive-500 flex flex-row gap-sm items-center',
        'animate-[pulse_2s_infinite]',
        themeClasses('bg-destructive-100 text-destructive-900', 'bg-destructive-900 text-destructive-100'),
      )
    "
  >
    <Icon name="alert-square" size="md" /> Lost Connection, retrying...
  </div>
</template>

<script setup lang="ts">
import { clsx } from "clsx";
import { computed } from "vue";
import { RouteLocationAsPathGeneric, RouteLocationAsRelativeGeneric, useRoute, useRouter } from "vue-router";
import { themeClasses, Icon, NewButton } from "@si/vue-lib/design-system";
import NavbarButton from "@/components/layout/navbar/NavbarButton.vue";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";

const route = useRoute();
const router = useRouter();
const ffStore = useFeatureFlagsStore();

const isConnected = computed(() => {
  if (ffStore.SHOW_WS_DISCONNECT) return props.connected;
  return true;
});

interface NavProps {
  workspaceId: string;
  changeSetId: string;
  componentId?: string;
  onHead: boolean;
  viewId?: string;
  connected: boolean;
}

const props = defineProps<NavProps>();

const compositionLink = computed(() => {
  const name = props.componentId ? "new-hotness-component" : props.viewId ? "new-hotness-view" : "new-hotness";
  const params: Partial<NavProps> = { ...props };
  delete params.connected;
  return {
    name,
    params,
  } as RouteLocationAsRelativeGeneric | RouteLocationAsPathGeneric;
});

const onSim = computed(
  () =>
    route.name?.toString().startsWith("new-hotness") &&
    route.name?.toString() !== "new-hotness-review" &&
    !route.name?.toString().startsWith("new-hotness-policy"),
);
const onAuthor = computed(() => route.matched.some((r) => r.name === "workspace-lab"));
const onReview = computed(() => route.matched.some((r) => r.name === "new-hotness-review"));
const onAudit = computed(() => route.matched.some((r) => r.name === "workspace-audit"));
const onPolicy = computed(() => route.name?.toString().startsWith("new-hotness-policy"));
</script>
