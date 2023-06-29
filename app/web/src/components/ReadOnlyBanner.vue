<template>
  <div
    class="dark:border-neutral-600 text-sm leading-tight text-neutral-500 flex flex-row items-center pr-xs"
  >
    <div class="flex flex-col grow items-center">
      <div class="text-center font-bold text-xl pt-xs">Read Only</div>
      <div class="text-left flex flex-row items-center gap-4 px-xs pb-xs">
        <!-- <a
        href="#"
        class="hover:text-neutral-600 dark:hover:text-neutral-400"
        @click="hideReadOnly"
      >
        <Icon name="x-circle" />
      </a> -->

        <div class="text-sm italic line-clamp-3">
          You cannot make changes here. To make changes, go to the
          <RouterLink
            :to="{
              name: 'workspace-compose',
              params: { changeSetId: 'auto' },
            }"
            class="text-action-500 hover:underline"
            >model view</RouterLink
          >
          and select a change set.
        </div>
      </div>
    </div>
    <VButton
      v-if="showRefreshAllButton"
      icon="refresh"
      variant="ghost"
      loadingIcon="refresh-active"
      loadingText="Refreshing..."
      :loading="refreshing"
      @click="onClickRefreshButton"
      >Resources</VButton
    >
  </div>
</template>

<script lang="ts" setup>
import { VButton } from "@si/vue-lib/design-system";
import { ref, onBeforeUnmount } from "vue";
import { RouterLink } from "vue-router";
import { useComponentsStore } from "@/store/components.store";

const componentsStore = useComponentsStore();

const refreshing = ref(false);

defineProps({
  showRefreshAllButton: { type: Boolean, default: false },
});

let timeout: Timeout;

const onClickRefreshButton = () => {
  refreshing.value = true;
  componentsStore.REFRESH_ALL_RESOURCE_INFO();
  timeout = setTimeout(() => {
    refreshing.value = false;
  }, 3000);
};

onBeforeUnmount(() => {
  clearTimeout(timeout);
});
</script>
