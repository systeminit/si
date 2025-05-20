<template>
  <ol class="ml-xs cursor-pointer" @click.stop="() => (open = !open)">
    {{
      tree.prop.name
    }}
    <template v-if="open">
      <li v-for="child in tree.children" :key="child.id" class="ml-xs">
        <PropsAsTree :tree="child" />
      </li>
    </template>
  </ol>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { Prop } from "@/workers/types/dbinterface";

const open = ref(true);

export interface PropsAsTree {
  id: string;
  children: PropsAsTree[];
  prop: Prop;
  parent?: string;
}
defineProps<{ tree: PropsAsTree }>();
</script>
