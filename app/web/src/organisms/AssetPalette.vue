<template>
  <template v-if="schemasReqStatus.isPending || addMenuReqStatus.isPending">
    loading...
  </template>
  <template v-else-if="schemasReqStatus.isSuccess">
    <!-- <SiSearch /> -->

    <p
      class="border-b-2 dark:border-neutral-600 text-sm leading-tight p-2.5 text-neutral-500"
    >
      Drag the assets that you wish to include in your application into the
      canvas to the right.
    </p>

    <ul class="overflow-y-auto">
      <SiCollapsible
        v-for="(category, categoryIndex) in addMenuData"
        :key="categoryIndex"
        :label="category.displayName"
        as="li"
        content-as="ul"
        default-open
      >
        <li
          v-for="(schema, schemaIndex) in category.schemas"
          :key="schemaIndex"
          class="select-none"
        >
          <SiNodeSprite
            :class="selectedSchemaId === schema.id ? 'bg-action-500' : ''"
            :color="schema.color"
            :name="schema.displayName"
            class="border-b-2 dark:border-neutral-600 hover:bg-action-500 dark:text-white hover:text-white hover:cursor-pointer"
            @mousedown="onSelect(schema.id)"
          />
        </li>
      </SiCollapsible>
    </ul>
  </template>
</template>

<script lang="ts" setup>
import _ from "lodash";
import { computed, ref } from "vue";
import SiNodeSprite from "@/molecules/SiNodeSprite.vue";
import SiCollapsible from "@/organisms/SiCollapsible.vue";

import { useComponentsStore } from "@/store/components.store";

export type SelectAssetEvent = {
  schemaId: number;
};

const emit = defineEmits<{
  (e: "select", selectAssetEvent: SelectAssetEvent): void;
}>();

const componentsStore = useComponentsStore();
// NOTE - component store is automatically fetching things we need when it is used
// otherwise we could trigger calls here

// TODO - probably should not need 2 requests here. currently we only use schema variants for the colors...
const schemasReqStatus = componentsStore.getRequestStatus(
  "FETCH_AVAILABLE_SCHEMAS",
);
const addMenuReqStatus = componentsStore.getRequestStatus(
  "FETCH_NODE_ADD_MENU",
);

const addMenuData = computed(() => componentsStore.nodeAddMenu);
const selectedSchemaId = ref<number>();

function onSelect(schemaId: number) {
  selectedSchemaId.value = schemaId;
  emit("select", { schemaId });
}
</script>
