<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <h3 class="m-xs">{{ props.label }}</h3>
  <ul class="flex flex-col gap-xs mx-xs">
    <li
      v-for="conn in props.connections"
      :key="`${conn.key}`"
      class="p-xs border-neutral-600 border"
    >
      <TruncateWithTooltip
        :class="
          clsx(
            'text-sm cursor-pointer hover:underline',
            themeClasses(
              'text-neutral-600 hover:text-action-500',
              'text-neutral-400 hover:text-action-300',
            ),
          )
        "
        :lineClamp="2"
        @click="() => navigate(conn.componentId)"
      >
        {{ conn.component.schemaName }}
        {{ conn.component.name }}
      </TruncateWithTooltip>
      <!-- negative margin pulls things together -->
      <p
        :class="
          clsx(
            'text-lg mb-[-6px] mt-[-4px]',
            themeClasses('text-black', 'text-white'),
          )
        "
      >
        {{ conn.self }}
      </p>
      <p
        :class="
          clsx('text-sm', themeClasses('text-neutral-600', 'text-neutral-400'))
        "
      >
        {{ conn.other }}
      </p>
    </li>
  </ul>
</template>

<script setup lang="ts">
import { useRoute, useRouter } from "vue-router";
import clsx from "clsx";
import { themeClasses, TruncateWithTooltip } from "@si/vue-lib/design-system";
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
