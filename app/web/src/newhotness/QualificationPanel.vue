<template>
  <ul class="p-xs flex flex-col gap-xs">
    <QualificationView
      v-for="qualification in qualifications"
      :key="qualification.avId"
      :qualification="qualification"
    />
  </ul>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import {
  AttributeTree,
  BifrostComponent,
} from "@/workers/types/entity_kind_types";
import QualificationView from "@/newhotness/QualificationView.vue";
import { AttributeValueId } from "@/store/status.store";
import { findAvsAtPropPath } from "./util";

export type QualificationStatus = "success" | "failure" | "warning" | "unknown";

export interface Qualification {
  name?: string;
  message?: string;
  status?: QualificationStatus;
  avId?: AttributeValueId;
}

const props = defineProps<{
  component: BifrostComponent;
  attributeTree?: AttributeTree;
}>();

const root = computed(() => props.attributeTree);

const qualifications = computed<Qualification[]>(() => {
  const items: Qualification[] = [];
  if (!root.value) return items;
  const r = root.value;
  const data = findAvsAtPropPath(r, [
    "root",
    "qualification",
    "qualificationItem",
  ]);
  if (!data) return items;
  const { attributeValues } = data;
  attributeValues.forEach((av) => {
    const name = av.key;
    const children = r.treeInfo[av.id]?.children ?? [];
    let status;
    let message;
    children.forEach((avId) => {
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const child = r.attributeValues[avId]!;
      if (child.path?.endsWith("result")) status = child.value;
      else if (child.path?.endsWith("message")) message = child.value;
    });
    items.push({
      avId: av.id,
      name,
      status,
      message,
    });
  });
  return items;
});
</script>
