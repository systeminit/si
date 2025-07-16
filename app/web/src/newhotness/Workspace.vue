<template>
  <div id="app-layout" class="h-screen flex flex-col">
    <!-- nav itself is fixed at 60 px-->
    <nav
      :class="
        clsx(
          'navbar bg-neutral-900 text-white relative shadow-[0_4px_4px_0_rgba(0,0,0,0.15)]',
          'z-90 h-[60px] overflow-hidden shrink-0 flex flex-row justify-between select-none',
          windowWidth > 740 && 'gap-sm',
        )
      "
    >
      <!-- Left side -->
      <NavbarPanelLeft ref="navbarPanelLeftRef" />

      <!-- Center -->
      <div
        v-if="!lobby"
        class="flex flex-row flex-none items-center h-full justify-center place-items-center mx-auto overflow-hidden"
      >
        <NavbarButton
          tooltipText="Compose"
          icon="diagram"
          :selected="route.name?.toString().startsWith('new-hotness')"
          :linkTo="compositionLink"
        />

        <NavbarButton
          tooltipText="Customize"
          icon="beaker"
          :selected="route.matched.some((r) => r.name === 'workspace-lab')"
          :linkTo="{
            path: `/w/${workspacePk}/${changeSetId}/l`,
          }"
        />

        <NavbarButton
          tooltipText="Audit"
          icon="eye"
          :selected="route.matched.some((r) => r.name === 'workspace-audit')"
          :linkTo="{
            path: `/w/${workspacePk}/${changeSetId}/a`,
          }"
        />
      </div>

      <!-- Right -->
      <NavbarPanelRight />
    </nav>

    <!-- grow the main body to fit all the space in between the nav and the bottom of the browser window
     min-h-0 prevents the main container from being *larger* than the max it can grow, no matter its contents -->
    <main v-if="!lobby" class="grow min-h-0">
      <div v-if="tokenFail">Bad Token</div>
      <ComponentPage v-else-if="componentId" :componentId="componentId" />
      <FuncRunDetails v-else-if="funcRunId" :funcRunId="funcRunId" />
      <LatestFuncRunDetails
        v-else-if="actionId"
        :functionKind="FunctionKind.Action"
        :actionId="actionId"
      />
      <Explore v-else @openChangesetModal="openChangesetModal" />
    </main>

    <!-- Since lobby hides away the navbar, it's more of an overlay and stays apart from all else -->
    <Transition
      enterActiveClass="duration-300 ease-out"
      enterFromClass="transform opacity-0"
      enterToClass="opacity-100"
      leaveActiveClass="delay-1000 duration-200 ease-in"
      leaveFromClass="opacity-100"
      leaveToClass="transform opacity-0"
    >
      <Lobby v-if="!tokenFail && lobby" />
    </Transition>
  </div>
</template>

<script lang="ts" setup>
import { useRoute, useRouter } from "vue-router";
import clsx from "clsx";
import {
  computed,
  onMounted,
  onBeforeUnmount,
  onBeforeMount,
  ref,
  provide,
  watch,
  Ref,
  inject,
} from "vue";
import * as _ from "lodash-es";
import { useQuery, useQueryClient } from "@tanstack/vue-query";
import { Span, trace } from "@opentelemetry/api";
import NavbarPanelLeft from "@/components/layout/navbar/NavbarPanelLeft.vue";
import NavbarPanelRight from "@/components/layout/navbar/NavbarPanelRight.vue";
import NavbarButton from "@/components/layout/navbar/NavbarButton.vue";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import * as heimdall from "@/store/realtime/heimdall";
import { useAuthStore } from "@/store/auth.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useRealtimeStore } from "@/store/realtime/realtime.store";
import {
  ComponentDetails,
  EntityKind,
  OutgoingCounts,
  SchemaMembers,
} from "@/workers/types/entity_kind_types";
import { SchemaId } from "@/api/sdf/dal/schema";
import Lobby from "./Lobby.vue";
import Explore, { GroupByUrlQuery, SortByUrlQuery } from "./Explore.vue";
import FuncRunDetails from "./FuncRunDetails.vue";
import LatestFuncRunDetails from "./LatestFuncRunDetails.vue";
import { Context, FunctionKind } from "./types";
import {
  startMouseEmitters,
  startKeyEmitter,
  startWindowResizeEmitter,
  windowResizeEmitter,
} from "./logic_composables/emitters";
import { tokensByWorkspacePk } from "./logic_composables/tokens";
import ComponentPage from "./ComponentDetails.vue";

const tracer = trace.getTracer("si-vue");
const navbarPanelLeftRef = ref<InstanceType<typeof NavbarPanelLeft>>();

const props = defineProps<{
  workspacePk: string;
  changeSetId: string;
  componentId?: string;
  viewId?: string;
  secretId?: string;
  funcRunId?: string;
  actionId?: string;
}>();

