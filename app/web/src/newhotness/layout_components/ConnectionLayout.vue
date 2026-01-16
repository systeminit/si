<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <h3 class="m-xs text-sm">{{ props.label }}</h3>
  <ul class="flex flex-col gap-xs mx-xs">
    <li
      v-for="conn in props.connections"
      :key="`${conn.key}`"
      :class="
        clsx(
          'py-xs pr-xs border rounded-sm',
          themeClasses('border-neutral-300', 'border-neutral-600'),
          props.label === 'Incoming' &&
            highlightedPath === conn.self &&
            themeClasses('border-neutral-500 bg-neutral-300', 'border-neutral-400 bg-neutral-700'),
        )
      "
    >
      <div class="flex flex-row gap-2xs">
        <div class="flex-none flex items-center justify-center pl-2xs mt-[1px]">
          <Icon
            name="incoming-connection"
            size="none"
            :class="clsx('w-8 h-11', themeClasses('text-neutral-500', 'text-neutral-400'))"
          />
        </div>

        <div class="flex-1 min-w-0 flex flex-col gap-3xs pl-0 font-mono text-xs">
          <div
            :class="
              clsx(
                'flex flex-row gap-xs -ml-xs p-2xs rounded-sm',
                props.label === 'Incoming' &&
                  ctx.componentDetails.value[conn.componentId]?.name &&
                  clsx('cursor-pointer', themeClasses('hover:bg-neutral-200', 'hover:bg-neutral-700')),
              )
            "
            @click="
              props.label === 'Incoming' && ctx.componentDetails.value[conn.componentId]?.name
                ? navigate(conn.componentId)
                : undefined
            "
          >
            <template v-if="props.label === 'Incoming'">
              <TruncateWithTooltip
                :class="
                  clsx(
                    'flex-1 min-w-0 basis-1/3 max-w-fit px-2xs py-2xs rounded-[2px] border',
                    themeClasses(
                      'bg-white border-neutral-300 text-[#207E65]',
                      'bg-neutral-900 border-neutral-600 text-[#AAFEC7]',
                    ),
                  )
                "
              >
                {{ ctx.componentDetails.value[conn.componentId]?.schemaVariantName }}
              </TruncateWithTooltip>
              <TruncateWithTooltip
                :class="
                  clsx(
                    'flex-1 min-w-0 basis-1/3 max-w-fit px-2xs py-2xs rounded-[2px] border',
                    themeClasses(
                      'bg-white border-neutral-300 text-[#631AC2]',
                      'bg-neutral-900 border-neutral-600 text-[#D4B4FE]',
                    ),
                  )
                "
              >
                {{ ctx.componentDetails.value[conn.componentId]?.name }}
              </TruncateWithTooltip>
              <TruncateWithTooltip
                :class="
                  clsx(
                    'flex-1 min-w-0 basis-1/3 max-w-fit px-2xs py-2xs rounded-[2px] border',
                    themeClasses(
                      'bg-white border-neutral-300 text-action-700',
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
                      'bg-white border-neutral-300 text-action-700',
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
                'flex flex-row gap-xs p-2xs rounded-sm',
                props.label === 'Outgoing' &&
                  ctx.componentDetails.value[conn.componentId]?.name &&
                  clsx('cursor-pointer', themeClasses('hover:bg-neutral-200', 'hover:bg-neutral-700')),
              )
            "
            @click="
              props.label === 'Outgoing' && ctx.componentDetails.value[conn.componentId]?.name
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
                      'bg-white border-neutral-300 text-action-700',
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
                    'flex-1 min-w-0 basis-1/3 max-w-fit px-2xs py-2xs rounded-[2px] border',
                    themeClasses(
                      'bg-white border-neutral-300 text-[#207E65]',
                      'bg-neutral-900 border-neutral-600 text-[#AAFEC7]',
                    ),
                  )
                "
              >
                {{ ctx.componentDetails.value[conn.componentId]?.schemaVariantName }}
              </TruncateWithTooltip>
              <TruncateWithTooltip
                :class="
                  clsx(
                    'flex-1 min-w-0 basis-1/3 max-w-fit px-2xs py-2xs rounded-[2px] border',
                    themeClasses(
                      'bg-white border-neutral-300 text-[#631AC2]',
                      'bg-neutral-900 border-neutral-600 text-[#D4B4FE]',
                    ),
                  )
                "
              >
                {{ ctx.componentDetails.value[conn.componentId]?.name }}
              </TruncateWithTooltip>
              <TruncateWithTooltip
                :class="
                  clsx(
                    'flex-1 min-w-0 basis-1/3 max-w-fit px-2xs py-2xs rounded-[2px] border',
                    themeClasses(
                      'bg-white border-neutral-300 text-action-700',
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
import { themeClasses, TruncateWithTooltip, Icon } from "@si/vue-lib/design-system";
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
  highlightedPath?: string;
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
