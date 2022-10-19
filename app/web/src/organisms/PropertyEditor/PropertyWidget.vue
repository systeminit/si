<template>
  <div class="" @keyup.stop @keydown.stop>
    <!-- <div class="flex flex-row items-center w-full" @keyup.stop @keydown.stop> -->
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
      :disabled="disabled"
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
      :disabled="disabled"
      :prop-id="props.propValue.propId"
      :value-id="props.propValue.id"
      :array-length="props.arrayLength"
      @add-to-map="addToMap($event)"
      @toggle-collapsed="setCollapsed($event)"
    />
    <!-- TODO(nick): until we use the "options" for select, let's just ignore them for now and force a text box  -->
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
      :disabled="disabled"
      :func="props.propValue.func"
      :class="INPUT_CLASSES"
      @updated-property="updatedProperty($event)"
    />
    <WidgetCheckBox
      v-else-if="props.schemaProp.widgetKind.kind === 'checkbox'"
      :name="props.schemaProp.name"
      :path="path"
      :collapsed-paths="props.collapsedPaths"
      :value="props.propValue.value"
      :prop-id="props.propValue.propId"
      :value-id="props.propValue.id"
      :doc-link="props.schemaProp.docLink"
      :validation="props.validation"
      :disabled="disabled"
      :class="INPUT_CLASSES"
      @updated-property="updatedProperty($event)"
    />
    <WidgetSelectBox
      v-else-if="
        props.schemaProp.widgetKind.kind === 'secretSelect' ||
        props.schemaProp.widgetKind.kind === 'select'
      "
      :name="props.schemaProp.name"
      :options="props.schemaProp.widgetKind.options || []"
      :path="path"
      :collapsed-paths="props.collapsedPaths"
      :value="props.propValue.value"
      :prop-id="props.propValue.propId"
      :value-id="props.propValue.id"
      :doc-link="props.schemaProp.docLink"
      :validation="props.validation"
      :disabled="disabled"
      :class="INPUT_CLASSES"
      @updated-property="updatedProperty($event)"
    />

    <!-- restricting to text props for now -->

    <!-- hiding fn button for now until we clean up this UI -->
    <!-- <WidgetFuncButton
      v-if="!props.isFirstProp"
      :func="props.propValue.func"
      :value-id="props.propValue.id"
      @create-attribute-func="onCreateAttributeFunc"
    /> -->

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
import _ from "lodash";
import {
  PropertyEditorProp,
  PropertyEditorValidation,
  PropertyEditorValue,
  UpdatedProperty,
  AddToArray,
  AddToMap,
  PropertyPath,
  FuncWithPrototypeContext,
} from "@/api/sdf/dal/property_editor";
import { isCustomizableFuncKind } from "@/api/sdf/dal/func";
import { tw } from "@/utils/style_helpers";
import WidgetHeader from "./WidgetHeader.vue";
import WidgetTextBox from "./WidgetTextBox.vue";
import WidgetCheckBox from "./WidgetCheckBox.vue";
import WidgetSelectBox from "./WidgetSelectBox.vue";
import WidgetArray from "./WidgetArray.vue";
import WidgetMap from "./WidgetMap.vue";
// import WidgetFuncButton from "./WidgetFuncButton.vue";

const INPUT_CLASSES = tw`pl-lg pr-sm pt-sm`;

const props = defineProps<{
  schemaProp: PropertyEditorProp;
  propValue: PropertyEditorValue;
  validation?: PropertyEditorValidation;
  path?: PropertyPath;
  collapsedPaths: Array<Array<string>>;
  disabled?: boolean;
  arrayIndex?: number;
  arrayLength?: number;
  isFirstProp?: boolean;
}>();

const emits = defineEmits<{
  (e: "toggleCollapsed", path: Array<string>): void;
  (e: "updatedProperty", v: UpdatedProperty): void;
  (e: "addToArray", v: AddToArray): void;
  (e: "addToMap", v: AddToMap): void;
  (
    e: "createAttributeFunc",
    currentFunc: FuncWithPrototypeContext,
    valueId: number,
  ): void;
}>();

// If we have a custom func attached, don't allow them to set the attribute
const disabled = computed(
  () =>
    props.disabled || isCustomizableFuncKind(props.propValue.func.backendKind),
);

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

// const onCreateAttributeFunc = (
//   currentFunc: FuncWithPrototypeContext,
//   valueId: number,
// ) => emits("createAttributeFunc", currentFunc, valueId);

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
  // if (props.schemaProp.kind === "array") {
  //  console.log("checking an array", {
  //    arrayIndex: JSON.stringify(arrayIndex?.value),
  //  });

  //  if (_.isUndefined(arrayIndex?.value)) {
  //    return false;
  //  }
  // } else if (props.schemaProp.kind === "map") {
  //  if (_.isUndefined(props.propValue.key)) {
  //    return false;
  //  }
  // } else {
  //  return false;
  // }
  // return true;
});
</script>
