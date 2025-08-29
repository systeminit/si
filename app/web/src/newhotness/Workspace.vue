<template>
  <div
    id="app-layout"
    :class="
      clsx('h-screen flex flex-col', themeClasses('bg-white', 'bg-neutral-900'))
    "
  >
    <!-- nav itself is fixed at 60 px-->
    <nav
      :class="
        clsx(
          'navbar relative shadow-[0_4px_4px_0_rgba(0,0,0,0.15)] border-b',
          'z-90 h-[60px] overflow-hidden shrink-0 flex flex-row justify-between select-none',
          'bg-neutral-800 border-neutral-600 text-white',
          windowWidth > 740 && 'gap-sm',
        )
      "
    >
      <!-- Left side -->
      <NavbarPanelLeft
        ref="navbarPanelLeftRef"
        :workspaceId="workspacePk"
        :changeSetId="changeSetId"
        :invalidWorkspace="tokenFail"
      />

      <!-- Center -->
      <NavbarPanelCenter
        v-if="!lobby && !tokenFail"
        :workspaceId="workspacePk"
        :changeSetId="changeSetId"
        :componentId="componentId"
        :viewId="viewId"
        connected
      />

      <!-- Right -->
      <NavbarPanelRight
        :changeSetId="changeSetId"
        :workspaceId="workspacePk"
        :changeSetsNeedingApproval="changeSetsNeedingApproval"
        :invalidWorkspace="tokenFail"
      />
    </nav>
    <div
      v-if="!hasUsedAiAgent"
      class="w-full shrink-0 flex flex-row items-center justify-between bg-[#7D4A1740] border border-warning-500 px-sm py-xs"
    >
      <div class="flex flex-row items-center gap-xs">
        <Icon name="alert-triangle-outline" class="text-warning-500" />
        <span>
          System Initiative works best with an AI Agent.
          <a
            href="https://docs.systeminit.com/tutorials/getting-started"
            class="underline"
            target="_blank"
          >
            Follow these instructions to get started.
          </a>
        </span>
      </div>
    </div>

    <!-- Since lobby hides away the navbar, it's more of an overlay and stays apart from all else -->
    <Transition
      enterActiveClass="duration-300 ease-out"
      enterFromClass="transform opacity-0"
      enterToClass="opacity-100"
      leaveActiveClass="delay-1000 duration-200 ease-in"
      leaveFromClass="opacity-100"
      leaveToClass="transform opacity-0"
    >
      <Lobby
        v-if="!tokenFail && lobby"
        :loadingSteps="_.values(muspelheimStatuses)"
      />
    </Transition>

    <main
      v-if="tokenFail"
      :class="
        clsx(
          'grow min-h-0 m-sm border flex flex-row items-center justify-center',
          themeClasses('border-neutral-400', 'border-neutral-600'),
        )
      "
      :data-error-code="tokenFailStatus"
    >
      <EmptyState
        v-if="tokenFailStatus === 401"
        icon="logo-si"
        text="Unauthorized"
        secondaryText="Your account is not authorized to access this workspace"
        finalLinkText="Click here to see your workspaces"
        @final="bumpToWorkspaces"
      />
      <EmptyState
        v-else-if="tokenFailStatus === 400"
        icon="logo-si"
        text="Invalid workspace id"
        :secondaryText="`A workspace with the id &quot;${workspacePk}&quot; does not exist`"
        finalLinkText="Click here to see your workspaces"
        @final="bumpToWorkspaces"
      />
      <EmptyState
        v-else
        icon="logo-si"
        text="Something went wrong"
        secondaryText="Click here to see your workspaces"
        secondaryClickable
        @secondary="bumpToWorkspaces"
      />
    </main>
    <template v-else-if="!lobby">
      <Onboarding
        v-if="showOnboarding"
        @completed="onboardingCompleted = true"
      />
      <!-- grow the main body to fit all the space in between the nav and the bottom of the browser window
           min-h-0 prevents the main container from being *larger* than the max it can grow, no matter its contents -->
      <main v-else class="grow min-h-0">
        <ComponentPage v-if="componentId" :componentId="componentId" />
        <FuncRunDetails v-else-if="funcRunId" :funcRunId="funcRunId" />
        <LatestFuncRunDetails
          v-else-if="actionId"
          :functionKind="FunctionKind.Action"
          :actionId="actionId"
        />
        <Review v-else-if="onReviewPage" />
        <Explore v-else @openChangesetModal="openChangesetModal" />
      </main>
    </template>
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
import { Icon, themeClasses } from "@si/vue-lib/design-system";
import { storeToRefs } from "pinia";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import * as heimdall from "@/store/realtime/heimdall";
import { useAuthStore } from "@/store/auth.store";
import { useRealtimeStore } from "@/store/realtime/realtime.store";
import {
  ComponentDetails,
  EntityKind,
  OutgoingCounts,
  SchemaMembers,
} from "@/workers/types/entity_kind_types";
import { SchemaId } from "@/api/sdf/dal/schema";
import { ChangeSet, ChangeSetStatus } from "@/api/sdf/dal/change_set";
import { muspelheimStatuses } from "@/store/realtime/heimdall";
import Onboarding from "@/newhotness/Onboarding.vue";
import NavbarPanelRight from "./nav/NavbarPanelRight.vue";
import Lobby from "./Lobby.vue";
import Explore, { GroupByUrlQuery, SortByUrlQuery } from "./Explore.vue";
import FuncRunDetails from "./FuncRunDetails.vue";
import LatestFuncRunDetails from "./LatestFuncRunDetails.vue";
import { Context, FunctionKind, AuthApiWorkspace } from "./types";
import {
  startMouseEmitters,
  startKeyEmitter,
  startWindowResizeEmitter,
  windowResizeEmitter,
} from "./logic_composables/emitters";
import { tokensByWorkspacePk } from "./logic_composables/tokens";
import ComponentPage from "./ComponentDetails.vue";
import NavbarPanelLeft from "./nav/NavbarPanelLeft.vue";
import { useChangeSets } from "./logic_composables/change_set";
import { routes, useApi } from "./api_composables";
import Review from "./Review.vue";
import EmptyState from "./EmptyState.vue";
import NavbarPanelCenter from "./nav/NavbarPanelCenter.vue";

