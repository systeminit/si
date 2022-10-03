<template>
  <SiPanel remember-size-key="workflow-left" side="left" :min-size="220">
    <FixPicker />
  </SiPanel>
  <div class="grow h-full relative bg-neutral-50 dark:bg-neutral-900">
    <GenericDiagram
      v-if="diagramData"
      :nodes="diagramData?.nodes"
      :edges="diagramData?.edges"
      read-only
    />
  </div>
  <SiPanel remember-size-key="workflow-right" side="right">
    <FixHistory />
  </SiPanel>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import _ from "lodash";
import SiPanel from "@/atoms/SiPanel.vue";
import DiagramService2 from "@/service/diagram2";
import { QualificationService } from "@/service/qualification";
import { ChangeSetService } from "@/service/change_set";
import FixPicker from "../FixPicker.vue";
import FixHistory from "../FixHistory.vue";
import { DiagramStatusIcon } from "../GenericDiagram/diagram_types";
import GenericDiagram from "../GenericDiagram/GenericDiagram.vue";

type QualificationStatus = "success" | "failure" | "running";
const qualificationStatusToIconMap: Record<
  QualificationStatus,
  DiagramStatusIcon
> = {
  success: { icon: "check", tone: "success" },
  failure: { icon: "alert", tone: "error" },
  running: { icon: "loading", tone: "info" },
};

const rawDiagramData = DiagramService2.useDiagramData();
const qualificationSummary = QualificationService.useQualificationSummary();

ChangeSetService.switchToHead();

const diagramData = computed(() => {
  return {
    ...rawDiagramData.value,
    nodes: _.map(rawDiagramData.value?.nodes, (node) => {
      // Default to "si" if we do not have a logo.
      let typeIcon = "si";
      if (
        node.category === "AWS" ||
        node.category === "CodeOS" ||
        node.category === "Docker" ||
        node.category === "Kubernetes"
      ) {
        typeIcon = node.category;
      }

      const componentQualificationSummary = _.find(
        qualificationSummary.value?.components,
        (cq) => cq.componentId.toString() === node.id,
      );
      let summaryStatus: QualificationStatus | undefined;
      if (componentQualificationSummary) {
        if (
          componentQualificationSummary.total >
          componentQualificationSummary.succeeded +
            componentQualificationSummary.failed
        )
          summaryStatus = "running";
        else if (componentQualificationSummary.failed > 0)
          summaryStatus = "failure";
        else summaryStatus = "success";
      }
      return {
        ...node,
        typeIcon,
        statusIcons: summaryStatus
          ? [qualificationStatusToIconMap[summaryStatus]]
          : [],
      };
    }),
  };
});
</script>
