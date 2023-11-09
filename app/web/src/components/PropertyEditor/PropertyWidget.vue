<template>
  <div v-if="!schemaProp.isHidden" class="" @keyup.stop @keydown.stop>
    <!-- <div class="flex flex-row items-center w-full" @keyup.stop @keydown.stop> -->
    <WidgetHeader
      v-if="showArrayElementHeader"
      :name="schemaProp.name"
      :path="path"
      :collapsedPaths="collapsedPaths"
      @toggle-collapsed="setCollapsed($event)"
    />
    <WidgetHeader
      v-if="schemaProp.widgetKind.kind === 'header' && !showArrayElementHeader"
      :name="schemaProp.name"
      :path="path"
      :collapsedPaths="collapsedPaths"
      @toggle-collapsed="setCollapsed($event)"
    />
    <WidgetArray
      v-else-if="schemaProp.widgetKind.kind === 'array'"
      :name="schemaProp.name"
      :path="path"
      :collapsedPaths="collapsedPaths"
      :disabled="disabled"
      :propId="propValue.propId"
      :valueId="propValue.id"
      :arrayLength="arrayLength"
      @add-to-array="addToArray($event)"
      @toggle-collapsed="setCollapsed($event)"
    />
    <WidgetMap
      v-else-if="schemaProp.widgetKind.kind === 'map'"
      :name="schemaProp.name"
      :path="path"
      :collapsedPaths="collapsedPaths"
      :disabled="disabled"
      :propId="propValue.propId"
      :valueId="propValue.id"
      :arrayLength="arrayLength"
      @add-to-map="addToMap($event)"
      @toggle-collapsed="setCollapsed($event)"
    />
    <!-- TODO(nick): until we use the "options" for select, let's just ignore them for now and force a text box  -->
    <WidgetTextBox
      v-else-if="schemaProp.widgetKind.kind === 'text'"
      :name="schemaProp.name"
      :path="path"
      :collapsedPaths="collapsedPaths"
      :value="propValue.value"
      :propId="propValue.propId"
      :valueId="propValue.id"
      :propKind="schemaProp.kind"
      :docLink="schemaProp.docLink"
      :validation="validation"
      :disabled="disabled"
      :class="INPUT_CLASSES"
      @updated-property="updatedProperty($event)"
    />
    <WidgetTextBox
      v-else-if="schemaProp.widgetKind.kind === 'textArea'"
      textArea
      :name="schemaProp.name"
      :path="path"
      :collapsedPaths="collapsedPaths"
      :value="propValue.value"
      :propId="propValue.propId"
      :valueId="propValue.id"
      :propKind="schemaProp.kind"
      :docLink="schemaProp.docLink"
      :validation="validation"
      :disabled="disabled"
      :class="INPUT_CLASSES"
      @updated-property="updatedProperty($event)"
    />
    <WidgetTextBox
      v-else-if="schemaProp.widgetKind.kind === 'password'"
      :name="schemaProp.name"
      :path="path"
      :collapsedPaths="collapsedPaths"
      :value="propValue.value"
      :propId="propValue.propId"
      :valueId="propValue.id"
      :propKind="schemaProp.kind"
      :docLink="schemaProp.docLink"
      :validation="validation"
      :disabled="disabled"
      :class="INPUT_CLASSES"
      @updated-property="updatedProperty($event)"
    />
    <WidgetCodeEditor
      v-else-if="schemaProp.widgetKind.kind === 'codeEditor'"
      :name="schemaProp.name"
      :path="path"
      :collapsedPaths="collapsedPaths"
      :value="propValue.value"
      :propId="propValue.propId"
      :valueId="propValue.id"
      :propKind="schemaProp.kind"
      :docLink="schemaProp.docLink"
      :validation="validation"
      :disabled="disabled"
      :class="INPUT_CLASSES"
      @updated-property="updatedProperty($event)"
    />
    <WidgetCheckBox
      v-else-if="schemaProp.widgetKind.kind === 'checkbox'"
      :name="schemaProp.name"
      :path="path"
      :collapsedPaths="collapsedPaths"
      :value="propValue.value"
      :propId="propValue.propId"
      :valueId="propValue.id"
      :docLink="schemaProp.docLink"
      :validation="validation"
      :disabled="disabled"
      :class="INPUT_CLASSES"
      @updated-property="updatedProperty($event)"
    />
    <WidgetSelectBox
      v-else-if="schemaProp.widgetKind.kind === 'select'"
      :name="schemaProp.name"
      :options="schemaProp.widgetKind.options || []"
      :path="path"
      :collapsedPaths="collapsedPaths"
      :value="propValue.value"
      :propId="propValue.propId"
      :valueId="propValue.id"
      :docLink="schemaProp.docLink"
      :validation="validation"
      :disabled="disabled"
      :class="INPUT_CLASSES"
      @updated-property="updatedProperty($event)"
    />
    <WidgetComboBox
      v-else-if="schemaProp.widgetKind.kind === 'comboBox'"
      :name="schemaProp.name"
      :options="schemaProp.widgetKind.options || []"
      :path="path"
      :collapsedPaths="collapsedPaths"
      :value="propValue.value"
      :propId="propValue.propId"
      :valueId="propValue.id"
      :docLink="schemaProp.docLink"
      :validation="validation"
      :disabled="disabled"
      :class="INPUT_CLASSES"
      @updated-property="updatedProperty($event)"
    />
    <WidgetColorBox
      v-else-if="schemaProp.widgetKind.kind === 'color'"
      :name="schemaProp.name"
      :path="path"
      :collapsedPaths="collapsedPaths"
      :value="propValue.value"
      :propId="propValue.propId"
      :valueId="propValue.id"
      :docLink="schemaProp.docLink"
      :validation="validation"
      :disabled="disabled"
      :class="INPUT_CLASSES"
      @updated-property="updatedProperty($event)"
    />
    <WidgetSecret
      v-else-if="schemaProp.widgetKind.kind === 'secret'"
      :name="schemaProp.name"
      :path="path"
      :options="schemaProp.widgetKind.options || []"
      :collapsedPaths="collapsedPaths"
      :value="(propValue.value as SecretId)"
      :propId="propValue.propId"
      :valueId="propValue.id"
      @updated-property="updatedProperty($event)"
    />

    <!-- restricting to text props for now -->

    <!-- hiding fn button for now until we clean up this UI -->
    <!-- <WidgetFuncButton
      v-if="!isFirstProp"
      :func="propValue.func"
      :value-id="propValue.id"
      @create-attribute-func="onCreateAttributeFunc"
    /> -->

    <!--<div v-else>
      <div class="flex">
        {{ path }}
      </div>
      <div class="flex">
        {{ schemaProp }}
      </div>
      <div class="flex">
        {{ propValue }}
      </div>
    </div>
      -->
  </div>
</template>

<script setup lang="ts">
import { toRefs, computed } from "vue";
import * as _ from "lodash-es";
import { tw } from "@si/vue-lib";
import {
  PropertyEditorProp,
  PropertyEditorValidation,
  PropertyEditorValue,
  UpdatedProperty,
  AddToArray,
  AddToMap,
  PropertyPath,
} from "@/api/sdf/dal/property_editor";
import { SecretId } from "@/store/secrets.store";
import WidgetHeader from "./WidgetHeader.vue";
import WidgetTextBox from "./WidgetTextBox.vue";
import WidgetCodeEditor from "./WidgetCodeEditor.vue";
import WidgetCheckBox from "./WidgetCheckBox.vue";
import WidgetSelectBox from "./WidgetSelectBox.vue";
import WidgetArray from "./WidgetArray.vue";
import WidgetMap from "./WidgetMap.vue";
import WidgetComboBox from "./WidgetComboBox.vue";
import WidgetColorBox from "./WidgetColorBox.vue";
import WidgetSecret from "./WidgetSecret.vue";

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
}>();

const disabled = computed(() => props.disabled || props.schemaProp.isReadonly);

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
