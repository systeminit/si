<template>
  <!--
    NOTE: if you want to hide a diff, you almost certainly want to exclude it in
    shouldIncludeDiff() in Review.vue (or don't send it in the ComponentDiff MV!).
  -->
  <div
    :class="
      clsx(
        'flex flex-col gap-xs p-xs border',
        themeClasses(
          'border-neutral-400 bg-white',
          'border-neutral-600 bg-neutral-800',
        ),
      )
    "
  >
    <h1 class="h-10 py-xs">{{ name }}</h1>

    <template v-if="showDiffs">
      <ReviewAttributeItemSourceAndValue
        v-if="diff?.new"
        :sourceAndValue="diff.new"
        :secret="secret"
      />
      <!-- TODO use revertibleSource to determine revertibility (but right now revertibleSource seems not right!) -->
      <ReviewAttributeItemSourceAndValue
        v-if="diff?.old"
        :sourceAndValue="diff.old"
        old
        :revertible="!disableRevert"
        :secret="secret"
        @revert="revert"
      />
    </template>

    <div v-if="isParent" class="flex flex-col gap-xs">
      <template v-for="(childItem, childName) in children" :key="childName">
        <ReviewAttributeItem
          :selectedComponentId="selectedComponentId"
          :name="childName"
          :item="childItem"
          :disableRevert="disableRevertChildren"
        />
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { computed, PropType } from "vue";
import { ComponentId } from "@/api/sdf/dal/component";
import { AttributeSource } from "@/store/components.store";
import ReviewAttributeItemSourceAndValue from "./ReviewAttributeItemSourceAndValue.vue";
import { useApi, routes, componentTypes } from "./api_composables";
import { AttributeDiffTree } from "./Review.vue";
import {
  AttributeSourceLocation,
  SimplifiedAttributeSource,
} from "../workers/types/entity_kind_types";

const saveApi = useApi();

const props = defineProps({
  selectedComponentId: {
    type: String as PropType<ComponentId>,
    required: true,
  },
  name: { type: String, required: true },
  item: { type: Object as PropType<AttributeDiffTree>, required: true },
  secret: { type: Boolean },
  disableRevert: { type: Boolean },
});

const path = computed(() => props.item.path);
const children = computed(() => props.item.children);
const diff = computed(() => props.item.diff);

const revertibleSource = computed(() => {
  if (!diff.value?.old) return undefined;
  const { $source } = diff.value.old;
  if ($source.fromSchema || $source.fromAncestor) return undefined;
  if ($source.prototype) return undefined;
  return { $source } as AttributeSource;
});

const revert = async () => {
  if (!revertibleSource.value) return;

  const call = saveApi.endpoint<{ success: boolean }>(
    routes.UpdateComponentAttributes,
    { id: props.selectedComponentId },
  );

  const payload = {
    [path.value]: revertibleSource.value,
  };

  await call.put<componentTypes.UpdateComponentAttributesArgs>(payload);
};

const isParent = computed(
  () => props.item.children && Object.keys(props.item.children).length > 0,
);

const showDiffs = computed(() => {
  if (!isParent.value) {
    return true;
  }

  if (props.item.diff?.new?.$source) {
    if (!props.item.diff.old?.$source) {
      if ("value" in props.item.diff.new.$source) {
        return false;
      } else {
        return true;
      }
    }

    const oldSource = props.item.diff.old.$source;
    const newSource = props.item.diff.new.$source;

    if (
      oldSource.component !== newSource.component ||
      oldSource.path !== newSource.path
    )
      return true;
  }

  return false;
});

const disableRevertChildren = computed(
  () =>
    !!(
      props.disableRevert ||
      (props.item.diff?.new &&
        sourceAndValueDisplayKind(props.item.diff.new.$source, props.secret) ===
          "subscription")
    ),
);
</script>

<script lang="ts">
export const trimPath = (rawPath: string) => {
  if (rawPath.startsWith("/domain/")) {
    return rawPath.slice(8);
  } else if (rawPath.startsWith("/si/")) {
    return rawPath.slice(4);
  } else if (rawPath.startsWith("/")) {
    return rawPath.slice(1);
  } else {
    return rawPath;
  }
};
export const sourceAndValueDisplayKind = (
  source: AttributeSourceLocation & SimplifiedAttributeSource,
  secret = false,
) => {
  if ("component" in source) return "subscription";
  else if (secret) return "secret";
  else if ("value" in source) return "value";
  // TODO(Wendy) - complex AV diffs?
  return "hidden";
};
</script>
