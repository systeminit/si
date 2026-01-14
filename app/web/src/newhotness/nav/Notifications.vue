<template>
  <button
    v-if="numberICanApprove > 0 || currentChangeSetNeedsApproval"
    v-tooltip="{
      content: tooltipText,
      theme: 'notifications',
    }"
    :class="
      clsx(
        'relative h-full flex flex-row gap-2xs items-center children:pointer-events-none font-bold',
        numberICanApprove > 0 || currentChangeSetNeedsApproval ? 'bg-destructive-900' : 'hover:bg-black',
        (numberICanApprove > 0 || currentChangeSetNeedsApproval) && !compact ? 'p-xs' : 'p-sm',
      )
    "
    @click="handleNotificationClick"
  >
    <Icon
      name="bell"
      :class="clsx(numberICanApprove > 0 || currentChangeSetNeedsApproval ? 'text-destructive-500' : 'text-shade-0')"
    />
    <template v-if="(numberICanApprove > 0 || currentChangeSetNeedsApproval) && !compact">
      <PillCounter
        :count="Math.max(numberICanApprove, currentChangeSetNeedsApproval ? 1 : 0)"
        noColorStyles
        hideIfZero
        class="bg-destructive-500 py-2xs"
      />
      <div class="text-xs">
        Approval{{ numberICanApprove > 1 || (numberICanApprove === 0 && currentChangeSetNeedsApproval) ? "s" : "" }}
      </div>
    </template>
    <ApprovalPendingModal
      v-if="numberICanApprove > 0 || currentChangeSetNeedsApproval"
      ref="pendingApprovalModalRef"
      :changeSetsNeedingApproval="changeSetsNeedingApproval"
    />
  </button>
</template>

<script setup lang="ts">
import clsx from "clsx";
import { computed, ref, onMounted, onBeforeUnmount } from "vue";
import { Icon, PillCounter } from "@si/vue-lib/design-system";
import { useQueries } from "@tanstack/vue-query";
import { ChangeSetId, ChangeSet, ChangeSetStatus } from "@/api/sdf/dal/change_set";
import { ApprovalData } from "../types";
import ApprovalPendingModal from "./ApprovalPendingModal.vue";
import { useContext } from "../logic_composables/context";
import { approverForChangeSet } from "../logic_composables/change_set";
import { useApi, routes, apiContextForChangeSet } from "../api_composables";

const props = defineProps<{
  changeSetsNeedingApproval: ChangeSet[];
}>();

const pendingApprovalModalRef = ref<InstanceType<typeof ApprovalPendingModal> | null>(null);

const ctx = useContext();

const queries = computed(() =>
  props.changeSetsNeedingApproval.map((changeSet) => {
    const changeSetId = changeSet.id;
    return {
      // TODO(nick): use the approvals enabled feature flag
      // NOTE(nick): this needs a different query key than the individual query because they
      // return different payloads. Without this, users of both queries will get colliding data. It
      // may be possible to unite them with the same query key to avoid calling the same route
      // twice.
      queryKey: ["approvalstatusbychangesetid", changeSetId],
      queryFn: async () => {
        const apiCtx = apiContextForChangeSet(ctx, changeSetId);
        const api = useApi(apiCtx);

        const call = api.endpoint<ApprovalData>(routes.ChangeSetApprovalStatus);
        const response = await call.get();
        if (api.ok(response)) {
          return { changeSetId, approvalData: response.data };
        }
        return undefined;
      },
    };
  }),
);
const allApprovalDataQueries = useQueries({
  queries,
});
const allApprovalData = computed(() => {
  const results: Record<ChangeSetId, ApprovalData> = {};
  for (const approvalDataQuery of allApprovalDataQueries.value) {
    if (approvalDataQuery.data) results[approvalDataQuery.data.changeSetId] = approvalDataQuery.data.approvalData;
  }
  return results;
});

const numberICanApprove = computed(() => {
  let approvable = 0;
  props.changeSetsNeedingApproval.forEach((changeSet) => {
    const approvalData = allApprovalData.value[changeSet.id];
    if (!approvalData || !ctx.user) return;
    if (approverForChangeSet(ctx.user.pk, approvalData)) approvable++;
  });
  return approvable;
});

const currentChangeSetNeedsApproval = computed(() => {
  return ctx.changeSet.value?.status === ChangeSetStatus.NeedsApproval;
});

const tooltipText = computed(() => {
  if (numberICanApprove.value === 1) {
    return "You have a Change Set to approve.";
  } else if (numberICanApprove.value > 1) {
    return `You have ${numberICanApprove.value} Change Sets to approve.`;
  } else if (currentChangeSetNeedsApproval.value) {
    return "This change set requires approval before it can be applied.";
  } else {
    return "No Notifications";
  }
});

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

const compact = computed(() => windowWidth.value < 850);

const openPendingApprovalsModal = () => {
  if (numberICanApprove.value > 0 || currentChangeSetNeedsApproval.value) {
    pendingApprovalModalRef.value?.open();
  }
};

const handleNotificationClick = () => {
  if (numberICanApprove.value > 0 || currentChangeSetNeedsApproval.value) {
    // Open the approvals modal if you have change sets to approve OR if current change set needs approval
    openPendingApprovalsModal();
  }
};
</script>
