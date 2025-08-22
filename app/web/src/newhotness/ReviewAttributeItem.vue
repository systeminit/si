<template>
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
    <!-- TODO maybe you want to see container values (i.e. when there are children) when the source is not a value -->
    <ReviewAttributeItemSourceAndValue
      v-if="diff?.new"
      :sourceAndValue="diff.new"
    />
    <!-- TODO use revertibleSource to determine revertibility (but right now revertibleSource seems not right!) -->
    <ReviewAttributeItemSourceAndValue
      v-if="diff?.old"
      :sourceAndValue="diff.old"
      old
      revertible
      @revert="revert"
    />

    <div v-if="children" class="flex flex-col gap-xs">
      <template v-for="(childItem, childName) in children" :key="childName">
        <ReviewAttributeItem
          :selectedComponentId="selectedComponentId"
          :name="childName"
          :item="childItem"
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

const saveApi = useApi();

const props = defineProps({
  selectedComponentId: {
    type: String as PropType<ComponentId>,
    required: true,
  },
  name: { type: String, required: true },
  item: { type: Object as PropType<AttributeDiffTree>, required: true },
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

  const { req, newChangeSetId } =
    await call.put<componentTypes.UpdateComponentAttributesArgs>(payload);
};
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
</script>
