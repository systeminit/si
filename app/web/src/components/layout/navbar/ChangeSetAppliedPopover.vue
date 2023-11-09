<template>
  <Popover ref="internalRef" popDown onTopOfEverything :anchorTo="anchorTo">
    <div
      class="absolute top-0 left-[50%] translate-x-[-50%] translate-y-[-100%] w-0 h-0 border-transparent border-b-white dark:border-b-neutral-700 border-[16px] border-b-[12px]"
    />
    <div
      class="bg-white dark:bg-neutral-700 rounded-lg flex flex-col items-center w-96 max-h-[90vh] shadow-md overflow-hidden pb-xs"
    >
      <div class="px-sm pt-sm pb-xs w-full">
        The change set you were in was merged.
      </div>

      <!-- TODO(Wendy) - Eventually we should display who merged it! -->
      <!-- <div v-if="applyUser" class="pr-xs">
        <UserCard :user="applyUser" hideChangesetInfo />
      </div> -->
      <div class="px-sm pb-sm pt-xs w-full">
        You are now on Head. You can continue your work by creating a new change
        set or joining another existing change set.
      </div>
      <VButton
        label="Dismiss"
        variant="ghost"
        size="sm"
        @click="internalRef.close"
      />
    </div>
  </Popover>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import { VButton } from "@si/vue-lib/design-system";
import Popover from "@/components/Popover.vue";
import { useChangeSetsStore } from "@/store/change_sets.store";
// import UserCard from "./UserCard.vue";

const changeSetsStore = useChangeSetsStore();
const internalRef = ref();

defineProps({
  anchorTo: { type: Object },
});

// TODO(Wendy) - currently this comes through as an email address, should be a UserInfo or ActorView!
// const applyUser = computed(() => {
//   return changeSetsStore.postApplyActor;
// });

const openAt = (pos: { x: number; y: number }) => {
  internalRef.value.openAt(pos);
  // This Popover automatically closes after 10 seconds
  setTimeout(close, 10000);
};

const close = () => {
  internalRef.value.close();
  changeSetsStore.postApplyActor = null;
};

const isOpen = computed(() => internalRef.value.isOpen);

defineExpose({
  openAt,
  close,
  isOpen,
});
</script>
