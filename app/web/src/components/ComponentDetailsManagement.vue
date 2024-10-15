<template>
  <div v-if="componentsStore.selectedComponent">
    <div class="flex flex-col h-full w-full">
      <div
        v-if="!componentsStore.selectedComponent?.resourceId"
        class="text-sm text-neutral-700 dark:text-neutral-300 p-xs italic border-b dark:border-neutral-600"
      >
        Management functions typically require the resource identifier, enter it
        here
      </div>

      <div class="ml-xs mt-xs">
        <VormInput
          v-model="resourceId"
          compact
          type="text"
          label="Resource Id"
          @blur="saveResource"
        />
      </div>
      <div
        class="text-sm text-neutral-700 dark:text-neutral-300 p-xs italic border-b dark:border-neutral-600"
      >
        The functions below will run immediately in a change set
      </div>
      <ul class="text-sm">
        <template
          v-for="prototype in funcStore.managementFunctionsForSelectedComponent"
          :key="prototype.managementPrototypeId"
        >
          <ManagementRunPrototype
            :prototype="prototype"
            :componentId="componentsStore.selectedComponent.id"
          />
        </template>
      </ul>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref, watch } from "vue";
import { VormInput } from "@si/vue-lib/design-system";
import { useFuncStore } from "@/store/func/funcs.store";
import { useComponentsStore } from "@/store/components.store";
import ManagementRunPrototype from "./ManagementRunPrototype.vue";

const funcStore = useFuncStore();
const componentsStore = useComponentsStore();

const resourceId = ref("");

watch(
  () => componentsStore.selectedComponent?.resourceId,
  () => {
    resourceId.value = componentsStore.selectedComponent?.resourceId || "";
  },
  { immediate: true },
);

const saveResource = () => {
  if (componentsStore.selectedComponent && resourceId.value)
    componentsStore.SET_RESOURCE_ID(
      componentsStore.selectedComponent?.id,
      resourceId.value,
    );
};
</script>
