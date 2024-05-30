<template>
  <VButton
    v-if="!changeSetsStore.headSelected"
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
          clsx(
            'text-xl font-bold',
            themeClasses(
              'bg-success-600 text-shade-0',
              'bg-success-300 text-success-900',
            ),
          )
        "
      />
    </template>

    <ApprovalFlowModal
      ref="approvalFlowModalRef"
      votingKind="merge"
      @completeVoting="applyChangeSet"
    />
  </VButton>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import * as _ from "lodash-es";
import { useRouter, useRoute } from "vue-router";
import {
  VButton,
  Icon,
  PillCounter,
  themeClasses,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useStatusStore } from "@/store/status.store";
import { useActionsStore } from "@/store/actions.store";
import { useAuthStore } from "@/store/auth.store";
import ApprovalFlowModal from "./ApprovalFlowModal.vue";

const statusStore = useStatusStore();
const changeSetsStore = useChangeSetsStore();
const actionsStore = useActionsStore();
const authStore = useAuthStore();
const router = useRouter();
const route = useRoute();

const displayCount = computed(() => actionsStore.proposedActions.length);

const applyChangeSetReqStatus =
  changeSetsStore.getRequestStatus("APPLY_CHANGE_SET");

const approvalFlowModalRef = ref<InstanceType<typeof ApprovalFlowModal> | null>(
  null,
);

const openApprovalFlowModal = () => {
  approvalFlowModalRef.value?.open();
};

// Applies the current change set3
const applyChangeSet = async () => {
  if (!route.name) return;
  const resp = await changeSetsStore.APPLY_CHANGE_SET(
    authStore.user?.email ?? "",
  );
  if (resp.result.success) {
    router.replace({
      name: route.name,
      params: {
        ...route.params,
        changeSetId: "head",
      },
    });
  }
};

const statusStoreUpdating = computed(() => {
  if (statusStore.globalStatus) {
    return statusStore.globalStatus.isUpdating;
  } else return false;
});
</script>
