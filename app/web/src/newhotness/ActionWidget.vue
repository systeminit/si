<template>
  <div
    class="cursor-pointer"
    :class="
      clsx(
        'flex flex-row items-center gap-xs p-2xs border-x border-b',
        themeClasses('border-neutral-400', 'border-neutral-600'),
      )
    "
    @click="clickHandler"
  >
    <Toggle :selected="!!props.actionId" class="flex-none" />
    <StatusIndicatorIcon
      type="action"
      :status="actionPrototypeView.kind"
      tone="inherit"
      class="flex-none"
    />
    <div class="font-bold leading-normal">
      {{ actionPrototypeView.displayName || actionPrototypeView.name }}
    </div>

    <Icon
      v-if="removeApi.bifrosting.value || addApi.bifrosting.value"
      name="loader"
      class="ml-auto"
      size="sm"
    />

    <VButton
      v-if="actionPrototypeView.kind === ActionKind.Refresh"
      tooltip="REFRESH"
      tooltipPlacement="top"
      icon="refresh"
      iconTone="action"
      size="sm"
      tone="neutral"
      label="REFRESH RIGHT NOW"
      @click.stop="runRefresh"
    />
  </div>
</template>

<script setup lang="ts">
import * as _ from "lodash-es";
import clsx from "clsx";
import { Icon, VButton, themeClasses, Toggle } from "@si/vue-lib/design-system";
import { useRoute } from "vue-router";
import { inject } from "vue";
import StatusIndicatorIcon from "@/components/StatusIndicatorIcon.vue";
import { ActionId, ActionKind } from "@/api/sdf/dal/action";
import {
  ActionPrototypeView,
  BifrostComponent,
} from "@/workers/types/entity_kind_types";
import { routes, useApi } from "./api_composables";
import { assertIsDefined, Context, isOnHead } from "./types";

const props = defineProps<{
  component: BifrostComponent;
  actionPrototypeView: ActionPrototypeView;
  actionId?: ActionId;
}>();

const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);

const route = useRoute();
const removeApi = useApi();
const addApi = useApi();
const refreshApi = useApi();
const runRefresh = async () => {
  const call = refreshApi.endpoint(routes.RefreshAction, {
    componentId: props.component.id,
  });
  refreshApi.setWatchFn(() => props.actionId);
  await call.put({});
};
const clickHandler = async () => {
  if (props.actionId) {
    const call = removeApi.endpoint(routes.ActionCancel, {
      id: props.actionId,
    });
    removeApi.setWatchFn(() => props.actionId);

    // This route can mutate head, so we do not need to handle new change set semantics.
    await call.put({});
  } else {
    // Need to special case Refresh funcs until we can rid the world of ForceChangeSet
    if (
      isOnHead(ctx) &&
      props.actionPrototypeView.kind === ActionKind.Refresh
    ) {
      await runRefresh();
    } else {
      const call = addApi.endpoint(routes.ActionAdd);
      addApi.setWatchFn(() => props.actionId);
      const { req, newChangeSetId } = await call.post<{
        componentId: string;
        prototypeId: string;
      }>({
        componentId: props.component.id,
        prototypeId: props.actionPrototypeView.id,
      });
      if (newChangeSetId && addApi.ok(req)) {
        addApi.navigateToNewChangeSet(
          {
            name: "new-hotness-component",
            params: {
              workspacePk: route.params.workspacePk,
              changeSetId: newChangeSetId,
              componentId: props.component.id,
            },
          },
          newChangeSetId,
        );
      }
    }
  }
};
</script>
