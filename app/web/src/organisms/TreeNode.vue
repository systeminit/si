<template>
  <li
    class="border-b-2 dark:border-neutral-600 cursor-pointer"
    @click.stop="emit('select', component.id)"
  >
    <template v-if="component.children">
      <Disclosure v-if="component.matchesFilter" v-slot="{ open }" default-open>
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
          <div class="flex flex-row items-center">
            <Icon
              v-if="statusIcons.change"
              :name="statusIcons.change.icon"
              :tone="statusIcons.change.tone"
            />
            <div v-else class="w-6 h-6" />
            <Icon
              v-if="statusIcons.qualification"
              :name="statusIcons.qualification.icon"
              :tone="statusIcons.qualification.tone"
            />
            <div v-else class="w-6 h-6" />
            <Icon
              v-if="statusIcons.confirmation"
              :name="statusIcons.confirmation.icon"
              :tone="statusIcons.confirmation.tone"
            />
            <div v-else class="w-6 h-6" />
          </div>
        </div>
        <DisclosurePanel>
          <Tree
            :tree-data="component.children"
            class="pl-8"
            @select="(componentId) => emit('select', componentId)"
          />
        </DisclosurePanel>
      </Disclosure>
      <template v-else>
        <Tree
          :tree-data="component.children"
          class="pl-8"
          @select="(componentId) => emit('select', componentId)"
        />
      </template>
    </template>
    <div v-else-if="component.matchesFilter" class="flex flex-row items-center">
      <span
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
      <div class="flex flex-row items-center">
        <Icon
          v-if="statusIcons.change"
          :name="statusIcons.change.icon"
          :tone="statusIcons.change.tone"
        />
        <div v-else class="w-6 h-6" />
        <Icon
          v-if="statusIcons.qualification"
          :name="statusIcons.qualification.icon"
          :tone="statusIcons.qualification.tone"
        />
        <div v-else class="w-6 h-6" />
        <Icon
          v-if="statusIcons.confirmation"
          :name="statusIcons.confirmation.icon"
          :tone="statusIcons.confirmation.tone"
        />
        <div v-else class="w-6 h-6" />
      </div>
    </div>
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

const statusIcons = computed(() => props.component.statusIcons ?? {});
</script>
