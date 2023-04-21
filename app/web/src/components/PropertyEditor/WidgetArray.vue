<template>
  <div>
    <WidgetHeader
      :collapsed-paths="props.collapsedPaths"
      :name="props.name"
      :path="props.path"
      @toggle-collapsed="setCollapsed($event)"
    />
    <div
      v-show="isShown && !isCollapsed && !props.disabled"
      class="flex pl-8 pt-4 w-full"
    >
      <VButton
        icon="plus"
        size="xs"
        label="Add to array"
        tone="neutral"
        @click="addToArray()"
      />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { toRefs } from "vue";
import { VButton } from "@si/vue-lib/design-system";
import { usePropertyEditorIsShown } from "@/utils/usePropertyEditorIsShown";
import { AddToArray, PropertyPath } from "@/api/sdf/dal/property_editor";
import WidgetHeader from "./WidgetHeader.vue";

const props = defineProps<{
  name: string;
  path?: PropertyPath;
  collapsedPaths: Array<Array<string>>;
  disabled?: boolean;
  propId: string;
  valueId: string;
  arrayLength?: number;
}>();
const emits = defineEmits<{
  (e: "toggle-collapsed", path: Array<string>): void;
  (e: "addToArray", v: AddToArray): void;
}>();

const setCollapsed = (path: Array<string>) => {
  emits("toggle-collapsed", path);
};

const { name, path, collapsedPaths } = toRefs(props);
const { isShown, isCollapsed } = usePropertyEditorIsShown(
  name,
  collapsedPaths,
  path,
  true,
);

const addToArray = () => {
  emits("addToArray", {
    propId: props.propId,
    valueId: props.valueId,
  });
};
</script>
