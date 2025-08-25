<template>
  <div
    v-if="auditLogs.length > 0"
    ref="scrollContainerRef"
    class="px-xs py-sm overflow-x-hidden max-h-full"
    @scrollend="handleScrollEnd"
  >
    <!--
    TODO(nick,paul): talk to Victoria about how we want these styled.
    <div class="flex-1 flex flex-row justify-between items-center gap-xs">
      <VButton
        class="grow rounded-sm"
        tone="neutral"
        variant="ghost"
        size="xs"
        :rounded="false"
        label="Expand All"
        @click="handleAll('expand')"
      />
      <VButton
        class="grow rounded-sm"
        tone="neutral"
        variant="ghost"
        size="xs"
        :rounded="false"
        label="Collapse All"
        @click="handleAll('collapse')"
      />
    </div>
    -->
    <div ref="wrapperRef" class="grid gap-xs relative">
      <!-- Single continuous timeline line behind all items -->
      <div
        v-if="auditLogs.length > 0"
        :class="
          clsx(
            'absolute left-[5px] w-[2px] z-0 top-sm translate-x-[-50%]',
            themeClasses('bg-neutral-400', 'bg-neutral-600'),
          )
        "
        :style="timelineStyle"
      />

      <div
        v-for="auditLog in auditLogs"
        ref="logRefs"
        :key="identifier(auditLog)"
        v-tooltip="
          shouldExpand(auditLog)
            ? 'Click to hide full audit log'
            : 'Click to see full audit log'
        "
        class="grid grid-cols-[10px_1fr] gap-2xs items-stretch cursor-pointer"
      >
        <div
          class="relative flex flex-row justify-center items-start h-full pt-sm self-stretch"
        >
          <div class="w-2.5 h-2.5 rounded-full bg-neutral-500 flex-shrink-0" />
        </div>
        <div
          :class="
            clsx(
              'p-xs border rounded-sm min-h-[2.5rem] min-w-0 break-words',
              themeClasses('border-neutral-400', 'border-neutral-600'),
            )
          "
          @click="toggleExpand(auditLog)"
        >
          <div class="flex flex-col">
            <div
              class="flex flex-row gap-xs items-center justify-between text-sm"
            >
              <TruncateWithTooltip class="py-2xs">
                {{ auditLog.title }}
              </TruncateWithTooltip>

              <!-- Put the timestamp and chevron at the end. -->
              <div class="flex flex-row gap-xs items-center">
                <Timestamp
                  class="text-neutral-400"
                  :date="auditLog.inner.timestamp"
                  relative="shorthand"
                  enableDetailTooltip
                  refresh
                />
                <Icon
                  class="text-neutral-400"
                  :name="
                    shouldExpand(auditLog) ? 'chevron--down' : 'chevron--left'
                  "
                />
              </div>
            </div>
            <div
              v-if="auditLog.beforeValue && auditLog.afterValue"
              class="flex flex-row gap-sm"
            >
              <TruncateWithTooltip class="line-through text-neutral-500 py-2xs">
                {{ auditLog.beforeValue }}
              </TruncateWithTooltip>
              <TruncateWithTooltip
                :class="
                  clsx(
                    'py-2xs',
                    themeClasses('text-neutral-600', 'text-neutral-300'),
                  )
                "
              >
                {{ auditLog.afterValue }}
              </TruncateWithTooltip>
            </div>
          </div>
          <Transition
            enterActiveClass="transition duration-100"
            enterFromClass="opacity-0 scale-95"
            enterToClass="opacity-100 scale-100"
            leaveActiveClass="transition duration-100"
            leaveFromClass="opacity-100 scale-100"
            leaveToClass="opacity-0 scale-95"
            @beforeEnter="startUpdateTimeline"
            @beforeLeave="startUpdateTimeline"
            @afterEnter="finishUpdateTimeline"
            @afterLeave="finishUpdateTimeline"
          >
            <div v-if="shouldExpand(auditLog)" class="mt-xs transition-all">
              <CodeViewer :code="JSON.stringify(auditLog.inner, null, 2)" />
            </div>
          </Transition>
        </div>
      </div>
    </div>

    <!-- Loading indicator and marker for when all entries are loaded. -->
    <div
      v-if="isFetchingNextPage || !hasNextPage"
      class="flex flex-row items-center justify-center mt-md text-sm text-neutral-500"
    >
      <Icon v-if="isFetchingNextPage" name="loader" size="sm" />
      <span v-if="isFetchingNextPage"> Loading More Logs... </span>
      <span v-else-if="!hasNextPage"> All Entries Loaded </span>
    </div>
  </div>
  <EmptyState
    v-else
    class="p-lg"
    icon="component"
    text="No Direct Changes"
    secondaryText="There were no direct changes made to the component."
  />
</template>

