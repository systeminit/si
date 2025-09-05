<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <h3 class="m-xs text-sm">{{ props.label }}</h3>
  <ul class="flex flex-col gap-xs mx-xs">
    <li
      v-for="conn in props.connections"
      :key="`${conn.key}`"
      class="py-xs pr-xs border-neutral-600 border"
    >
      <div class="flex gap-2xs">
        <div class="flex-none flex items-center justify-center">
          <Icon name="incoming-connection" size="lg" />
        </div>

        <div class="flex-1 flex flex-col gap-2xs pl-0">
          <div
            :class="
              clsx(
                'flex gap-xs text-sm -ml-xs pl-xs',
                props.label === 'Incoming' &&
                  ctx.componentDetails.value[conn.componentId]?.name &&
                  clsx(
                    'cursor-pointer',
                    themeClasses(
                      'hover:bg-neutral-200',
                      'hover:bg-neutral-700',
                    ),
                  ),
              )
            "
            @click="
              props.label === 'Incoming' &&
              ctx.componentDetails.value[conn.componentId]?.name
                ? navigate(conn.componentId)
                : undefined
            "
          >
            <template v-if="props.label === 'Incoming'">
              <TruncateWithTooltip
                :class="
                  clsx(
                    'flex-shrink min-w-0 max-w-[120px] px-2xs py-2xs rounded-[2px] border',
                    themeClasses(
                      'bg-neutral-100 border-neutral-300 text-newhotness-greenlight',
                      'bg-neutral-900 border-neutral-600 text-newhotness-greendark',
                    ),
                  )
                "
              >
                {{
                  ctx.componentDetails.value[conn.componentId]
                    ?.schemaVariantName
                }}
              </TruncateWithTooltip>
              <TruncateWithTooltip
                :class="
                  clsx(
                    'flex-shrink min-w-0 max-w-[120px] px-2xs py-2xs rounded-[2px] border',
                    themeClasses(
                      'bg-neutral-100 border-neutral-300 text-newhotness-purplelight',
                      'bg-neutral-900 border-neutral-600 text-newhotness-purpledark',
                    ),
                  )
                "
              >
                {{ ctx.componentDetails.value[conn.componentId]?.name }}
              </TruncateWithTooltip>
              <TruncateWithTooltip
                :class="
                  clsx(
                    'flex-shrink min-w-0 max-w-[120px] px-2xs py-2xs rounded-[2px] border',
                    themeClasses(
                      'bg-neutral-100 border-neutral-300 text-action-700',
                      'bg-neutral-900 border-neutral-600 text-[#8BCDEE]',
                    ),
                  )
                "
              >
                {{ conn.other }}
              </TruncateWithTooltip>
            </template>
            <template v-else-if="props.label === 'Outgoing'">
              <TruncateWithTooltip
                :class="
                  clsx(
                    'flex-shrink min-w-0 px-2xs py-2xs rounded-[2px] border',
                    themeClasses(
                      'bg-neutral-100 border-neutral-300 text-action-700',
                      'bg-neutral-900 border-neutral-600 text-[#8BCDEE]',
                    ),
                  )
                "
              >
                {{ conn.self }}
              </TruncateWithTooltip>
            </template>
          </div>

          <div
            :class="
              clsx(
                'flex gap-xs text-sm -ml-xs pl-xs',
                props.label === 'Outgoing' &&
                  ctx.componentDetails.value[conn.componentId]?.name &&
                  clsx(
                    'cursor-pointer',
                    themeClasses(
                      'hover:bg-neutral-200',
                      'hover:bg-neutral-700',
                    ),
                  ),
              )
            "
            @click="
              props.label === 'Outgoing' &&
              ctx.componentDetails.value[conn.componentId]?.name
                ? navigate(conn.componentId)
                : undefined
            "
          >
            <template v-if="props.label === 'Incoming'">
              <TruncateWithTooltip
                :class="
                  clsx(
                    'flex-shrink min-w-0 px-2xs py-2xs rounded-[2px] border',
                    themeClasses(
                      'bg-neutral-100 border-neutral-300 text-action-700',
                      'bg-neutral-900 border-neutral-600 text-[#8BCDEE]',
                    ),
                  )
                "
              >
                {{ conn.self }}
              </TruncateWithTooltip>
            </template>
            <template v-else-if="props.label === 'Outgoing'">
              <TruncateWithTooltip
                :class="
                  clsx(
                    'flex-shrink min-w-0 max-w-[120px] px-2xs py-2xs rounded-[2px] border',
                    themeClasses(
                      'bg-neutral-100 border-neutral-300 text-newhotness-greenlight',
                      'bg-neutral-900 border-neutral-600 text-newhotness-greendark',
                    ),
                  )
                "
              >
                {{
                  ctx.componentDetails.value[conn.componentId]
                    ?.schemaVariantName
                }}
              </TruncateWithTooltip>
              <TruncateWithTooltip
                :class="
                  clsx(
                    'flex-shrink min-w-0 max-w-[120px] px-2xs py-2xs rounded-[2px] border',
                    themeClasses(
                      'bg-neutral-100 border-neutral-300 text-newhotness-purplelight',
                      'bg-neutral-900 border-neutral-600 text-newhotness-purpledark',
                    ),
                  )
                "
              >
                {{ ctx.componentDetails.value[conn.componentId]?.name }}
              </TruncateWithTooltip>
              <TruncateWithTooltip
                :class="
                  clsx(
                    'flex-shrink min-w-0 max-w-[120px] px-2xs py-2xs rounded-[2px] border',
                    themeClasses(
                      'bg-neutral-100 border-neutral-300 text-action-700',
                      'bg-neutral-900 border-neutral-600 text-[#8BCDEE]',
                    ),
                  )
                "
              >
                {{ conn.other }}
              </TruncateWithTooltip>
            </template>
          </div>
        </div>
      </div>
    </li>
  </ul>
</template>

<script setup lang="ts">
import { useRoute, useRouter } from "vue-router";
import clsx from "clsx";
import {
  themeClasses,
  TruncateWithTooltip,
  Icon,
} from "@si/vue-lib/design-system";
import { useContext } from "../logic_composables/context";

export interface SimpleConnection {
  key: string;
  componentId: string;
  self: string;
  other: string;
}

const ctx = useContext();

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
