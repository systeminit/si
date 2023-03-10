<template>
  <div
    ref="readOnlyRef"
    class="border-b-2 dark:border-neutral-600 text-sm leading-tight text-neutral-500 flex flex-col"
  >
    <div class="text-center font-bold text-xl pt-xs">Read Only</div>
    <div class="flex flex-row items-center gap-4 px-xs pb-xs">
      <!-- <a
        href="#"
        class="hover:text-neutral-600 dark:hover:text-neutral-400"
        @click="hideReadOnly"
      >
        <Icon name="x-circle" />
      </a> -->

      <div class="text-sm italic">
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
</template>

<script lang="ts" setup>
import { ref } from "vue";
import Icon from "@/ui-lib/icons/Icon.vue";

const readOnlyRef = ref();

const hideReadOnly = () => {
  readOnlyRef.value.classList.add("hidden");
};
</script>