<script setup lang="ts">
import { useInfiniteQuery } from "@tanstack/vue-query";
import { computed, reactive, ref } from "vue";
import {
  TruncateWithTooltip,
  Timestamp,
  Icon,
  themeClasses,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { ComponentId } from "@/api/sdf/dal/component";
import { AuditLog, EntityKind } from "@/workers/types/entity_kind_types";
import CodeViewer from "@/components/CodeViewer.vue";
import { useMakeKey } from "@/store/realtime/heimdall";
import { routes, useApi } from "./api_composables";
import EmptyState from "./EmptyState.vue";
import { useContext } from "./logic_composables/context";

const props = defineProps<{
  componentId: ComponentId;
}>();

const componentId = computed(() => props.componentId);

const scrollContainerRef = ref<HTMLElement | null>(null);
const wrapperRef = ref<HTMLDivElement>();
const logRefs = ref<HTMLDivElement[]>();

const ctx = useContext();
const key = useMakeKey();

const pageSize = 100;
const increaseSize = 50;

// Identifies the specific audit log we are working with.
const identifier = (auditLog: ProcessedAuditLog) =>
  `${props.componentId}-${auditLog.inner.kind}-${auditLog.inner.timestamp}`;

// Keep track of which audit logs should be expanded.
const expand = reactive<Record<string, boolean>>({});
const toggleExpand = (auditLog: ProcessedAuditLog) => {
  const key = identifier(auditLog);
  if (expand[key] === undefined) {
    expand[key] = true;
  } else {
    expand[key] = !expand[key];
  }
};
const shouldExpand = (auditLog: ProcessedAuditLog): boolean => {
  return expand[identifier(auditLog)] ?? false;
};

// TODO(nick,paul): this comes back when the expand and collapse buttons come back.
// // Ability to expand and collapse everything.
// const handleAll = (option: "expand" | "collapse") => {
//   for (const auditLog of auditLogs.value) {
//     expand[identifier(auditLog)] = option === "expand";
//   }
// };

interface ProcessedAuditLog {
  inner: AuditLog;
  title: string;
  beforeValue?: string;
  afterValue?: string;
}

type AuditLogsForComponentResponse = {
  logs: AuditLog[];
  canLoadMore: boolean;
};

const auditLogsApi = useApi(ctx);
const { data, fetchNextPage, hasNextPage, isFetchingNextPage } =
  useInfiniteQuery({
    queryKey: key(EntityKind.AuditLogsForComponent, componentId),
    queryFn: async ({ pageParam = pageSize }) => {
      const call = auditLogsApi.endpoint<AuditLogsForComponentResponse>(
        routes.AuditLogsForComponent,
        { componentId: componentId.value },
      );
      const response = await call.get(
        new URLSearchParams({
          size: `${pageParam}`,
          sort_ascending: "false",
        }),
      );
      if (auditLogsApi.ok(response)) {
        return response.data;
      }
      return { logs: [], canLoadMore: false };
    },
    staleTime: 60 * 2 * 1000,
    initialPageParam: pageSize,
    getNextPageParam: (lastPage: AuditLogsForComponentResponse) => {
      if (!lastPage.canLoadMore) return undefined;
      return lastPage.logs.length + increaseSize;
    },
    maxPages: 1,
  });

// Flatten all pages and filter out socket-related audit logs
const auditLogs = computed((): ProcessedAuditLog[] => {
  if (!data.value) return [];

  // There should only be one page!
  const allLogs = data.value.pages.flatMap(
    (page: AuditLogsForComponentResponse) => page.logs,
  );

  return allLogs
    .filter((auditLog: AuditLog) => {
      // NOTE(nick,paul,brit): this is intentionally omega hacked. We expect this to change over time.
      if (auditLog.kind === "UpdateDependentProperty") {
        if (
          ["codeItem", "qualificationItem", "resource_value"].includes(
            auditLog.entityName,
          )
        )
          return false;

        // End my suffering.
        const beforeValue = (auditLog.metadata.beforeValue as string) ?? "null";
        const afterValue = (auditLog.metadata.afterValue as string) ?? "null";
        if (beforeValue === afterValue) return false;
      }

      // Filter out sockets.
      if (
        auditLog.kind === "UpdateDependentOutputSocket" ||
        auditLog.kind === "UpdateDependentInputSocket"
      )
        return false;

      // We made it!
      return true;
    })
    .map((filteredAuditLog: AuditLog): ProcessedAuditLog => {
      // Now that we have filtered the audit logs to only those that are relevant to the user, we
      // can change how they are displayed based on the kind.
      if (
        ["UpdateDependentProperty", "SetAttribute", "UnsetAttribute"].includes(
          filteredAuditLog.kind,
        )
      ) {
        if (filteredAuditLog.kind === "UpdateDependentProperty") {
          return {
            inner: filteredAuditLog,
            title: `${filteredAuditLog.entityName} changed`,
            beforeValue:
              (filteredAuditLog.metadata.beforeValue as string) ?? "null",
            afterValue:
              (filteredAuditLog.metadata.afterValue as string) ?? "null",
          };
        } else {
          const beforeValue = filteredAuditLog.metadata.beforeValue as Record<
            string,
            unknown
          >;
          const afterValue = filteredAuditLog.metadata.afterValue as Record<
            string,
            unknown
          >;
          return {
            inner: filteredAuditLog,
            title: `${filteredAuditLog.entityName} changed`,
            beforeValue: (beforeValue.Value as string) ?? "null",
            afterValue: (afterValue.Value as string) ?? "null",
          };
        }
      }

      // By default, display the audit log how we do in the audit trail screen.
      return {
        inner: filteredAuditLog,
        title: `${filteredAuditLog.title} ${filteredAuditLog.entityType}`,
      };
    });
});

const handleScrollEnd = () => {
  if (!scrollContainerRef.value) return;
  if (hasNextPage.value && !isFetchingNextPage.value) {
    fetchNextPage();
  }
};

const timelineForceUpdate = ref(false);
const timelineStyle = computed(() => {
  if (!logRefs.value) return "";

  const lastLog = logRefs.value[logRefs.value.length - 1];

  if (!lastLog) return "";

  // eslint-disable-next-line @typescript-eslint/no-unused-expressions
  timelineForceUpdate.value;

  return `bottom: ${lastLog.clientHeight - 22}px`;
});

const startUpdateTimeline = () => {
  timelineForceUpdate.value = true;
};

const finishUpdateTimeline = () => {
  timelineForceUpdate.value = false;
};
</script>
