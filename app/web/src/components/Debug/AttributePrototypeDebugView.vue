<template>
  <div>
    <TreeNode
      v-for="[funcArgName, funcArgViews] in Object.entries(data)"
      :key="funcArgName"
      :defaultOpen="false"
      :label="funcArgName"
      alwaysShowArrow
      enableGroupToggle
      indentationSize="xs"
      leftBorderSize="none"
    >
      <TreeNode
        v-for="funcView in funcArgViews"
        :key="funcView.valueSourceId"
        :label="funcView.valueSource"
        alwaysShowArrow
        enableGroupToggle
        indentationSize="sm"
        leftBorderSize="none"
      >
        <DebugViewItem title="Argument Name" :data="funcView.name" />
        <DebugViewItem title="Value Source" :data="funcView.valueSource" />
        <DebugViewItem title="Value Source Id" :data="funcView.valueSourceId" />
        <DebugViewItem title="Value" :data="funcView.value ?? 'NULL/None'" />
        <DebugViewItem
          title="Connection Kind"
          :data="funcView.socketSourceKind ?? 'NULL'"
        />
        <DebugViewItem
          title="Path To Attribute Value"
          :data="funcView.path ?? 'NULL'"
        />
        <DebugViewItem title="Data Is Used" :data="funcView.isUsed" />
      </TreeNode>
    </TreeNode>
  </div>
</template>

<script setup lang="ts">
import { TreeNode } from "@si/vue-lib/design-system";
import { FuncArgDebugView } from "@/store/components.store";
import DebugViewItem from "./DebugViewItem.vue";

defineProps<{ data: { [key: string]: FuncArgDebugView[] } }>();
</script>
