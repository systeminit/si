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
  AttributeValue,
  BifrostComponent,
  Prop,
} from "@/workers/types/entity_kind_types";
import QualificationView from "@/newhotness/QualificationView.vue";
import { AttributeValueId } from "@/store/status.store";
import { ValidationOutputStatus } from "@/api/sdf/dal/property_editor";
import { findAvsAtPropPath } from "./util";

export type QualificationStatus = "success" | "failure" | "warning";

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

const qualificationsWithoutValidations = computed<Qualification[]>(() => {
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

const convertValidationStatusToQualificationStatus = (
  status: ValidationOutputStatus,
): QualificationStatus => {
  switch (status) {
    case "Failure":
      return "failure";
    case "Error":
      return "failure";
    default:
      return "success";
  }
};

const qualifications = computed<Qualification[]>(() => {
  const results = [...qualificationsWithoutValidations.value];

  validations.value.forEach(({ attributeValue, prop }) => {
    if (attributeValue.validation && prop) {
      results.push({
        name: prop.name,
        message: attributeValue.validation.message || "",
        status: convertValidationStatusToQualificationStatus(
          attributeValue.validation.status,
        ),
      });
    }
  });

  return results;
});
</script>
