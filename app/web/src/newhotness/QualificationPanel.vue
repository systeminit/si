<template>
  <ul class="p-xs flex flex-col gap-xs">
    <template v-if="attributeTree && qualifications.length > 0">
      <QualificationView
        v-for="qualification in qualifications"
        :key="qualification.avId ?? undefined"
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
  DependentValues,
} from "@/workers/types/entity_kind_types";
import QualificationView from "@/newhotness/QualificationView.vue";
import { AttributePath } from "@/api/sdf/dal/component";
import { AttributeValueId } from "./types";
import { findAvsAtPropPath } from "./util";
import EmptyState from "./EmptyState.vue";

export type QualificationStatus = "success" | "failure" | "warning" | "unknown";

export interface Qualification {
  name?: null | string;
  message?: null | string;
  status?: null | QualificationStatus;
  avId?: null | AttributeValueId;
  // This exists so the validation qualification can pass in its output, since that comes baked in the data
  // We should avoid using it for normal qualifications, for which QualificationView will lazily fetch the output
  output?: null | string[];
  /** Indicates whether the qualification is dirty (running or waiting to run) */
  isDirty?: boolean;
}

const props = defineProps<{
  component: BifrostComponent;
  attributeTree?: AttributeTree | null;
  dependentValues?: DependentValues | null;
}>();

function attributeIsDirty(path: AttributePath) {
  const root = props.attributeTree;
  if (!root) return false;
  return (
    props.dependentValues?.componentAttributes[root.id]?.includes(path) ?? false
  );
}

/// Actual qualification results (excludes validations)
const componentQualifications = computed<Qualification[]>(() => {
  const root = props.attributeTree;
  if (!root) return [];

  // Get the actual qualification results from the tree
  const qualificationItems = findAvsAtPropPath(props.attributeTree, [
    "root",
    "qualification",
    "qualificationItem",
  ]);
  if (!qualificationItems) return [];

  return qualificationItems.attributeValues.map((av) => {
    const qualification: Qualification = {
      avId: av.id,
      name: av.key,
      isDirty: attributeIsDirty(av.path),
    };

    // Set result and message based on the AttributeTree
    for (const avId of root.treeInfo[av.id]?.children ?? []) {
      const child = root.attributeValues[avId];
      // TODO should we set both if they both exist?
      if (child?.path?.endsWith("result"))
        qualification.status = child.value as QualificationStatus;
      else if (child?.path?.endsWith("message"))
        qualification.message = child.value as string;
    }

    return qualification;
  });
});

/// A qualification representing the validation status of all attribute values
const validationsQualification = computed<Qualification | undefined>(() => {
  const root = props.attributeTree;
  if (!root) return undefined;

  // Since we have all the data locally, we compute the validation rollup qualification over here
  // The qualification also gets computed in the backed for the old UI and luminork, so at some point we may
  // revisit this, but this works well.
  let hasValidations = false;
  let isDirty = false;
  const validationOutput: string[] = [];
  Object.values(root.attributeValues).forEach((av) => {
    const prop = root.props[av.propId ?? ""];
    if (!av.validation || !prop) return;
    hasValidations = true;

    // If any of the values are dirty, mark the qualifications as dirty too
    isDirty ||= attributeIsDirty(av.path);

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

  if (!hasValidations) return undefined;

  const status = validationOutput.length > 0 ? "failure" : "success";
  const message = `Component has ${validationOutput.length} invalid value(s).`;
  const output = validationOutput.length > 0 ? validationOutput : undefined;

  return {
    name: "Prop Validations",
    status,
    message,
    output,
    isDirty,
  };
});

const qualifications = computed<Qualification[]>(() => {
  // Grab the component qualifications
  let items = componentQualifications.value;

  // If there are validations, show them as a qualification as well
  if (validationsQualification.value) {
    items = [...items, validationsQualification.value];
  }

  // Sort qualifications with failed first, then warning, then success, then unknown
  return items.sort((a, b) => {
    const statusOrder = { failure: 0, warning: 1, success: 2, unknown: 3 };
    const aOrder = statusOrder[a.status as keyof typeof statusOrder] ?? 4;
    const bOrder = statusOrder[b.status as keyof typeof statusOrder] ?? 4;
    return aOrder - bOrder;
  });
});
</script>
