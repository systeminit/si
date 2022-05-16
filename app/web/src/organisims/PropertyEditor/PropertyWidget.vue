<template>
  <div class="flex flex-col" @keyup.stop @keydown.stop>
    <WidgetHeader
      v-if="props.schemaProp.widgetKind == 'header'"
      :name="props.schemaProp.name"
      :path="path"
      :collapsed-paths="props.collapsedPaths"
      @toggle-collapsed="setCollapsed($event)"
    />
    <WidgetTextBox
      v-else-if="props.schemaProp.widgetKind == 'text'"
      :name="props.schemaProp.name"
      :path="path"
      :collapsed-paths="props.collapsedPaths"
      :value="props.propValue.value"
      :prop-id="props.propValue.propId"
      :value-id="props.propValue.id"
      @updated-property="updatedProperty($event)"
      class="py-4 px-8"
    />
    <div v-else>
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
  </div>
</template>

<script setup lang="ts">
import {
  PropertyEditorProp,
  PropertyEditorValue,
  UpdatedProperty,
} from "@/api/sdf/dal/property_editor";
import WidgetHeader from "./WidgetHeader.vue";
import WidgetTextBox from "./WidgetTextBox.vue";

const props = defineProps<{
  schemaProp: PropertyEditorProp;
  propValue: PropertyEditorValue;
  path: Array<string>;
  collapsedPaths: Array<Array<string>>;
}>();

const emits = defineEmits<{
  (e: "toggleCollapsed", path: Array<string>): void;
  (e: "updatedProperty", v: UpdatedProperty): void;
}>();

const setCollapsed = (path: string[]) => {
  emits("toggleCollapsed", path);
};

const updatedProperty = (event: UpdatedProperty) => {
  emits("updatedProperty", event);
};
</script>
