<template>
  <li
    class="dark:border-neutral-600 cursor-pointer"
    @click.exact.stop="emit('select', component.id)"
    @click.meta.stop="emit('multiselect', component.id)"
    @click.ctrl.stop="emit('multiselect', component.id)"
    @dblclick.stop="emit('pan', component.id)"
  >
    <template v-if="component.children">
      <Disclosure v-if="component.matchesFilter" v-slot="{ open }" default-open>
        <ComponentTreeNodeHeader
          :title="component.displayName"
          subtitle="Frame"
          :status-icons="statusIcons"
          icon="hashtag"
          :is-selected="isSelected"
        >
          <DisclosureButton>
            <Icon
              :name="open ? 'chevron--down' : 'chevron--right'"
              size="sm"
              class="dark:text-white flex-shrink-0 block"
            />
          </DisclosureButton>
        </ComponentTreeNodeHeader>
        <DisclosurePanel>
          <ComponentTree
            :tree-data="component.children"
            node-class="pl-6"
            @select="(componentId) => emit('select', componentId)"
            @multiselect="(componentId) => emit('multiselect', componentId)"
            @pan="(componentId) => emit('pan', componentId)"
          />
        </DisclosurePanel>
      </Disclosure>
      <template v-else>
        <ComponentTree
          :tree-data="component.children"
          node-class="pl-6"
          @select="(componentId) => emit('select', componentId)"
          @multiselect="(componentId) => emit('multiselect', componentId)"
          @pan="(componentId) => emit('pan', componentId)"
        />
      </template>
    </template>
    <ComponentTreeNodeHeader
      v-else-if="component.matchesFilter"
      class="pl-5"
      :title="component.displayName"
      :subtitle="component.schemaName"
      :icon="component.icon"
      :status-icons="statusIcons"
      :is-selected="isSelected"
    />
  </li>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import { Disclosure, DisclosurePanel, DisclosureButton } from "@headlessui/vue";
import _ from "lodash";
import {
  ComponentTreeNode,
  useComponentsStore,
} from "@/store/components.store";
import Icon from "@/ui-lib/icons/Icon.vue";
import ComponentTree from "@/organisms/ComponentTree.vue";
import ComponentTreeNodeHeader from "@/organisms/ComponentTreeNodeHeader.vue";

const props = defineProps<{ component: ComponentTreeNode }>();

const emit = defineEmits<{
  (e: "select", componentId: string): void;
  (e: "multiselect", componentId: string): void;
  (e: "pan", componentId: string): void;
}>();

const componentsStore = useComponentsStore();

const isSelected = computed(() => {
  return _.includes(componentsStore.selectedComponentIds, props.component.id);
});

const statusIcons = computed(() => props.component.statusIcons ?? {});
</script>
