<template>
  <ul>
    <TreeNode
      v-for="(node, index) in treeData"
      :key="index"
      :component="node"
      @select="(componentId) => emit('select', componentId)"
      @multiselect="(componentId) => emit('multiselect', componentId)"
      @pan="(componentId) => emit('pan', componentId)"
    />
  </ul>
</template>

<script lang="ts" setup>
import { ComponentTreeNode } from "@/store/components.store";
import TreeNode from "@/organisms/TreeNode.vue";

const props = defineProps<{ treeData: ComponentTreeNode[] }>();

const emit = defineEmits<{
  (e: "select", componentId: string): void;
  (e: "multiselect", componentId: string): void;
  (e: "pan", componentId: string): void;
}>();
</script>
