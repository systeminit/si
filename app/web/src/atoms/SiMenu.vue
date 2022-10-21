<template>
  <VMenu
    :disabled="props.disabled ?? false"
    :placement="props.isNotRoot ? 'right-start' : 'bottom-start'"
    :triggers="props.isNotRoot ? ['hover'] : ['click']"
    :hide-triggers="['hover', 'click']"
  >
    <button
      v-if="props.isNotRoot"
      class="bg-action-900 hover:bg-action-600 p-2 flex text-base w-full text-white"
    >
      <div class="grow">{{ props.tree.name }}</div>
      <div><Icon name="chevron--right" class="self-end" /></div>
    </button>
    <slot></slot>

    <template #popper>
      <template v-if="!props.disabled">
        <template v-for="child in props.tree.children" :key="child.name">
          <SiMenu
            v-if="child.kind === 'tree'"
            :tree="child"
            is-not-root
            @selected="selected"
          />
          <div v-else>
            <button
              class="bg-action-900 hover:bg-action-600 p-2 text-base w-full text-white"
              @click="selected(child.value, $event)"
            >
              {{ child.name }}
            </button>
          </div>
        </template>
      </template>
    </template>
  </VMenu>
</template>

<script setup lang="ts">
import { defineAsyncComponent } from "vue";
import { Menu as VMenu } from "floating-vue";
import { SiMenuTree } from "@/utils/menu";
import Icon from "@/ui-lib/icons/Icon.vue";

// Eliminate the circular dependency of this recursive atom
// by using `defineAsyncComponent` in a careful way to preserve the ability for
// typeechecking to work with `tsc` and the `volar` language server used in
// VSCode/NeoVim/Vim.
//
// See:
// https://github.com/johnsoncodehk/volar/issues/644#issuecomment-1012716529
const SiMenu = defineAsyncComponent(
  // eslint-disable-next-line @typescript-eslint/no-explicit-any, import/no-self-import
  () => import("./SiMenu.vue") as any,
);

const emits = defineEmits(["selected"]);
const selected = (value: unknown, event: MouseEvent) => {
  emits("selected", value, event);
};

const props = defineProps<{
  tree: SiMenuTree;
  disabled?: boolean;
  isNotRoot?: boolean;
}>();
</script>
