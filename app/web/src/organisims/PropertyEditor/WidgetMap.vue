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
      class="pl-8 flex flex-col w-full pt-4"
    >
      <div class="w-full pr-24">
        <SiTextBox :id="newKeyId" v-model="newKey" title="key" />
      </div>
      <div class="flex pt-4 pr-16">
        <SiButton
          kind="standard"
          label="Add to map"
          icon="plus"
          :disabled="submitDisabled"
          @click="addToMap()"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, toRefs, computed } from "vue";
import SiTextBox from "@/atoms/SiTextBox2.vue";
import SiButton from "@/atoms/SiButton.vue";
import _ from "lodash";
import { usePropertyEditorIsShown } from "@/composables/usePropertyEditorIsShown";
import { AddToMap, PropertyPath } from "@/api/sdf/dal/property_editor";
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
  (e: "addToMap", v: AddToMap): void;
}>();

const setCollapsed = (path: Array<string>) => {
  emits("toggle-collapsed", path);
};

const newKeyId = ref<string>(`newMap${props.valueId}`);
const newKey = ref<string>("");
const submitDisabled = computed(() => {
  return newKey.value == "";
});

const { name, path, collapsedPaths } = toRefs(props);
const { isShown, isCollapsed } = usePropertyEditorIsShown(
  name,
  collapsedPaths,
  path,
  true,
);

const addToMap = () => {
  emits("addToMap", {
    propId: props.propId,
    valueId: props.valueId,
    key: newKey.value,
  });
  newKey.value = "";
};
</script>
