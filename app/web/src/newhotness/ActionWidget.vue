<template>
  <div
    class="cursor-pointer"
    :class="
      clsx(
        'flex flex-row items-center gap-xs p-2xs border-x border-b',
        themeClasses('border-neutral-200', 'border-neutral-600'),
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
  </div>
</template>

<script setup lang="ts">
import * as _ from "lodash-es";
import clsx from "clsx";
import { Icon, themeClasses, Toggle } from "@si/vue-lib/design-system";
import { useRoute, useRouter } from "vue-router";
import StatusIndicatorIcon from "@/components/StatusIndicatorIcon.vue";
import { ActionId } from "@/api/sdf/dal/action";
import {
  ActionPrototypeView,
  BifrostComponent,
} from "@/workers/types/entity_kind_types";
import { routes, useApi } from "./api_composables";

const props = defineProps<{
  component: BifrostComponent;
  actionPrototypeView: ActionPrototypeView;
  actionId?: ActionId;
}>();

const router = useRouter();
const route = useRoute();
const removeApi = useApi();
const addApi = useApi();
removeApi.setWatchFn(() => props.actionId);
addApi.setWatchFn(() => props.actionId);
const clickHandler = async () => {
  if (props.actionId) {
    const call = removeApi.endpoint(routes.ActionCancel, {
      id: props.actionId,
    });
    await call.put({});
  } else {
    const call = addApi.endpoint(routes.ActionAdd);
    const { req, newChangeSetId } = await call.post<{
      componentId: string;
      prototypeId: string;
    }>({
      componentId: props.component.id,
      prototypeId: props.actionPrototypeView.id,
    });
    if (newChangeSetId && addApi.ok(req)) {
      router.push({
        name: "new-hotness-component",
        params: {
          workspacePk: route.params.workspacePk,
          changeSetId: newChangeSetId,
          componentId: props.component.id,
        },
      });
    }
  }
};
</script>
