<template>
  <!--
    NOTE: if you want to hide a diff, you almost certainly want to exclude it in
    shouldIncludeDiff() in Review.vue (or don't send it in the ComponentDiff MV!).
  -->
  <div
    :class="
      clsx(
        'flex flex-col gap-2xs px-sm py-xs border',
        themeClasses('border-neutral-400 bg-white', 'border-neutral-600 bg-neutral-800'),
      )
    "
  >
    <!-- Title and revert button-->
    <div class="flex flex-row items-center">
      <h1 class="h-10 py-xs mr-auto text-sm">
        {{ name }}
      </h1>
      <NewButton v-if="!disableRevert && !!revertToSource" size="xs" label="Revert" @click="revert" />
    </div>

    <template v-if="showDiffs">
      <!-- TODO use revertibleSource to determine revertibility (but right now revertibleSource seems not right!) -->
      <ReviewAttributeItemSourceAndValue v-if="diff?.old" :sourceAndValue="diff.old" old />
      <ReviewAttributeItemSourceAndValue v-if="diff?.new" :sourceAndValue="diff.new" />
    </template>

    <div v-if="isParent" class="flex flex-col gap-xs">
      <template v-for="(childItem, childName) in children" :key="childName">
        <ReviewAttributeItem
          :selectedComponentId="selectedComponentId"
          :name="childName"
          :item="childItem"
          :disableRevert="disableRevert"
        />
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { NewButton, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { computed, PropType } from "vue";
import * as _ from "lodash-es";
import { ComponentId } from "@/api/sdf/dal/component";
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
  disableRevert: { type: Boolean, required: true },
});

const path = computed(() => props.item.path);
const children = computed(() => props.item.children);
const diff = computed(() => props.item.diff);

/**
 * This is the thing you would send to the attributes API to revert this value.
 *
 * @return The { $source: ... } you would pass to the attributes API.
 *         - returns undefined if there is no way to reset the value (for example, the old value was set by an ancestor)
 *         - returns undefined if the reset would not actually change anything (for example, a subscription has not changed, but the upstream value has caused us to show a diff)
 */
const revertToSource = computed(() => {
  if (!diff.value) return undefined;

  // If you wouldn't show a diff, then you shouldn't revert.
  if (!showDiffs.value) return undefined;

  // If the sources are the same, the revert would be a noop
  const oldSource = diff.value.old?.$source;
  const newSource = diff.value.new?.$source;
  if (_.isEqual(oldSource, newSource)) return undefined;

  // If the new source is from an ancestor (such as an object/array subscription), you can't
  // directly revert it (you have to revert the parent).
  if (newSource?.fromAncestor && !newSource?.fromSchema) return undefined;

  // If the old source was explicitly set, return it as the source
  if (oldSource && !oldSource.fromSchema && !oldSource.fromAncestor) {
    return { $source: oldSource };
  }

  // The old source was explicitly set! But if the new one wasn't, this is a noop.
  if (!(newSource && !newSource.fromSchema && !newSource.fromAncestor)) return undefined;

  // Unset: The old source was *not* explicitly set by the user.
  return { $source: null };
});

const revert = async () => {
  if (!revertToSource.value) return;

  const call = saveApi.endpoint<{ success: boolean }>(routes.UpdateComponentAttributes, {
    id: props.selectedComponentId,
  });

  const payload = {
    [path.value]: revertToSource.value,
  };

  await call.put<componentTypes.UpdateComponentAttributesArgs>(payload);
};

const isParent = computed(() => props.item.children && Object.keys(props.item.children).length > 0);

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

    if (oldSource.component !== newSource.component || oldSource.path !== newSource.path) return true;
  }

  return false;
});
</script>
