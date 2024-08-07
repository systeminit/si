<template>
  <ScrollArea
    class="flex flex-col h-full border border-t-0 border-neutral-300 dark:border-neutral-600"
  >
    <template #top>
      <div class="flex flex-col py-xs">
        <div class="flex flex-row items-center gap-xs text-2xl font-bold px-xs">
          <Icon name="component" size="xl" class="flex-none" />
          <TruncateWithTooltip class="flex-grow">
            {{ $props.moduleName }}
          </TruncateWithTooltip>
          <template v-if="functions.length > 0">
            <IconButton
              class="flex-none"
              icon="dots-vertical"
              variant="classic"
              noBorderOnHover
              iconIdleTone="neutral"
              :selected="codeMenuRef?.isOpen"
              @click="codeMenuRef?.open"
            />
            <DropdownMenu ref="codeMenuRef">
              <DropdownMenuItem
                :label="assetFn.name"
                @click="jumpTo(assetFuncCodeRef)"
              />
              <DropdownMenuItem
                v-for="(func, index) in functions"
                :key="index"
                :label="func.name"
                @click="jumpTo(funcCodeRefs[index])"
              />
            </DropdownMenu>
          </template>
          <VButton
            class="flex-none"
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
          class="px-xs text-xs italic flex flex-row flex-wrap gap-x-lg text-neutral-600 dark:text-neutral-200"
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
      </div>
    </template>

    <div
      v-if="loadModuleReqStatus.isError || !loadModuleReqStatus.isRequested"
      class="p-xs text-center text-neutral-400 dark:text-neutral-300"
    >
      Unable to retrieve details for
      {{ moduleId ? `"${moduleId}"` : "unknown module" }}
    </div>
    <template v-else-if="moduleObj">
      <TreeNode
        v-if="functions.length > 0"
        ref="assetFuncCodeRef"
        alwaysShowArrow
        enableGroupToggle
        :defaultOpen="false"
        noIndentationOrLeftBorder
        enableDefaultHoverClasses
        :label="assetFn.name"
      >
        <CodeViewer :code="assetFn.code" disableScroll />
      </TreeNode>
      <CodeViewer v-else :code="assetFn.code" disableScroll />
      <TreeNode
        v-for="func in functions"
        ref="funcCodeRefs"
        :key="func.id"
        alwaysShowArrow
        enableGroupToggle
        :defaultOpen="false"
        noIndentationOrLeftBorder
        enableDefaultHoverClasses
        :label="func.name"
      >
        <CodeViewer :code="func.code" disableScroll />
      </TreeNode>
    </template>
    <RequestStatusMessage
      v-else
      :requestStatus="loadModuleReqStatus"
      loadingMessage="Retrieving Module"
    />
  </ScrollArea>
</template>

<script lang="ts" setup>
import { watch, computed, ref } from "vue";
import * as _ from "lodash-es";
import {
  RequestStatusMessage,
  ScrollArea,
  Timestamp,
  VButton,
  Icon,
  TreeNode,
  DropdownMenu,
  DropdownMenuItem,
} from "@si/vue-lib/design-system";
import { useModuleStore, ModuleSpec } from "@/store/module.store";
import { nilId } from "@/utils/nilId";
import { ModuleId } from "@/api/sdf/dal/module";
import TruncateWithTooltip from "./TruncateWithTooltip.vue";
import CodeViewer from "./CodeViewer.vue";
import IconButton from "./IconButton.vue";

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

const install = () => moduleStore.INSTALL_REMOTE_MODULE([props.moduleId]);

watch(
  () => props.moduleId,
  () => {
    moduleStore.GET_REMOTE_MODULE_SPEC(props.moduleId);
  },
  { immediate: true },
);

const codeMenuRef = ref<InstanceType<typeof DropdownMenu>>();
const assetFuncCodeRef = ref<InstanceType<typeof TreeNode>>();
const funcCodeRefs = ref<InstanceType<typeof TreeNode>[]>([]);

const jumpTo = (code: InstanceType<typeof TreeNode> | undefined) => {
  if (code) {
    if (code !== assetFuncCodeRef.value) {
      assetFuncCodeRef.value?.toggleIsOpen(false);
    }
    funcCodeRefs.value.forEach((t) => {
      t.toggleIsOpen(false);
    });
    code.toggleIsOpen(true);
    const el = code.$el as Element;
    el.scrollIntoView({ block: "nearest", inline: "nearest" });
  }
};
</script>
