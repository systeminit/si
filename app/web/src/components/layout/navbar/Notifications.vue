<template>
  <button
    v-if="numberICanApprove > 0"
    v-tooltip="{
      content: tooltipText,
      theme: 'notifications',
    }"
    :class="
      clsx(
        'relative h-full flex flex-row gap-2xs items-center children:pointer-events-none font-bold',
        numberICanApprove > 0 ? 'bg-destructive-900' : 'hover:bg-black',
        numberICanApprove > 0 && !compact ? 'p-xs' : 'p-sm',
      )
    "
    @click="openPendingApprovalsModal"
  >
    <Icon name="bell" :class="clsx(numberICanApprove > 0 ? 'text-destructive-500' : 'text-shade-0')" />
    <template v-if="numberICanApprove > 0 && !compact">
      <PillCounter :count="numberICanApprove" noColorStyles hideIfZero class="bg-destructive-500 py-2xs" />
      <div class="text-xs">Approval{{ numberICanApprove > 1 ? "s" : "" }}</div>
    </template>
    <ApprovalPendingModal v-if="numberICanApprove > 0" ref="pendingApprovalModalRef" />
  </button>
</template>

<script setup lang="ts">
import clsx from "clsx";
import { computed, ref, onMounted, onBeforeUnmount } from "vue";
import { Icon, PillCounter } from "@si/vue-lib/design-system";
import { approverForChangeSet, useChangeSetsStore } from "@/store/change_sets.store";
import { useAuthStore } from "@/store/auth.store";
import ApprovalPendingModal from "../../ApprovalPendingModal.vue";

const changeSetsStore = useChangeSetsStore();
const authStore = useAuthStore();

const pendingApprovalModalRef = ref<InstanceType<typeof ApprovalPendingModal> | null>(null);

const numberICanApprove = computed(() => {
  let approvable = 0;
  changeSetsStore.changeSetsNeedingApproval.forEach((changeSet) => {
    const approvalData = changeSetsStore.changeSetsApprovalData[changeSet.id];
    if (!approvalData || !authStore.user) return;
    if (approverForChangeSet(authStore.user.pk, approvalData)) approvable++;
  });
  return approvable;
});

const tooltipText = computed(() => {
  if (numberICanApprove.value === 1) {
    return "You have a Change Set to approve.";
  } else if (numberICanApprove.value > 1) {
    return `You have ${numberICanApprove.value} Change Sets to approve.`;
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
  if (numberICanApprove.value > 0) {
    pendingApprovalModalRef.value?.open();
  }
};
</script>
