<template>
  <li
    class="border-b-2 dark:border-neutral-600 cursor-pointer"
    @click.stop="emit('select', component.id)"
  >
    <Disclosure
      v-if="component.children"
      v-slot="{ open }"
      :default-open="false"
    >
      <div class="flex flex-row">
        <DisclosureButton
          :class="
            clsx(
              'flex py-2 text-left font-medium focus:outline-none items-center',
              {
                sm: 'text-sm',
                md: 'text-base',
                lg: 'text-lg',
              }['sm'],
            )
          "
        >
          <Icon
            :name="open ? 'chevron--down' : 'chevron--right'"
            size="sm"
            class="dark:text-white flex-shrink-0 block"
          />
        </DisclosureButton>
        <span
          :class="
            selectedComponentId === component.id
              ? ['bg-action-500 text-white']
              : ['hover:bg-action-400 hover:text-white']
          "
          :style="{
            'border-color': component.color || colors.neutral[400],
          }"
          class="px-2 py-2 border-l-8 group flex flex-col items-baseline w-full"
        >
          <span
            class="whitespace-nowrap text-ellipsis overflow-hidden shrink leading-tight"
            >{{ component.displayName || "si-123" }}</span
          >
          <i
            :class="
              selectedComponentId === component.id
                ? ['bg-action-500 text-white']
                : ['text-neutral-500 group-hover:text-white']
            "
            class="text-sm pl-1 flex-none"
          >
            Frame
          </i>
        </span>
      </div>
      <DisclosurePanel>
        <Tree
          :tree-data="component.children"
          class="pl-8"
          @select="(componentId) => emit('select', componentId)"
        />
      </DisclosurePanel>
    </Disclosure>
    <span
      v-else
      :class="
        selectedComponentId === component.id
          ? ['bg-action-500 text-white']
          : ['hover:bg-action-400 hover:text-white']
      "
      :style="{
        'border-color': component.color || colors.neutral[400],
      }"
      class="w-full px-2 py-2 border-l-8 group flex flex-col items-baseline ml-5"
    >
      <span
        class="whitespace-nowrap text-ellipsis overflow-hidden shrink leading-tight"
        >{{ component.displayName || "si-123" }}</span
      >
      <i
        :class="
          selectedComponentId === component.id
            ? ['bg-action-500 text-white']
            : ['text-neutral-500 group-hover:text-white']
        "
        class="text-sm pl-1 flex-none"
      >
        {{ component.schemaName }}
      </i>
    </span>
  </li>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import { Disclosure, DisclosurePanel, DisclosureButton } from "@headlessui/vue";
import clsx from "clsx";
import {
  ComponentTreeNode,
  useComponentsStore,
} from "@/store/components.store";
import { colors } from "@/utils/design_token_values";
import Icon from "@/ui-lib/icons/Icon.vue";
import Tree from "@/organisms/Tree.vue";

const props = defineProps<{ component: ComponentTreeNode }>();

const emit = defineEmits<{
  (e: "select", componentId: string): void;
}>();

const componentsStore = useComponentsStore();

const selectedComponentId = computed(() => componentsStore.selectedComponentId);
</script>
