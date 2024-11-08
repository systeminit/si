<template>
  <div>
    <DropdownMenu ref="contextMenuRef" :forceAbove="false" forceAlignRight>
      <DropdownMenuItem
        :disabled="!funcRunId"
        :onSelect="
          () => {
            emit('menuClick', funcRunId ?? '', 'arguments');
          }
        "
        label="Arguments"
      />
      <DropdownMenuItem
        :disabled="!funcRunId"
        :onSelect="
          () => {
            emit('menuClick', funcRunId ?? '', 'codeExecuted');
          }
        "
        label="Code Executed"
      />
      <DropdownMenuItem
        :disabled="!funcRunId"
        :onSelect="
          () => {
            emit('menuClick', funcRunId ?? '', 'resourceResult');
          }
        "
        label="Resource Result"
      />
      <DropdownMenuItem
        :disabled="!funcRunId"
        :onSelect="
          () => {
            emit('menuClick', funcRunId ?? '', 'logs');
          }
        "
        label="Logs"
      />
      <DropdownMenuItem
        v-if="showFuncView"
        :onSelect="
          () => {
            emit('viewFunc');
          }
        "
        label="View Function"
      />
    </DropdownMenu>

    <DetailsPanelMenuIcon
      :selected="contextMenuRef?.isOpen"
      @click="
        (e) => {
          contextMenuRef?.open(e, false);
        }
      "
    />
  </div>
</template>

<script lang="ts" setup>
import { ref } from "vue";
import { DropdownMenu, DropdownMenuItem } from "@si/vue-lib/design-system";
import { FuncRunId } from "@/store/func_runs.store";
import DetailsPanelMenuIcon from "./DetailsPanelMenuIcon.vue";

const contextMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const props = defineProps<{ funcRunId?: FuncRunId; showFuncView?: boolean }>();

const emit = defineEmits<{
  (e: "menuClick", funcRunId: FuncRunId, tabSlug: string): void;
  (e: "viewFunc"): void;
}>();
</script>
