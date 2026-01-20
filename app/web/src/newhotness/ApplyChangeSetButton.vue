<template>
  <section v-if="!ctx.onHead.value">
    <NewButton
      ref="applyButtonRef"
      tone="action"
      :label="squish ? 'Apply' : 'Apply Change Set'"
      :pill="proposedActions.length"
      class="ml-2xs mr-xs"
      :loadingText="squish ? 'Applying' : 'Applying Changes'"
      :loading="applyInFlight"
      :disabled="buttonDisabled"
      :icon="buttonDisabled ? 'loader' : undefined"
      :tooltip="buttonTooltipText"
      tooltipPlacement="bottom-end"
      tooltipTheme="apply-button"
      @click="openApplyChangeSetModal"
    />
    <ApplyChangeSetModal ref="applyChangeSetModalRef" votingKind="merge" :actions="proposedActions" />
  </section>
</template>

<script lang="ts" setup>
import { computed, ref, watchEffect, nextTick } from "vue";
import * as _ from "lodash-es";
import { NewButton } from "@si/vue-lib/design-system";
import { useQuery } from "@tanstack/vue-query";
import { useRoute, useRouter } from "vue-router";
import { BifrostActionViewList, EntityKind } from "@/workers/types/entity_kind_types";
import { bifrost, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import ApplyChangeSetModal from "./ApplyChangeSetModal.vue";
import { useApplyChangeSet } from "./logic_composables/change_set";
import { useContext } from "./logic_composables/context";
import { useStatus } from "./logic_composables/status";

const props = defineProps<{
  squish?: boolean;
}>();

const ctx = useContext();

const route = useRoute();
const router = useRouter();

const applyChangeSetModalRef = ref<InstanceType<typeof ApplyChangeSetModal>>();

const openApplyChangeSetModal = () => {
  applyChangeSetModalRef.value?.open();
};

// Watch for query parameter to auto-open modal after navigation
watchEffect(() => {
  if (route.query.openApplyModal === "true") {
    // Clean up the query parameter
    router.replace({
      ...route,
      query: { ...route.query, openApplyModal: undefined },
    });

    // Open the modal after navigation and context update are complete
    setTimeout(() => {
      nextTick(() => {
        openApplyChangeSetModal();
      });
    }, 100); // Small delay to ensure context is updated
  }
});

const { applyInFlight } = useApplyChangeSet(ctx);

const key = useMakeKey();
const args = useMakeArgs();

const actionsRaw = useQuery<BifrostActionViewList | null>({
  queryKey: key(EntityKind.ActionViewList),
  queryFn: async () => await bifrost<BifrostActionViewList>(args(EntityKind.ActionViewList)),
  enabled: ctx.queriesEnabled,
});
const actions = computed(() => actionsRaw.data.value?.actions ?? []);
const proposedActions = computed(() =>
  actions.value.filter((action) => action.originatingChangeSetId === ctx.changeSetId.value),
);

const status = useStatus();
const buttonDisabled = computed(() => {
  // Need a change set to apply...
  if (!ctx.changeSet.value) return true;

  // If we are on HEAD, we cannot apply.
  if (ctx.onHead.value) return true;

  // If the change set is churning on work on flight, do not allow the ability to apply.
  if (status[ctx.changeSet.value.id] === "syncing") return true;

  return !ctx.queriesEnabled.value;
});
const buttonTooltipText = computed(() => buttonDisabled.value ? "We are updating this change set based on recent changes. Applying will be available once the updates are finished." : undefined);
</script>
