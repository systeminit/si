<template>
  <div>
    <WidgetHeader
      :name="props.name"
      :path="props.path"
      :collapsed-paths="props.collapsedPaths"
      @toggle-collapsed="setCollapsed($event)"
    />
    <div
      v-show="isShown && !isCollapsed && !props.disabled"
      class="flex pl-8 pt-4 w-full"
    >
      <SiButton
        kind="standard"
        label="Add to array"
        icon="plus"
        @click="addToArray()"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { toRefs } from "vue";
import SiButton from "@/atoms/SiButton.vue";
import _ from "lodash";
import { usePropertyEditorIsShown } from "@/composables/usePropertyEditorIsShown";
import { AddToArray, PropertyPath } from "@/api/sdf/dal/property_editor";
import WidgetHeader from "./WidgetHeader.vue";

const props = defineProps<{
  name: string;
  path?: PropertyPath;
  collapsedPaths: Array<Array<string>>;
  disabled?: boolean;
  propId: number;
  valueId: number;
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
