<template>
  <button
    v-if="userIsApprover"
    v-tooltip="{
      content: tooltipText,
      theme: 'notifications',
    }"
    :class="
      clsx(
        'relative h-full flex flex-row gap-2xs items-center children:pointer-events-none font-bold',
        pendingApprovalCount > 0 ? 'bg-destructive-900' : 'hover:bg-black',
        pendingApprovalCount > 0 && !compact ? 'p-xs' : 'p-sm',
      )
    "
    @click="openPendingApprovalsModal"
  >
    <Icon
      name="bell"
      :class="
        clsx(pendingApprovalCount > 0 ? 'text-destructive-500' : 'text-shade-0')
      "
    />
    <template v-if="pendingApprovalCount > 0 && !compact">
      <PillCounter
        :count="pendingApprovalCount"
        noColorStyles
        hideIfZero
        class="bg-destructive-500 py-2xs"
      />
      <div class="text-xs">
        Approval{{ pendingApprovalCount > 1 ? "s" : "" }}
      </div>
    </template>
    <ApprovalPendingModal
      v-if="pendingApprovalCount > 0"
      ref="pendingApprovalModalRef"
    />
  </button>
</template>

<script setup lang="ts">
import clsx from "clsx";
import { computed, ref, onMounted, onBeforeUnmount } from "vue";
import { Icon, PillCounter } from "@si/vue-lib/design-system";
import { useChangeSetsStore } from "@/store/change_sets.store";
import ApprovalPendingModal from "../../ApprovalPendingModal.vue";

const changeSetsStore = useChangeSetsStore();

const pendingApprovalModalRef = ref<InstanceType<
  typeof ApprovalPendingModal
> | null>(null);

const userIsApprover = computed(
  () => changeSetsStore.currentUserIsDefaultApprover,
);
const pendingApprovalCount = computed(
  () => changeSetsStore.changeSetsNeedingApproval.length,
);

const tooltipText = computed(() => {
  if (pendingApprovalCount.value === 1) {
    return "You have a Change Set to approve.";
  } else if (pendingApprovalCount.value > 1) {
    return `You have ${pendingApprovalCount.value} Change Sets to approve.`;
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
  if (pendingApprovalCount.value > 0) {
    pendingApprovalModalRef.value?.open();
  }
};
</script>
