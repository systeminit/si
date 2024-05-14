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
  </div>
</template>

<script lang="ts" setup>
import { RouterLink } from "vue-router";

defineProps({
  showRefreshAllButton: { type: Boolean, default: false },
});
</script>