const authStore = useAuthStore();

const span = ref<Span | undefined>();

const lobby = computed(() => {
  const muspelheimStates = heimdall.muspelheimStatuses.value;
  if (Object.keys(muspelheimStates).length === 0) {
    return true;
  }

  for (const changeSetId in muspelheimStates) {
    if (!muspelheimStates[changeSetId]) {
      return true;
    }
  }
  return false;
});

watch(
  lobby,
  () => {
    if (!span.value && lobby.value) {
      span.value = tracer.startSpan("lobby");
      span.value.setAttributes({
        ...props,
        user: authStore.user?.pk,
        "si.workspace.id": props.workspacePk,
        "si.changeset.id": props.changeSetId,
      });
    }

    if (span.value && !lobby.value) {
      span.value.end();
      span.value = undefined;
    }
  },
  { immediate: true },
);

const featureFlagsStore = useFeatureFlagsStore();
const changeSetsStore = useChangeSetsStore();
const realtimeStore = useRealtimeStore();

const workspacePk = computed(() => props.workspacePk);
const changeSetId = computed(() => props.changeSetId);

// no tan stack queries hitting sqlite until after the cold start has finished
const queriesEnabled = computed(
  () => heimdall.initCompleted.value && !lobby.value,
);

const countsQueryKey = computed(() => {
  return [
    workspacePk.value,
    changeSetId.value,
    EntityKind.OutgoingCounts,
    workspacePk.value,
  ];
});
const args = computed(() => {
  return {
    workspaceId: workspacePk.value,
    changeSetId: changeSetId.value,
  };
});
const countsQuery = useQuery<OutgoingCounts>({
  queryKey: countsQueryKey,
  enabled: queriesEnabled,
  queryFn: async () => await heimdall.getOutgoingConnectionsCounts(args.value),
});

const namesQueryKey = computed(() => {
  return [
    workspacePk.value,
    changeSetId.value,
    EntityKind.ComponentDetails,
    workspacePk.value,
  ];
});
const namesQuery = useQuery<ComponentDetails>({
  queryKey: namesQueryKey,
  enabled: queriesEnabled,
  queryFn: async () => {
    return await heimdall.getComponentDetails(args.value);
  },
});
const schemaQueryKey = computed(() => {
  return [
    workspacePk.value,
    changeSetId.value,
    EntityKind.SchemaMembers,
    workspacePk.value,
  ];
});
const schemaQuery = useQuery<Record<SchemaId, SchemaMembers>>({
  queryKey: schemaQueryKey,
  enabled: queriesEnabled,
  queryFn: async () => {
    const data = await heimdall.getSchemaMembers(args.value);
    return data.reduce((obj, s) => {
      obj[s.id] = s;
      return obj;
    }, {} as Record<SchemaId, SchemaMembers>);
  },
});

const outgoingCounts = computed(() => {
  return countsQuery.data.value ?? {};
});

const componentDetails = computed(() => {
  return namesQuery.data.value ?? {};
});

const schemaMembers = computed(() => {
  return schemaQuery.data.value ?? {};
});

const ctx = computed<Context>(() => {
  return {
    workspacePk,
    changeSetId,
    user: authStore.user,
    onHead: computed(() => changeSetsStore.headSelected),
    headChangeSetId: computed(() => changeSetsStore.headChangeSetId ?? ""),
    outgoingCounts,
    componentDetails,
    schemaMembers,
    queriesEnabled,
  };
});

watch(
  () => changeSetsStore.changeSetsById,
  () => {
    const ids = Object.keys(changeSetsStore.changeSetsById);
    if (ids.length > 0 && !ids.includes(props.changeSetId))
      if (changeSetsStore.headChangeSetId)
        router.push({
          name: "new-hotness",
          params: {
            workspacePk: props.workspacePk,
            changeSetId: changeSetsStore.headChangeSetId,
          },
        });
      else
        router.push({
          name: "new-hotness-workspace",
          params: {
            workspacePk: props.workspacePk,
          },
        });
  },
  { immediate: true },
);

startKeyEmitter(document);
startMouseEmitters(window);
startWindowResizeEmitter(window);

provide("CONTEXT", ctx.value);

export type SelectionsInQueryString = Partial<{
  map: string;
  grid: string;
  c: string;
  groupBy: GroupByUrlQuery;
  sortBy: SortByUrlQuery;
  pinned: string;
  viewId: string;
  searchQuery: string;
}>;

const compositionLink = computed(() => {
  // eslint-disable-next-line no-nested-ternary
  const name = props.componentId
    ? "new-hotness-component"
    : props.viewId
    ? "new-hotness-view"
    : "new-hotness";
  return {
    name,
    params: props,
  };
});

