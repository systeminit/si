import { computed, MaybeRefOrGetter, ref, toValue } from "vue";
import { useQuery } from "@tanstack/vue-query";
import { useRoute } from "vue-router";
import {
  ActionId,
  ActionKind,
  ActionPrototypeId,
  ActionState,
} from "@/api/sdf/dal/action";
import {
  BifrostComponent,
  ActionPrototypeView,
  EntityKind,
  ActionPrototypeViewList,
  BifrostActionViewList,
  ComponentInList,
} from "@/workers/types/entity_kind_types";
import {
  bifrost,
  bifrostExists,
  useMakeArgs,
  useMakeArgsForHead,
  useMakeKey,
  useMakeKeyForHead,
} from "@/store/realtime/heimdall";
import { routes, useApi } from "../api_composables";
import { ActionProposedView } from "../types";
import { useContext } from "./context";

export const useComponentActions = (
  componentRef: MaybeRefOrGetter<
    BifrostComponent | ComponentInList | undefined
  >,
) => {
  const makeKey = useMakeKey();
  const makeKeyForHead = useMakeKeyForHead();
  const makeArgs = useMakeArgs();
  const makeArgsForHead = useMakeArgsForHead();
  const ctx = useContext(); // NOTE(nick): this should likely be passed in...
  const route = useRoute();

  const component = computed(() => toValue(componentRef));
  const schemaVariantId = computed(() => {
    const comp = component.value;
    if (!comp) return "";

    // Handle both string (ComponentInList) and WeakReference (BifrostComponent) types
    const variantId = comp.schemaVariantId;
    if (typeof variantId === "string") {
      return variantId;
    }

    return variantId.id;
  });
  const queryKeyForActionPrototypeViews = makeKey(
    EntityKind.ActionPrototypeViewList,
    schemaVariantId,
  );

  const actionPrototypeViewsRaw = useQuery<ActionPrototypeViewList | null>({
    enabled: computed(() => ctx.queriesEnabled.value && !!component.value),
    queryKey: queryKeyForActionPrototypeViews,
    queryFn: async () => {
      if (!schemaVariantId.value) return null;
      return await bifrost<ActionPrototypeViewList>(
        makeArgs(EntityKind.ActionPrototypeViewList, schemaVariantId.value),
      );
    },
  });
  const actionPrototypeViews = computed(
    () => actionPrototypeViewsRaw.data.value?.actionPrototypes ?? [],
  );

  const queryKeyForComponentOnHead = makeKeyForHead(
    EntityKind.ComponentInList,
    component.value?.id,
  );
  const componentOnHeadRaw = useQuery({
    enabled: computed(() => ctx.queriesEnabled.value && !!component.value),
    queryKey: queryKeyForComponentOnHead,
    queryFn: async () => {
      if (!component.value) return null;
      return await bifrostExists(
        makeArgsForHead(EntityKind.ComponentInList, component.value.id),
      );
    },
  });
  const componentExistsOnHead = computed(() => !!componentOnHeadRaw.data.value);

  // Use the materialized view for actions to know what actions exist for a given prototype and the
  // selected component.
  const queryKeyForActionViewList = makeKey(EntityKind.ActionViewList);
  const actionViewList = useQuery<BifrostActionViewList | null>({
    queryKey: queryKeyForActionViewList,
    queryFn: async () =>
      await bifrost<BifrostActionViewList>(makeArgs(EntityKind.ActionViewList)),
  });

  const actionByPrototype = computed(() => {
    if (!actionViewList.data.value?.actions?.length || !component.value?.id) {
      return {};
    }

    const result: Record<ActionPrototypeId, ActionProposedView> = {};
    for (const action of actionViewList.data.value.actions) {
      if (action.componentId === component.value.id) {
        // When in a change set (not HEAD), only show actions that originated in the current change set
        if (
          !ctx.onHead.value &&
          action.originatingChangeSetId !== ctx.changeSetId.value
        ) {
          continue;
        }
        // NOTE(nick): this assumes that there can be one action for a given prototype and component.
        // As of the time of writing, this is true, but multiple actions per prototype and component
        // aren't disallowed from the underlying graph's perspective. Theorhetically, you could
        // enqueue two refreshes back-to-back. What then? I don't think we'll expose an interface to
        // do that for awhile, so this should be sufficient.
        result[action.prototypeId] = action;
      }
    }
    return result;
  });

  const actionIsRunning = (actionId?: ActionId | null) => {
    if (!actionId) return false;
    const state = actionViewList.data.value?.actions.find(
      (action) => action.id === actionId,
    )?.state;
    if (!state) return false;
    return [ActionState.Dispatched, ActionState.Running].includes(state);
  };

  // Helpers for Refresh Actions specifically
  const refreshActionPrototype = computed(() => {
    return (
      actionPrototypeViews.value.find(
        (action: ActionPrototypeView) => action.kind === ActionKind.Refresh,
      ) ?? null
    );
  });

  const refreshAction = computed(() => {
    if (!refreshActionPrototype.value?.id) return null;
    return actionByPrototype.value[refreshActionPrototype.value.id] ?? null;
  });

  const refreshActionRunning = computed(() => {
    return refreshAction.value
      ? actionIsRunning(refreshAction.value?.id)
      : false;
  });

  const refreshEnabled = computed(
    () =>
      !!(
        refreshActionPrototype.value &&
        component.value?.hasResource &&
        (ctx.onHead.value || componentExistsOnHead.value)
      ),
  );

  const runRefreshHandler = () => {
    const directRefreshApi = useApi(ctx);

    const executeRefresh = async () => {
      if (!component.value?.id || refreshActionRunning.value) return;
      const call = directRefreshApi.endpoint(routes.RefreshAction, {
        componentId: component.value.id,
      });
      directRefreshApi.setWatchFn(() => refreshAction.value?.state);
      await call.put({});
    };

    const bifrosting = computed(() => directRefreshApi.bifrosting.value);

    return {
      executeRefresh,
      bifrosting,
    };
  };

  const toggleActionHandler = (
    actionPrototypeView: ActionPrototypeView,
    actionId?: MaybeRefOrGetter<ActionId | undefined>,
  ) => {
    const removeApi = useApi(ctx);
    const addApi = useApi(ctx);
    const refreshApi = useApi(ctx);
    // Track which API is currently active as we could call 3 different APIs and need to propagate
    // the bifrosting status reactively for the correct one
    const activeApi = ref(addApi);

    const handleToggle = async () => {
      if (!component.value) return;
      const action = toValue(actionId);

      if (action) {
        activeApi.value = removeApi;
        // Removing/canceling existing action
        const call = removeApi.endpoint(routes.ActionCancel, {
          id: action,
        });
        removeApi.setWatchFn(() => toValue(actionId));
        await call.put({});
      } else {
        // Adding new action - check if it's a refresh action
        const onHead = ctx.onHead.value;
        const isRefresh = actionPrototypeView.kind === ActionKind.Refresh;
        if (onHead && isRefresh) {
          // For refresh actions on HEAD, we enqueue using another route until we can get rid of Force Change Set
          if (!component.value?.id || actionIsRunning(toValue(actionId)))
            return;
          activeApi.value = refreshApi;
          const call = refreshApi.endpoint(routes.RefreshAction, {
            componentId: component.value.id,
          });
          refreshApi.setWatchFn(() => toValue(actionId));
          await call.put({});
        } else {
          // Handle regular action add
          const call = addApi.endpoint(routes.ActionAdd);
          addApi.setWatchFn(() => toValue(actionId));
          const { req, newChangeSetId } = await call.post<{
            componentId: string;
            prototypeId: string;
          }>({
            componentId: component.value.id,
            prototypeId: actionPrototypeView.id,
          });
          if (newChangeSetId && addApi.ok(req)) {
            addApi.navigateToNewChangeSet(
              {
                name: "new-hotness-component",
                params: {
                  workspacePk: route.params.workspacePk,
                  changeSetId: newChangeSetId,
                  componentId: component.value.id,
                },
              },
              newChangeSetId,
            );
          }
        }
      }
    };

    const bifrosting = computed(() => activeApi.value.bifrosting ?? false);

    return {
      handleToggle,
      bifrosting,
    };
  };

  return {
    // Lists for the component
    actionPrototypeViews,
    actionByPrototype,

    // Whether an action is currently running
    actionIsRunning,

    // Specific refresh action properties
    refreshActionRunning,
    refreshEnabled,

    // Click Handlers for toggling actions and refreshing a resource
    toggleActionHandler,
    runRefreshHandler,
  };
};
