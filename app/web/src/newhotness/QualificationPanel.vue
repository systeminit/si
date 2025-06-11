<template>
  <ul class="p-xs flex flex-col gap-xs">
    <QualificationView
      v-for="item in items"
      :key="item.avId"
      :qualification="item"
    />
  </ul>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import {
  AttributeTree,
  AttributeValue,
  BifrostComponent,
  Prop,
} from "@/workers/types/entity_kind_types";
import QualificationView from "@/newhotness/QualificationView.vue";
import { AttributeValueId } from "@/store/status.store";
import { QualificationStatus } from "@/store/qualifications.store";
import { findAvsAtPropPath } from "./util";

export interface QualItem {
  name?: string;
  message?: string;
  result?: QualificationStatus;
  avId?: AttributeValueId;
}

const props = defineProps<{
  component: BifrostComponent;
  attributeTree?: AttributeTree;
}>();

const root = computed(() => props.attributeTree);

const qualItems = computed<QualItem[]>(() => {
  const items: QualItem[] = [];
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
    let result;
    let message;
    children.forEach((avId) => {
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const child = r.attributeValues[avId]!;
      if (child.path?.endsWith("result")) result = child.value;
      else if (child.path?.endsWith("message")) message = child.value;
    });
    items.push({
      avId: av.id,
      name,
      result,
      message,
    });
  });
  return items;
});

const validations = computed(() => {
  const avsWithValidation: {
    prop?: Prop;
    attributeValue: AttributeValue;
  }[] = [];
  if (!root.value) return avsWithValidation;
  Object.values(root.value.attributeValues).forEach((attributeValue) => {
    if (attributeValue.validation !== null) {
      const prop = root.value?.props[attributeValue.propId ?? ""];
      avsWithValidation.push({ attributeValue, prop });
    }
  });
  return avsWithValidation;
});

// TODO(Wendy) - this is very annoying!
// Why are statuses sometimes capitalized and sometimes not?!?!?!
const fixStatus = (status: string): QualificationStatus => {
  switch (status) {
    case "Success":
      return "success";
    case "Failure":
      return "failure";
    case "Error":
      return "failure";
    default:
      return "running";
  }
};

const items = computed<QualItem[]>(() => {
  const items = [...qualItems.value];

  validations.value.forEach(({ attributeValue, prop }) => {
    if (attributeValue.validation && prop) {
      items.push({
        name: prop.name,
        message: attributeValue.validation.message || "",
        result: fixStatus(attributeValue.validation.status),
      });
    }
  });

  return items;
});
</script>
