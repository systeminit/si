<template>
  <div>
    <DropdownMenu
      ref="contextMenuRef"
      :items="rightClickMenuItems"
      variant="editor"
      :anchorTo="anchor"
      alignOutsideRightEdge
    />
    <EraseModal ref="eraseModalRef" @confirm="componentsFinishErase" />
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import {
  DropdownMenu,
  DropdownMenuItemObjectDef,
} from "@si/vue-lib/design-system";
import { computed, nextTick, ref } from "vue";
import { ComponentId } from "@/api/sdf/dal/component";
import EraseModal from "./EraseModal.vue";
// import { useApi } from "./api_composables";

const contextMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const rightClickMenuItems = computed(() => {
  const items: DropdownMenuItemObjectDef[] = [];

  // can erase so long as you have not selected a view
  items.push({
    label: "Erase",
    shortcut: "⌘E",
    icon: "erase",
    onSelect: () => {},
  });

  return items;
});

// const eraseApi = useApi();
const eraseComponentIds = ref<ComponentId[] | undefined>(undefined);
const eraseModalRef = ref<InstanceType<typeof EraseModal>>();

const componentsStartErase = (componentIds: ComponentId[]) => {
  eraseComponentIds.value = componentIds;
  eraseModalRef.value?.open();
  close();
};
const componentsFinishErase = async () => {
  if (!eraseComponentIds.value || eraseComponentIds.value.length === 0) return;

  // TODO(WENDY) - finish this when we have the endpoint ready
  // const callApi = eraseApi.endpoint(
  //   routes.DeleteComponents,
  // );

  // const { req } = await callApi.delete({ componentIds: [eraseComponentId.value], forceErase: true });

  // if (eraseApi.ok(req)) {
  //   eraseModalRef.value?.close();
  // }
};

// eslint-disable-next-line @typescript-eslint/ban-types
const anchor = ref<Object | undefined>(undefined);

function open(
  // eslint-disable-next-line @typescript-eslint/ban-types
  anchorTo: Object,
  componentIds: ComponentId[],
) {
  anchor.value = anchorTo;
  // TODO(WENDY) - use the componentIds here!
  // eslint-disable-next-line no-console
  console.log(componentIds);
  nextTick(() => contextMenuRef.value?.open());
}

function close() {
  contextMenuRef.value?.close();
}

const isOpen = computed(() => contextMenuRef.value?.isOpen);

defineExpose({ open, close, isOpen, componentsStartErase });
</script>