const route = useRoute();
const router = useRouter();

const tokenFail = ref(false);

const queryClient = useQueryClient();
queryClient.setDefaultOptions({ queries: { staleTime: Infinity } });

const container = inject<{ loadingGuard: Ref<boolean> }>("LOADINGGUARD");

const getTokenForWorkspace = (workspaceId: string) => {
  return tokensByWorkspacePk[workspaceId];
};

onBeforeMount(async () => {
  if (container && container.loadingGuard.value) {
    return;
  }

  if (container) {
    container.loadingGuard.value = true;
  }
  // NOTE(nick,wendy): if you do not have the flag enabled, you will be re-directed. This will be
  // true for all of the new hotness routes, provided that they are all children of the parent
  // route that uses this component. This is wrapped in a "setTimeout" to ensure that the feature
  // flag loads in time.
  setTimeout(() => {
    if (!featureFlagsStore.ENABLE_NEW_EXPERIENCE) {
      router.push({ name: "workspace-index" });
    }
  }, 500);

  const thisWorkspacePk = workspacePk.value;
  const workspaceAuthToken = getTokenForWorkspace(thisWorkspacePk);
  if (!workspaceAuthToken) {
    tokenFail.value = true;
    return;
  }

  // Activate the norse stack, which is explicitly NOT flagged for the job-specific UI.
  await heimdall.init(thisWorkspacePk, workspaceAuthToken, queryClient);
  watch(
    [connectionShouldBeEnabled, heimdall.initCompleted],
    async () => {
      if (connectionShouldBeEnabled.value && heimdall.initCompleted.value) {
        // NOTE(nick,wendy): this says "reconnect", but it must run once on startup.
        await heimdall.bifrostReconnect();
      }
    },
    { immediate: true },
  );

  // NOTE: onBeforeMount doesn't wait on promises
  // the page will load before execution finishes
  await heimdall.muspelheim(thisWorkspacePk, true);
});

watch(
  () => [props.workspacePk, props.changeSetId],
  async ([newWorkspacePk, newChangeSetId], _) => {
    if (newWorkspacePk && newChangeSetId) {
      const workspaceToken = getTokenForWorkspace(newWorkspacePk);
      if (!workspaceToken) {
        tokenFail.value = true;
        return;
      }
      await heimdall.registerBearerToken(newWorkspacePk, workspaceToken);
      heimdall.niflheim(newWorkspacePk, newChangeSetId, true, false);
    }
  },
);

const hiddenAt = ref<Date | undefined>(undefined);

// Force muspelheim if the window has been hidden for 12 hours
const FORCE_MUSPELHEIM_AFTER_MS = 12 * 60 * 60 * 1000;

// We have put this in a watch, instead of in the event
// listener, in order to ensure we are reactive to the prop
watch(hiddenAt, (newValue, oldValue) => {
  if (!newValue && oldValue) {
    const now = new Date();
    const timeDiff = now.getTime() - oldValue.getTime();

    if (timeDiff >= FORCE_MUSPELHEIM_AFTER_MS) {
      heimdall.muspelheim(props.workspacePk, true);
    }
  }
});

document.addEventListener("visibilitychange", () => {
  if (document.hidden) {
    hiddenAt.value = new Date();
  } else {
    hiddenAt.value = undefined;
  }
});

realtimeStore.subscribe(
  "TOP_LEVEL_WORKSPACE",
  `workspace/${props.workspacePk}`,
  [
    {
      eventType: "ChangeSetCreated",
      callback: async (data) => {
        if (ctx.value.headChangeSetId.value) {
          await heimdall.linkNewChangeset(
            props.workspacePk,
            data.changeSetId,
            ctx.value.headChangeSetId.value,
          );
        }
      },
    },
  ],
);

const connectionShouldBeEnabled = computed(() => {
  try {
    const authStore = useAuthStore();
    return (
      authStore.userIsLoggedInAndInitialized && authStore.selectedWorkspaceToken
    );
  } catch (_err) {
    return false;
  }
});

const windowWidth = ref(window.innerWidth);

const windowResizeHandler = () => {
  windowWidth.value = window.innerWidth;
};

const openChangesetModal = () => {
  navbarPanelLeftRef.value?.openCreateModal();
};

const funcRunKey = "paginatedFuncRuns";

