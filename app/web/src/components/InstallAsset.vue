<template>
  <ScrollArea
    class="flex flex-col h-full border border-t-0 border-neutral-300 dark:border-neutral-600"
  >
    <template #top>
      <div class="p-xs">
        <TruncateWithTooltip
          class="text-2xl font-bold pb-2xs flex flex-row items-center gap-xs"
        >
          <Icon name="component" size="xl" />
          <div class="flex flex-row items-center gap-xs">
            {{ $props.moduleName }}
          </div>
        </TruncateWithTooltip>
        <VButton
          icon="component-plus"
          label="Install Module"
          :loading="installReqStatus.isPending"
          loadingText="Installing..."
          :requestStatus="installReqStatus"
          successText="Successfully Installed"
          tone="action"
          @click="install"
        />
      </div>
      <div
        v-if="moduleObj"
        class="text-xs italic flex flex-row flex-wrap gap-x-lg text-neutral-600 dark:text-neutral-200"
      >
        <div>
          <span class="font-bold">Name: </span>
          {{ moduleObj.name }}
        </div>
        <div>
          <span class="font-bold">Module Created At: </span>
          <Timestamp :date="moduleObj.createdAt" size="long" />
        </div>
        <div>
          <span class="font-bold">Created By: </span>{{ moduleObj.createdBy }}
        </div>
        <div>
          <span class="font-bold">Version: </span>{{ moduleObj.version }}
        </div>
      </div>
    </template>

    <div
      v-if="loadModuleReqStatus.isError || !loadModuleReqStatus.isRequested"
      class="p-2 text-center text-neutral-400 dark:text-neutral-300"
    >
      <template v-if="moduleId">
        Cannot retrieve details for "{{ moduleId }}"
      </template>
    </div>
    <template v-else-if="moduleObj">
      <CodeEditor
        :id="`module-asset-${moduleId}`"
        v-model="assetFn.code"
        disabled
        :recordId="moduleId"
      />
      <template v-for="func in functions" :key="func.id">
        <h2 class="text-xl">{{ func.name }}</h2>
        <CodeEditor
          :id="`module-${func.name}-${moduleId}`"
          v-model="func.code"
          disabled
          :recordId="moduleId"
        />
      </template>
    </template>
    <RequestStatusMessage
      v-else
      :requestStatus="loadModuleReqStatus"
      loadingMessage="Retrieving Module"
    />
  </ScrollArea>
</template>

<script lang="ts" setup>
import { watch, computed } from "vue";
import * as _ from "lodash-es";
import {
  RequestStatusMessage,
  ScrollArea,
  Timestamp,
  VButton,
  Icon,
} from "@si/vue-lib/design-system";
import { useModuleStore, ModuleSpec } from "@/store/module.store";
import { nilId } from "@/utils/nilId";
import { ModuleId } from "@/api/sdf/dal/module";
import TruncateWithTooltip from "./TruncateWithTooltip.vue";
import CodeEditor from "./CodeEditor.vue";

const props = defineProps<{
  moduleId: ModuleId;
  moduleName: string;
}>();

const moduleStore = useModuleStore();
const loadModuleReqStatus = moduleStore.getRequestStatus(
  "GET_REMOTE_MODULE_SPEC",
  props.moduleId,
);
const installReqStatus = moduleStore.getRequestStatus(
  "INSTALL_REMOTE_MODULE",
  props.moduleId,
);
const moduleObj = computed<ModuleSpec | undefined>(
  () => moduleStore.remoteModuleSpecsById[props.moduleId],
);

interface FuncDisplay {
  id: string;
  name: string;
  code: string;
}
const defaultAssetFn = {
  id: nilId(),
  name: "",
  code: "",
};
const assetFn = computed<FuncDisplay>(() => {
  if (!moduleObj.value) return defaultAssetFn;
  const f = moduleObj.value.funcs
    .filter((f) => !f.name.startsWith("si:"))
    .find((f) => f.data.backendKind === "jsSchemaVariantDefinition");
  if (!f) {
    return defaultAssetFn;
  } else {
    return {
      id: f.uniqueId,
      name: f.name,
      code: Buffer.from(f.data.codeBase64 || "", "base64").toString(),
    };
  }
});
const functions = computed<FuncDisplay[]>(() => {
  if (!moduleObj.value) return [];
  return moduleObj.value.funcs
    .filter((f) => !f.name.startsWith("si:"))
    .filter((f) => f.data.backendKind !== "jsSchemaVariantDefinition")
    .map((f) => {
      const code = Buffer.from(f.data.codeBase64 || "", "base64").toString();
      const display = {} as FuncDisplay;
      display.id = f.uniqueId;
      display.name = f.name;
      display.code = code;
      return display;
    });
});

const install = () => moduleStore.INSTALL_REMOTE_MODULE(props.moduleId);

watch(
  () => props.moduleId,
  () => {
    moduleStore.GET_REMOTE_MODULE_SPEC(props.moduleId);
  },
  { immediate: true },
);
</script>
