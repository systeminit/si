<template>
  <VButton
    v-if="!changeSetsStore.headSelected && !disableApplyButton"
    ref="applyButtonRef"
    size="md"
    tone="success"
    loadingText="Applying Changes"
    :requestStatus="applyChangeSetReqStatus"
    :disabled="statusStoreUpdating"
    square
    @click.stop="openApprovalFlowModal"
  >
    <div class="px-xs dark:text-neutral-800 font-medium">Apply Change Set</div>

    <template #icon>
      <Icon name="tools" class="dark:text-neutral-800" />
    </template>

    <template #iconRight>
      <PillCounter
        :count="displayCount"
        :paddingX="displayCount > 10 ? '2xs' : 'xs'"
        noColorStyles
        :class="
          clsx('text-xl font-bold', themeClasses('bg-success-600 text-shade-0', 'bg-success-300 text-success-900'))
        "
      />
    </template>

    <ApprovalFlowModal ref="approvalFlowModalRef" votingKind="merge" />
  </VButton>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import * as _ from "lodash-es";
import { VButton, Icon, PillCounter, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useStatusStore } from "@/store/status.store";
import { useActionsStore } from "@/store/actions.store";
import { ChangeSetStatus } from "@/api/sdf/dal/change_set";
import ApprovalFlowModal from "./ApprovalFlowModal.vue";

const actionsStore = useActionsStore();
const changeSetsStore = useChangeSetsStore();
const statusStore = useStatusStore();

const displayCount = computed(() => actionsStore.proposedActions.length);

const applyChangeSetReqStatus = changeSetsStore.getRequestStatus("APPLY_CHANGE_SET");

const approvalFlowModalRef = ref<InstanceType<typeof ApprovalFlowModal> | null>(null);
const openApprovalFlowModal = () => {
  approvalFlowModalRef.value?.open();
};

const disableApplyButton = computed(() => changeSetsStore.selectedChangeSet?.status !== ChangeSetStatus.Open);

const statusStoreUpdating = computed(() => {
  if (statusStore.globalStatus) {
    return statusStore.globalStatus.isUpdating;
  } else return false;
});
</script>
