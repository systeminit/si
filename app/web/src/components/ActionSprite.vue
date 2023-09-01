<template>
  <Collapsible as="div" contentAs="ul" :defaultOpen="false" hideBottomBorder>
    <template #label>
      <div
        class="flex flex-row gap-2 items-center text-sm relative min-w-0 w-full justify-end"
        @click="addAction"
      >
        <div class="flex flex-col min-w-0 grow">
          <span class="font-bold truncate flex flex-row">
            <span class="grow"
              >{{ actionName }} {{ component?.schemaName ?? "" }}</span
            >
            <VButton
              v-if="props.action.id"
              class="ml-2"
              size="sm"
              tone="neutral"
              icon="x-circle"
              @click.stop="removeAction"
            />
          </span>
          <span class="text-neutral-400 truncate">
            <!-- TODO(wendy) - sometimes the component name doesn't load properly? not sure why -->
            {{ component?.displayName ?? "unknown" }}
          </span>
        </div>
      </div>
    </template>
    <template #default>
      <div :class="clsx('w-full pl-[4.25rem] pr-4')">
        <div class="py-xs text-sm">
          <div class="flex flex-col">
            <div class="font-bold">Action:</div>
            <div>{{ props.action.name }}</div>
          </div>
        </div>
      </div>
    </template>
  </Collapsible>
</template>

<script setup lang="ts">
import { computed } from "vue";
import clsx from "clsx";
import { Collapsible, VButton } from "@si/vue-lib/design-system";
import { useComponentsStore } from "@/store/components.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { Action, NewAction } from "@/api/sdf/dal/change_set";

const componentStore = useComponentsStore();
const changeSetStore = useChangeSetsStore();

const props = defineProps<{
  action: Action | NewAction;
}>();

const actionName = computed(() => {
  const name = props.action.name.trim();
  return name.length ? name.slice(0, 1).toUpperCase() + name.slice(1) : "";
});

const component = computed(
  () => componentStore.componentsById[props.action.componentId],
);

const addAction = (event: Event) => {
  if (props.action.id || !changeSetStore.selectedChangeSet) return;
  event.preventDefault();
  event.stopPropagation();
  emit("add");
};
const removeAction = () => {
  if (!props.action.id || !changeSetStore.selectedChangeSet) return;
  emit("remove");
};

const emit = defineEmits<{
  (e: "add"): void;
  (e: "remove"): void;
}>();
</script>
