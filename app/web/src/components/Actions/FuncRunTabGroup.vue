<template>
  <RightPanelDrawer :open="props.open">
    <TabGroup ref="tabGroupRef" @closeButtonTabClicked="props.close">
      <TabGroupCloseButton />
      <ChangesPanelHistorySubpanelTab label="Arguments" slug="arguments" :data="args" />
      <ChangesPanelHistorySubpanelTab label="Code Executed" slug="codeExecuted" :data="code" />
      <ChangesPanelHistorySubpanelTab
        label="Result"
        slug="resourceResult"
        emptyStateSecondaryTextNeedsAnA
        :data="result"
      />
      <ChangesPanelHistorySubpanelTab label="Logs" slug="logs" :data="logs" />
    </TabGroup>
  </RightPanelDrawer>
</template>

<script lang="ts" setup>
import { ref, computed, watch } from "vue";
import { TabGroup, TabGroupCloseButton } from "@si/vue-lib/design-system";
import { FuncRun } from "@/store/func_runs.store";
import ChangesPanelHistorySubpanelTab from "../ChangesPanelHistorySubpanelTab.vue";
import RightPanelDrawer from "../RightPanelDrawer.vue";

type clickFn = () => void;

const tabGroupRef = ref<InstanceType<typeof TabGroup>>();

const props = defineProps<{
  funcRun: FuncRun | undefined;
  close: clickFn;
  open: boolean;
  selectedTab: string | undefined;
}>();

watch(
  () => props.selectedTab,
  (newVal, _) => {
    tabGroupRef.value?.selectTab(newVal);
  },
);

const args = computed<string | undefined>(() => {
  if (props.funcRun) {
    return JSON.stringify(props.funcRun.functionArgs, null, "  ");
  } else {
    return undefined;
  }
});

const code = computed<string | undefined>(() => {
  if (props.funcRun) {
    return Buffer.from(props.funcRun.functionCodeBase64, "base64").toString();
  } else {
    return undefined;
  }
});

const result = computed<string | undefined>(() => {
  if (props.funcRun) {
    return JSON.stringify(props.funcRun.resultValue, null, "  ");
  } else {
    return undefined;
  }
});

const logs = computed<string | undefined>(() => {
  if (props.funcRun?.logs) {
    let log_string = `Start Time: ${props.funcRun.logs.createdAt}\n`;
    log_string += `Updated Time: ${props.funcRun.logs.updatedAt}\n`;
    log_string += `Finished?: ${props.funcRun.logs.finalized}\n\n`;
    for (const log of props.funcRun.logs.logs) {
      // TODO: This is only needed because we leak out internal response struct
      // we should fix it in lang-js once testing is reworked.
      if (/ayrtonsennajscommand/.test(log.message)) {
        continue;
      }
      log_string += `${log.message}\n`;
    }
    return log_string;
  } else {
    return undefined;
  }
});
</script>
