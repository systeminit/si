<template>
  <section v-if="!ctx.onHead.value">
    <VButton
      ref="applyButtonRef"
      size="sm"
      tone="action"
      label="Apply Change Set"
      class="ml-2xs mr-xs"
      loadingText="Applying Changes"
      :loading="applyChangeSet.loading.value"
      :disabled="disableApplyChangeSet"
      @click="openApprovalFlowModal"
    >
      <template #iconRight>
        <!--
        NOTE(nick): I wanted this to look like "Add a component", but its concept of a pill
        more of a plaintext helper. The pills look a little different and I don't love it, but
        this will have to do in the meantime.
        -->
        <PillCounter
          :count="proposedActions.length"
          :paddingX="proposedActions.length > 10 ? '2xs' : 'xs'"
          noColorStyles
          class="border border-action-200 ml-2xs py-2xs"
        />
      </template>
    </VButton>
    <ApprovalFlowModal
      ref="approvalFlowModalRef"
      votingKind="merge"
      :actions="proposedActions"
    />
  </section>
</template>

<script lang="ts" setup>
import { computed, inject, ref } from "vue";
import * as _ from "lodash-es";
import { VButton, PillCounter } from "@si/vue-lib/design-system";
import { useQuery } from "@tanstack/vue-query";
import {
  BifrostActionViewList,
  EntityKind,
} from "@/workers/types/entity_kind_types";
import { bifrost, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import ApprovalFlowModal from "./ApprovalFlowModal.vue";
import { assertIsDefined, Context } from "./types";
import { useApplyChangeSet } from "./logic_composables/change_set";

const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);

const approvalFlowModalRef = ref<InstanceType<typeof ApprovalFlowModal>>();

const openApprovalFlowModal = () => {
  approvalFlowModalRef.value?.open();
};

const { applyChangeSet, disableApplyChangeSet } = useApplyChangeSet();

const key = useMakeKey();
const args = useMakeArgs();

const actionsRaw = useQuery<BifrostActionViewList | null>({
  queryKey: key(EntityKind.ActionViewList),
  queryFn: async () =>
    await bifrost<BifrostActionViewList>(args(EntityKind.ActionViewList)),
  enabled: ctx.queriesEnabled,
});
const actions = computed(() => actionsRaw.data.value?.actions ?? []);
const proposedActions = computed(() =>
  actions.value.filter(
    (action) => action.originatingChangeSetId === ctx.changeSetId.value,
  ),
);
</script>
