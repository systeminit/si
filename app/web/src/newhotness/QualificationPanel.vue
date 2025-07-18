<template>
  <ul class="p-xs flex flex-col gap-xs">
    <template v-if="attributeTree && qualifications.length > 0">
      <QualificationView
        v-for="qualification in qualifications"
        :key="qualification.avId"
        :qualification="qualification"
        :component="component.id"
      />
    </template>
    <EmptyState
      v-else
      icon="question-circle"
      text="No qualifications to display"
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
import EmptyState from "./EmptyState.vue";

export type QualificationStatus = "success" | "failure" | "warning" | "unknown";

export interface Qualification {
  name?: string;
  message?: string;
  status?: QualificationStatus;
  avId?: AttributeValueId;
  // This exists so the validation qualification can pass in its output, since that comes baked in the data
  // We should avoid using it for normal qualifications, for which QualificationView will lazily fetch the output
  output?: string[];
}

const props = defineProps<{
  component: BifrostComponent;
  attributeTree?: AttributeTree;
}>();

const root = computed(() => props.attributeTree);

const qualifications = computed<Qualification[]>(() => {
  const items: Qualification[] = [];
  const r = root.value;
  if (!r) return items;
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

  // Since we have all the data locally, we compute the validation rollup qualification over here
  // The qualification also gets computed in the backed for the old UI and luminork, so at some point we may
  // revisit this, but this works well.
  let hasValidations = false;
  const validationOutput: string[] = [];
  Object.values(r.attributeValues).forEach((av) => {
    const prop = r.props[av.propId ?? ""];
    if (!av.validation || !prop) return;
    hasValidations = true;

    // We believe that if we are connected to a subscription and that subscription
    // has yet to propagate a value, then it's a computed value and we should mark
    // the validation as passing for the user
    const pendingValue =
      av.externalSources &&
      av.externalSources?.length > 0 &&
      (av.value === "" || !av.value);
    if (pendingValue) return;

    const name = prop.name;

    if (av.validation.status === "Success") return;

    validationOutput.push(
      `${name}: ${av.validation.message ?? "unknown validation error"}`,
    );
  });

  if (hasValidations) {
    const status = validationOutput.length > 0 ? "failure" : "success";

    const message = `Component has ${validationOutput.length} invalid value(s).`;
    const output = validationOutput.length > 0 ? validationOutput : undefined;

    items.push({
      name: "Prop Validations",
      status,
      message,
      output,
    });
  }

  return items;
});
</script>
