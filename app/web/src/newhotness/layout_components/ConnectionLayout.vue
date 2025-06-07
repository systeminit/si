<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <h3 class="m-xs">{{ props.label }}</h3>
  <ul class="flex flex-col gap-xs mx-xs">
    <li
      v-for="conn in props.connections"
      :key="`${conn.key}`"
      class="p-xs border-neutral-600 border"
    >
      <p
        class="text-neutral-400 text-sm cursor-pointer"
        @click="() => navigate(conn.componentId)"
      >
        {{ conn.component.schemaName }}
        {{ conn.component.name }}
      </p>
      <!-- negative margin pulls things together -->
      <p class="text-white text-lg mb-[-6px] mt-[-4px]">{{ conn.self }}</p>
      <p class="text-neutral-400 text-sm">{{ conn.other }}</p>
    </li>
  </ul>
</template>

<script setup lang="ts">
import { useRoute, useRouter } from "vue-router";
import { BifrostComponentInList } from "@/workers/types/entity_kind_types";

export interface SimpleConnection {
  key: string;
  componentId: string;
  component: BifrostComponentInList;
  self: string;
  other: string;
}

const props = defineProps<{
  label: string;
  connections: SimpleConnection[];
}>();

const router = useRouter();
const route = useRoute();
const navigate = (componentId: string) => {
  const params = { ...route.params };
  params.componentId = componentId;
  router.push({
    name: "new-hotness-component",
    params,
  });
};
</script>