const AUTH_PORTAL_URL = import.meta.env.VITE_AUTH_PORTAL_URL;

const tracer = trace.getTracer("si-vue");
const navbarPanelLeftRef = ref<InstanceType<typeof NavbarPanelLeft>>();
const span = ref<Span | undefined>();

const route = useRoute();
const router = useRouter();

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
const featureFlagsStore = useFeatureFlagsStore();
const realtimeStore = useRealtimeStore();

const workspacePk = computed(() => props.workspacePk);
const changeSetId = computed(() => props.changeSetId);

const hasUsedAiAgent = computed(
  () => authStore.userWorkspaceFlags.executedAgent ?? false,
);

const coldStartInProgress = computed(() => heimdall.muspelheimInProgress.value);

// const haveWSConn = computed<boolean>(() => {
//   if (!heimdall.initCompleted.value) return true; // dont show error if we havent gotten started
//   return !!heimdall.wsConnections.value[props.workspacePk];
// });

// no tan stack queries hitting sqlite until after the cold start has finished
const queriesEnabled = computed(
  () => heimdall.initCompleted.value && !coldStartInProgress.value,
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

const changeSet = ref<ChangeSet | undefined>();
const _headChangeSetId = ref<string>("");
const approvers = ref<string[]>([]);
const onboardingCompleted = ref(false);
const reopenOnboarding = () => {
  onboardingCompleted.value = false;
};

const { user, userWorkspaceFlags } = storeToRefs(authStore);

const ctx = computed<Context>(() => {
  return {
    workspacePk,
    changeSetId,
    changeSet,
    approvers,
    user: user.value,
    userWorkspaceFlags,
    onHead: computed(() => {
      return changeSetId.value === _headChangeSetId.value;
    }),
    headChangeSetId: _headChangeSetId,
    outgoingCounts,
    componentDetails,
    schemaMembers,
    queriesEnabled,
    reopenOnboarding,
  };
});

const workspaceApi = useApi(ctx.value);
const workspaceQuery = useQuery<Record<string, AuthApiWorkspace>>({
  queryKey: ["workspaces"],
  staleTime: 5000,
  queryFn: async () => {
    const call = workspaceApi.endpoint<AuthApiWorkspace[]>(routes.Workspaces);
    const response = await call.get();
    if (workspaceApi.ok(response)) {
      const renameIdList = _.map(response.data, (w) => ({
        ...w,
        pk: w.id,
      }));
      const workspacesByPk = _.keyBy(renameIdList, "pk");
      return workspacesByPk;
    }
    return {} as Record<string, AuthApiWorkspace>;
  },
});

const workspaces = computed(() => {
  return {
    workspaces: computed(() => workspaceQuery.data.value),
  };
});
provide("WORKSPACES", workspaces.value);

const {
  openChangeSets,
  changeSet: activeChangeSet,
  headChangeSetId,
  defaultApprovers,
} = useChangeSets(ctx, queriesEnabled);

const changeSetsNeedingApproval = computed(() =>
  openChangeSets.value.filter(
    (cs) => cs.status === ChangeSetStatus.NeedsApproval,
  ),
);

watch(
  defaultApprovers,
  () => {
    approvers.value = defaultApprovers.value;
  },
  { immediate: true },
);
watch(
  activeChangeSet,
  () => {
    changeSet.value = activeChangeSet.value;
  },
  { immediate: true },
);
watch(
  headChangeSetId,
  () => {
    // never assign a blank value over a truth-y value
    if (_headChangeSetId.value && !headChangeSetId.value) return;

    _headChangeSetId.value = headChangeSetId.value;
  },
  { immediate: true },
);
watch(
  () => openChangeSets,
  () => {
    const ids = openChangeSets.value.map((c) => c.id);
    if (ids.length > 0 && !ids.includes(props.changeSetId)) {
      if (ctx.value.headChangeSetId.value) {
        router.push({
          name: "new-hotness",
          params: {
            workspacePk: props.workspacePk,
            changeSetId: headChangeSetId.value,
          },
        });
      } else {
        router.push({
          name: "new-hotness-workspace",
          params: {
            workspacePk: props.workspacePk,
          },
        });
      }
    }
  },
  { immediate: true, deep: true },
);

startKeyEmitter(document);
startMouseEmitters(window);
startWindowResizeEmitter(window);

provide("CONTEXT", ctx.value);

// LOBBY AND ONBOARDING FLAGS
const userPk = computed(() => authStore.user?.pk);
const checkOnboardingCompleteApi = useApi(ctx.value);
const loadedOnboardingStateFromApi = ref(false);
const checkOnboardingCompleteData = async () => {
  if (!userPk.value) return;

  const call = checkOnboardingCompleteApi.endpoint<{ firstTimeModal: boolean }>(
    routes.CheckDismissedOnboarding,
    { userPk: userPk.value },
  );
  const { data, status } = await call.get();

  if (status !== 200) {
    tokenFail.value = true;
    tokenFailStatus.value = status;
    return;
  }

  loadedOnboardingStateFromApi.value = true;
  // firstTimeModal === true means that we should SHOW it, so we need to invert it to see if it's been dismissed.
  onboardingCompleted.value = !data.firstTimeModal;
};

const lobby = computed(
  () =>
    coldStartInProgress.value ||
    // not looking for feature flag values in here (if for any reason, network connectivity, posthog being down, we didn't get feature flags everyone would be stuck in the lobby)
    loadedOnboardingStateFromApi.value === false, // Don't continue before we got the response from the auth API
);

const showOnboarding = computed(() => {
  if (!featureFlagsStore.INITIALIZER_ONBOARD) return false;

  return !onboardingCompleted.value;
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

export type SelectionsInQueryString = Partial<{
  map: string;
  grid: string;
  c: string;
  s: string; // represents the selected component indexes
  b: string; // 0 or 1, represents "in bulk editing" for ^ selected
  groupBy: GroupByUrlQuery;
  sortBy: SortByUrlQuery;
  pinned: string;
  defaultSubscriptions: string;
  viewId: string;
  searchQuery: string;
  retainSessionState: string; // If set, the component should load up with the last state it had on this tab. Used by Explore.vue
  hideSubscriptions: string; // Flag to hide unconnected components when navigating to map
}>;

const tokenFailStatus = ref();
const tokenFail = ref(false);

const queryClient = useQueryClient();
queryClient.setDefaultOptions({ queries: { staleTime: Infinity } });

const container = inject<{ loadingGuard: Ref<boolean> }>("LOADINGGUARD");

const getTokenForWorkspace = (workspaceId: string) => {
  return tokensByWorkspacePk[workspaceId];
};

onMounted(async () => {
  await checkOnboardingCompleteData();
});

onBeforeMount(async () => {
  if (container && container.loadingGuard.value) {
    return;
  }

  if (container) {
    container.loadingGuard.value = true;
  }

  const thisWorkspacePk = workspacePk.value;
  const workspaceAuthToken = getTokenForWorkspace(thisWorkspacePk);
  if (!workspaceAuthToken) {
    tokenFail.value = true;
    tokenFailStatus.value = 500;
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
  // TODO Add deployment init when we start handling deployment patches and indices
});

watch(
  () => [props.workspacePk, props.changeSetId],
  async ([newWorkspacePk, newChangeSetId], _) => {
    if (newWorkspacePk && newChangeSetId) {
      const workspaceToken = getTokenForWorkspace(newWorkspacePk);
      if (!workspaceToken) {
        tokenFail.value = true;
        tokenFailStatus.value = 401;
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
        queryClient.invalidateQueries({ queryKey: ["changesets"] });
        if (ctx.value.headChangeSetId.value) {
          await heimdall.linkNewChangeset(
            props.workspacePk,
            data.changeSetId,
            ctx.value.headChangeSetId.value,
          );
        }
      },
    },
    {
      eventType: "ChangeSetStatusChanged",
      callback: async (data) => {
        queryClient.invalidateQueries({ queryKey: ["changesets"] });
        queryClient.invalidateQueries({
          queryKey: ["approvalstatus", data.changeSet.id],
        });
        queryClient.invalidateQueries({
          queryKey: ["approvalstatusbychangesetid", data.changeSet.id],
        });
        /* TURN THIS ON WHEN WE REMOVE CHANGE SET STORE
        if (
          [
            ChangeSetStatus.Abandoned,
            ChangeSetStatus.Applied,
            ChangeSetStatus.Closed,
          ].includes(data.changeSet.status) &&
          data.changeSet.id !== ctx.value.headChangeSetId.value
        ) {
          heimdall.prune(props.workspacePk, data.changeSet.id);
        }
        // If I'm the one who requested this change set - toast that it's been approved/rejected/etc.
        if (data.changeSet.mergeRequestedByUserId === authStore.user?.pk) {
          if (data.changeSet.status === ChangeSetStatus.Rejected) {
            toast({
              component: ChangeSetStatusChanged,
              props: {
                user: data.changeSet.reviewedByUser,
                command: "rejected the request to apply",
                changeSetName: data.changeSet.name,
              },
            });
          } else if (data.changeSet.status === ChangeSetStatus.Approved) {
            toast({
              component: ChangeSetStatusChanged,
              props: {
                user: data.changeSet.reviewedByUser,
                command: "approved the request to apply",
                changeSetName: data.changeSet.name,
              },
            });
          }
        } else if (data.changeSet.status === ChangeSetStatus.NeedsApproval) {
          // this.FETCH_APPROVAL_STATUS(data.changeSet.id);
          // APPROVALS TODO, all the other change set store listeners
        }
        */
      },
    },
    {
      eventType: "ChangeSetAbandoned",
      callback: async (_data) => {
        queryClient.invalidateQueries({ queryKey: ["changesets"] });

        /* TURN THIS ON WHEN WE REMOVE CHANGE SET STORE
        if (
        if (data.changeSetId !== ctx.value.headChangeSetId.value) {
          heimdall.prune(props.workspacePk, data.changeSetId);
        }

        if (data.changeSetId === props.changeSetId) {
          if (headChangeSetId.value) {
            const params = { ...route.params };
            delete params.componentId;
            await router.push({
              name: "new-hotness-head",
              params: {
                ...params,
                changeSetId: ctx.value.headChangeSetId.value,
              },
            });

            toast({
              component: MovedToHead,
              props: {
                icon: "trash",
                changeSetName: activeChangeSet.value?.name,
                action: "abandoned",
              },
            });
          }
        }
        */
      },
    },
    {
      eventType: "ChangeSetCancelled",
      callback: () => {
        queryClient.invalidateQueries({ queryKey: ["changesets"] });
      },
    },
    {
      eventType: "ChangeSetApplied",
      callback: (_data) => {
        queryClient.invalidateQueries({ queryKey: ["changesets"] });

        /* TURN THIS ON WHEN WE REMOVE CHANGE SET STORE
        if (
        const { changeSetId: appliedId, toRebaseChangeSetId } = data;
        if (activeChangeSet) {
          if (appliedId !== ctx.value.headChangeSetId.value) {
            // never set HEAD to Applied
            heimdall.prune(props.workspacePk, appliedId);
          }
        }

        if (
          props.changeSetId === toRebaseChangeSetId &&
          props.changeSetId !== headChangeSetId.value
        ) {
          toast({
            component: RebaseOnBase,
          });
        }

        // `list_open_change_sets` gets called prior on voters
        // which means the change set is gone, so always move
        if (!props.changeSetId || props.changeSetId === appliedId) {
          if (route?.name) {
            if ((route.name as string).startsWith("new-hotness")) {
              let name = route.name as string;
              // if you're on a single-item page in the UI while just bring them to explore
              if (
                ![
                  "new-hotness",
                  "new-hotness-workspace-auto",
                  "new-hotness-head",
                ].includes(route.name as string)
              )
                name = "new-hotness";

              router.push({
                name,
                params: {
                  ...route.query, // this will keep map/grid, search, if its there
                  changeSetId: headChangeSetId.value,
                },
              });
            } else {
              router.push({
                name: route.name,
                params: {
                  ...route.params,
                  changeSetId: headChangeSetId.value,
                },
              });
            }
            if (!ctx.value.onHead.value)
              // FIXME(nick): use a new design in the new UI, but keep the toast.
              toast({
                component: MovedToHead,
                props: {
                  icon: "tools",
                  changeSetName: activeChangeSet.value?.name,
                  action: "merged",
                },
              });
          }
        }
        */
      },
    },
    {
      eventType: "ChangeSetRename",
      callback: () => {
        queryClient.invalidateQueries({ queryKey: ["changesets"] });
      },
    },
    {
      eventType: "ChangeSetApprovalStatusChanged",
      callback: (changeSetId) => {
        queryClient.invalidateQueries({
          queryKey: ["approvalstatus", changeSetId],
        });
        queryClient.invalidateQueries({
          queryKey: ["approvalstatusbychangesetid", changeSetId],
        });
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

const bumpToWorkspaces = () => {
  window.open(`${AUTH_PORTAL_URL}/workspaces/`, "_self");
};

const onReviewPage = computed(() => route.name === "new-hotness-review");

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

/* ================================================================================================
 * Shimmer for skeleton divs
 * ================================================================================================
 */
.skeleton-shimmer {
  position: relative;
  overflow: hidden;
}

.skeleton-shimmer::before {
  content: "";
  position: absolute;
  top: 0;
  left: -100%;
  width: 100%;
  height: 100%;
  animation: shimmer 1.5s infinite;
}

/* Light theme shimmer */
body.light .skeleton-shimmer::before {
  background: linear-gradient(
    90deg,
    transparent,
    rgba(255, 255, 255, 0.8),
    transparent
  );
}

/* Dark theme shimmer */
body.dark .skeleton-shimmer::before {
  background: linear-gradient(
    90deg,
    transparent,
    rgba(255, 255, 255, 0.1),
    transparent
  );
}

@keyframes shimmer {
  0% {
    left: -100%;
  }
  100% {
    left: 100%;
  }
}

/* ================================================================================================
 * END Shimmer for skeleton divs
 * ================================================================================================
 */
</style>
