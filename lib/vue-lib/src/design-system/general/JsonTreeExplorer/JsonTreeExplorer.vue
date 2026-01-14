<template>
  <div class="json-tree-explorer">
    <VButton
      size="xs"
      icon="expand-row"
      class="m-xs"
      @click="toggleAllOpen(true)"
    />
    <VButton size="xs" icon="collapse-row" @click="toggleAllOpen(false)" />
    <JsonTreeExplorerItem
      v-for="(value, prop) in object"
      :key="prop"
      :prop="prop"
      :value="value"
    />

    <DropdownMenu ref="contextMenuRef">
      <DropdownMenuItem
        v-if="contextMenuType === 'object' || contextMenuType === 'array'"
        icon="clipboard-copy"
        label="Copy raw JSON"
        @select="copyValueHandler"
      />
      <DropdownMenuItem
        v-else
        icon="clipboard-copy"
        label="Copy value"
        @select="copyValueHandler"
      />

      <DropdownMenuItem
        icon="dots-horizontal"
        label="Copy JSON path"
        @select="copyPathHandler"
      />
    </DropdownMenu>
  </div>
</template>

<script lang="ts">
type JsonTreeNodeType =
  | "link"
  | "string"
  | "number"
  | "array"
  | "object"
  | "boolean"
  | "empty"
  | "unknown";

type EventBusEvents = { toggleAllOpen: boolean };

type JsonTreeExplorerRootNodeContext = {
  openContextMenu(e: MouseEvent, path: string, type: JsonTreeNodeType): void;
  eventBus: Emitter<EventBusEvents>;
};

export const JsonTreeExplorerRootNodeContextInjectionKey: InjectionKey<JsonTreeExplorerRootNodeContext> =
  Symbol("JsonTreeExplorerRootNodeContext");

export function useJsonTreeRootContext() {
  const ctx = inject(JsonTreeExplorerRootNodeContextInjectionKey, null);
  if (!ctx)
    throw new Error(
      "<JsonTreeExplorerItem> should only be used within a <JsonTreeExplorer>",
    );
  return ctx;
}
</script>

<!-- eslint-disable vue/block-order,import/first -->
<script setup lang="ts">
import * as _ from "lodash-es";
import { InjectionKey, inject, provide, ref } from "vue";
import mitt, { Emitter } from "mitt";

import JsonTreeExplorerItem from "./JsonTreeExplorerItem.vue";
import { DropdownMenu, DropdownMenuItem, VButton } from "../..";

// import { copyToClipboard } from "@/utils/copy-paste";

const props = defineProps({
  object: Object,
  numPreviewProps: { type: Number, default: 3 },
});

const contextMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const contextMenuPath = ref<string>();
const contextMenuType = ref<"object">();

function openContextMenu(e: MouseEvent, path: string, type: "object") {
  contextMenuRef.value?.open(e, true);
  contextMenuPath.value = path;
  contextMenuType.value = type;
}

function toggleAllOpen(open: boolean) {
  eventBus.emit("toggleAllOpen", open);
  // this.$emit("toggleAllOpen", open);
}
function copyValueHandler() {
  if (!contextMenuPath.value) return;
  const val = _.get(props.object, contextMenuPath.value);
  if (_.isObject(val) || _.isArray(val)) {
    // eslint-disable-next-line @typescript-eslint/no-floating-promises
    navigator.clipboard.writeText(JSON.stringify(val, null, 2));
  } else {
    // eslint-disable-next-line @typescript-eslint/no-floating-promises
    navigator.clipboard.writeText(val.toString() || "");
  }
}
function copyPathHandler() {
  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  navigator.clipboard.writeText(contextMenuPath.value || "");
}

const eventBus = mitt<EventBusEvents>();

// EXPOSED TO CHILDREN
provide(JsonTreeExplorerRootNodeContextInjectionKey, {
  openContextMenu,
  eventBus,
});
</script>

<style lang="less">
.json-tree-explorer {
  margin-top: 10px;
  width: 100%;
  padding: 10px;
  background: #fff;
  color: #222;
}
</style>
