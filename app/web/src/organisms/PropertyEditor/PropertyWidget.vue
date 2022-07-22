<template>
  <div class="flex flex-col" @keyup.stop @keydown.stop>
    <WidgetHeader
      v-if="showArrayElementHeader"
      :name="props.schemaProp.name"
      :path="path"
      :collapsed-paths="props.collapsedPaths"
      @toggle-collapsed="setCollapsed($event)"
    />
    <WidgetHeader
      v-if="
        props.schemaProp.widgetKind.kind === 'header' && !showArrayElementHeader
      "
      :name="props.schemaProp.name"
      :path="path"
      :collapsed-paths="props.collapsedPaths"
      @toggle-collapsed="setCollapsed($event)"
    />
    <WidgetArray
      v-else-if="props.schemaProp.widgetKind.kind === 'array'"
      :name="props.schemaProp.name"
      :path="path"
      :collapsed-paths="props.collapsedPaths"
      :disabled="props.disabled"
      :prop-id="props.propValue.propId"
      :value-id="props.propValue.id"
      :array-length="props.arrayLength"
      @add-to-array="addToArray($event)"
      @toggle-collapsed="setCollapsed($event)"
    />
    <WidgetMap
      v-else-if="props.schemaProp.widgetKind.kind === 'map'"
      :name="props.schemaProp.name"
      :path="path"
      :collapsed-paths="props.collapsedPaths"
      :disabled="props.disabled"
      :prop-id="props.propValue.propId"
      :value-id="props.propValue.id"
      :array-length="props.arrayLength"
      @add-to-map="addToMap($event)"
      @toggle-collapsed="setCollapsed($event)"
    />
    <WidgetTextBox
      v-else-if="props.schemaProp.widgetKind.kind === 'text'"
      :name="props.schemaProp.name"
      :path="path"
      :collapsed-paths="props.collapsedPaths"
      :value="props.propValue.value"
      :prop-id="props.propValue.propId"
      :value-id="props.propValue.id"
      :prop-kind="props.schemaProp.kind"
      :doc-link="props.schemaProp.docLink"
      :validation="props.validation"
      :disabled="props.disabled"
      class="py-4 px-8"
      @updated-property="updatedProperty($event)"
    />
    <WidgetCheckBox
      v-else-if="props.schemaProp.widgetKind.kind == 'checkBox'"
      :name="props.schemaProp.name"
      :path="path"
      :collapsed-paths="props.collapsedPaths"
      :value="props.propValue.value"
      :prop-id="props.propValue.propId"
      :value-id="props.propValue.id"
      :doc-link="props.schemaProp.docLink"
      :validation="props.validation"
      :disabled="props.disabled"
      class="py-4 px-8"
      @updated-property="updatedProperty($event)"
    />
    <WidgetSelectBox
      v-else-if="props.schemaProp.widgetKind.kind == 'select'"
      :name="props.schemaProp.name"
      :options="props.schemaProp.widgetKind.options"
      :path="path"
      :collapsed-paths="props.collapsedPaths"
      :value="props.propValue.value"
      :prop-id="props.propValue.propId"
      :value-id="props.propValue.id"
      :doc-link="props.schemaProp.docLink"
      :validation="props.validation"
      :disabled="props.disabled"
      class="py-4 px-8"
      @updated-property="updatedProperty($event)"
    />
    <!--<div v-else>
      <div class="flex">
        {{ props.path }}
      </div>
      <div class="flex">
        {{ props.schemaProp }}
      </div>
      <div class="flex">
        {{ props.propValue }}
      </div>
    </div>
      -->
  </div>
</template>

<script setup lang="ts">
import { toRefs, computed } from "vue";
import {
  PropertyEditorProp,
  PropertyEditorValidation,
  PropertyEditorValue,
  UpdatedProperty,
  AddToArray,
  AddToMap,
  PropertyPath,
} from "@/api/sdf/dal/property_editor";
import WidgetHeader from "./WidgetHeader.vue";
import WidgetTextBox from "./WidgetTextBox.vue";
import WidgetCheckBox from "./WidgetCheckBox.vue";
import WidgetSelectBox from "./WidgetSelectBox.vue";
import WidgetArray from "./WidgetArray.vue";
import WidgetMap from "./WidgetMap.vue";
import _ from "lodash";

const props = defineProps<{
  schemaProp: PropertyEditorProp;
  propValue: PropertyEditorValue;
  validation?: PropertyEditorValidation;
  path?: PropertyPath;
  collapsedPaths: Array<Array<string>>;
  disabled?: boolean;
  arrayIndex?: number;
  arrayLength?: number;
}>();

const emits = defineEmits<{
  (e: "toggleCollapsed", path: Array<string>): void;
  (e: "updatedProperty", v: UpdatedProperty): void;
  (e: "addToArray", v: AddToArray): void;
  (e: "addToMap", v: AddToMap): void;
}>();

const { arrayIndex } = toRefs(props);

const setCollapsed = (path: string[]) => {
  emits("toggleCollapsed", path);
};

const updatedProperty = (event: UpdatedProperty) => {
  emits("updatedProperty", event);
};

const addToArray = (event: AddToArray) => {
  emits("addToArray", event);
};

const addToMap = (event: AddToMap) => {
  emits("addToMap", event);
};

const showArrayElementHeader = computed(() => {
  if (_.isUndefined(arrayIndex?.value) && _.isNull(props.propValue.key)) {
    return false;
  } else {
    console.log("showing array element header", {
      ai: JSON.stringify(arrayIndex?.value),
      key: JSON.stringify(props.propValue.key),
    });
    return true;
  }
  //if (props.schemaProp.kind == "array") {
  //  console.log("checking an array", {
  //    arrayIndex: JSON.stringify(arrayIndex?.value),
  //  });

  //  if (_.isUndefined(arrayIndex?.value)) {
  //    return false;
  //  }
  //} else if (props.schemaProp.kind == "map") {
  //  if (_.isUndefined(props.propValue.key)) {
  //    return false;
  //  }
  //} else {
  //  return false;
  //}
  //return true;
});
</script>