// Invalidates the paginatedFuncRuns query so it can update.
//
// We debounce because we frequently get a bunch of FuncRunLogUpdated events in a row, and we
// don't want to query the server over and over again.
//
// TODO we can't set the debounce much lower because the FuncRunLogUpdated WsEvent is emitted
// *before* the data is committed or rebased or something. In short, if you query
// for it too quickly, you will not get the updated data. We wait a little bit to
// give ourselves a better chance.
//
// Ultimately we need the WsEvent to fire *after* the data is updated. Then we can make this
// debounce a little shorter for a snappier UX.
const invalidatePaginatedFuncRuns = _.debounce(() => {
  const queryKey = [ctx.value.changeSetId, "paginatedFuncRuns"];
  // If the query took longer than 500ms, invalidating would cancel it, and we might never
  // actually finish! We'll just requeue later when it's done.
  if (queryClient.isFetching({ queryKey }) > 0) {
    invalidatePaginatedFuncRuns();
    return;
  }
  queryClient.invalidateQueries({ queryKey });
}, 500);

onMounted(() => {
  windowResizeHandler();
  windowResizeEmitter.on("resize", windowResizeHandler);
});

const invalidateOneFuncRun = _.debounce((funcRunId: string) => {
  const queryKey = [ctx.value.changeSetId, "funcRunLogs", funcRunId];
  if (queryClient.isFetching({ queryKey }) > 0) {
    invalidateOneFuncRun(funcRunId);
    return;
  }
  queryClient.invalidateQueries({ queryKey });
}, 500);

watch(
  ctx.value.changeSetId,
  () => {
    // stop listening to old change set
    realtimeStore.unsubscribe(funcRunKey);
    // listen to new change set
    // Invalidate the paginatedFuncRuns query when FuncRunLogUpdated events are received.
    realtimeStore.subscribe(
      funcRunKey,
      `changeset/${ctx.value.changeSetId.value}`,
      [
        {
          eventType: "FuncRunLogUpdated",
          callback: async (payload) => {
            if (payload.funcRunId) {
              invalidatePaginatedFuncRuns();
              invalidateOneFuncRun(payload.funcRunId);
            }
          },
        },
      ],
    );
  },
  { immediate: true },
);

onBeforeUnmount(() => {
  windowResizeEmitter.off("resize", windowResizeHandler);
  realtimeStore.unsubscribe(funcRunKey);
});
</script>

<style lang="less">
* {
  box-sizing: border-box;
  /* other global styles */
}

/* use on any div in a grid that you want to have scrollable content */
.scrollable {
  overflow-y: auto;
  scrollbar-width: thin;
}
body.dark .scrollable {
  scrollbar-color: @colors-neutral-800 @colors-black;
}
body.light .scrollable {
  scrollbar-color: @colors-neutral-200 @colors-white;
}

.grid > .scrollable {
  transition: max-height 1s;
}

/* any grid that has scrollable elements needs min-height 0 (just like main, above) otherwise the contents of the scrollable can blow out the container (putting it here globally means a human doesnt need to remember to do it every time) */
.grid:has(> .scrollable) {
  min-height: 0;
}

.tilegrid {
  display: grid;
  grid-gap: 1rem;

  > .tile {
    &.pinned {
      grid-column: 1 / -1;
    }
  }
}

/* This rules determines
 * - the min card width
 * - fit as many on a row
 * - and since its a grid, every card will have the same height (as tall as the tallest card)
 */
@supports (width: min(250px, 100%)) {
  .tilegrid {
    grid-template-columns: repeat(auto-fit, minmax(min(250px, 100%), 1fr));
    grid-auto-rows: min-content;
  }
}

// this is a LESS mixin, right now we've got 2 different logical elements with the same grid
// let's re-use the definition only once, until we don't want these elements to follow the same grid
.MixinGridIconsFlankingHeaderAndSubheader {
  display: grid;
  grid-row-gap: 2px;
  grid-column-gap: 0.5rem;
  // 32px is the icon size
  grid-template-columns: 32px minmax(0, 1fr) 32px;
  grid-template-rows: 16px 16px;
  grid-template-areas:
    "logo h2 spinner"
    "logo h3 spinner";

  // the icons are divs
  > div:first-child {
    grid-area: "logo";
  }
  > div:last-child {
    grid-area: spinner;
  }
  > h2 {
    grid-area: h2;
    font-weight: bold;
  }
  > h3 {
    grid-area: h3;
    font-size: 0.8rem;
  }
}

.actions.list {
  .item {
    // actions list items also follow the grid
    .MixinGridIconsFlankingHeaderAndSubheader();
  }
}

// component cards look like this, everywhere in the app
.tile.component {
  display: flex;
  flex-direction: column;

  > header {
    // this header follows the grid from the mixin
    .MixinGridIconsFlankingHeaderAndSubheader();
  }

  > footer {
    // always place a gap between buttons
    button + button {
      margin-left: 0.5rem;
    }
  }
}

// other cards may look differently, can be defined globally, or piecemeal

/* TODO(Wendy) - temporary color classes for the new UI */
.text-purple {
  color: #d4b4fe;
}

.text-green-light-mode {
  color: #3b8e48;
}

.text-green-dark-mode {
  color: #aafec7;
}
</style>
