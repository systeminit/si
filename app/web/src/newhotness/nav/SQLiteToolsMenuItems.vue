<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <DropdownMenuItem
    v-if="props.workspaceId && props.changeSetId"
    icon="niflheim"
    label="Re-do Cold Start"
    @click="heimdall.muspelheim(props.workspaceId, true)"
  />
  <DropdownMenuItem
    v-if="props.workspaceId && props.changeSetId"
    icon="refresh"
    label="Rebuild Index"
    @click="rebuild(props.workspaceId, props.changeSetId)"
  />
  <DropdownMenuItem icon="mjolnir" label="Throw Hammer" @click="() => emit('hammer')" />
  <DropdownMenuItem
    v-if="props.changeSetId"
    icon="odin"
    label="Log Sqlite"
    @click="() => props.changeSetId && heimdall.odin(props.changeSetId)"
  />
  <DropdownMenuItem icon="trash" label="Bobby Drop Tables" @click="() => heimdall.bobby()" />
  <DropdownMenuItem
    v-if="props.workspaceId && props.changeSetId"
    icon="trash"
    label="Ragnarok"
    @click="
      () => heimdall.ragnarok(props.workspaceId!, props.changeSetId!)
    "
  />
</template>

<script setup lang="ts">
import * as heimdall from "@/store/realtime/heimdall";
import { DropdownMenuItem } from '@si/vue-lib/design-system';
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { URLPattern, describePattern } from "@si/vue-lib";
import { sdfApiInstance } from "@/store/apis.web";


const props = defineProps<{
  changeSetId: string;
  workspaceId: string;
}>();

const emit = defineEmits<{
  (e: "hammer"): void;
}>();

const rebuild = (workspaceId: string, changeSetId: ChangeSetId) => {
  const pattern = [
    "v2",
    "workspaces",
    { workspaceId },
    "change-sets",
    { changeSetId },
    "index",
    "rebuild",
  ] as URLPattern;
  const [url] = describePattern(pattern);
  sdfApiInstance.post(url);
};
</script>