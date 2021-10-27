<template>
  <div v-on="$listeners" class="w-full">
    <div
      v-for="editField in editFields"
      :key="editField.path.join('.')"
      class="my-2"
    >
      <Header
        :entity="entity"
        :editMode="editMode"
        :editField="editField"
        :systemId="systemId"
        :backgroundColors="backgroundColors"
        :outdentCount="outdentCount"
        :treeOpenState="treeOpenState"
        @toggle-path="togglePath"
        @set-tree-open-state="setTreeOpenState"
        v-if="showFieldForWidget('header', editField)"
      />
      <TextField
        :entity="entity"
        :editMode="editMode"
        :editField="editField"
        :systemId="systemId"
        :diff="diff"
        v-else-if="showFieldForWidget('text', editField)"
      />
      <NumberField
        :entity="entity"
        :editMode="editMode"
        :editField="editField"
        :systemId="systemId"
        :diff="diff"
        v-else-if="showFieldForWidget('number', editField)"
      />
      <TextAreaField
        :entity="entity"
        :editMode="editMode"
        :editField="editField"
        :systemId="systemId"
        :diff="diff"
        v-else-if="showFieldForWidget('textArea', editField)"
      />
      <PasswordField
        :entity="entity"
        :editMode="editMode"
        :editField="editField"
        :systemId="systemId"
        :diff="diff"
        v-else-if="showFieldForWidget('password', editField)"
      />
      <CheckboxField
        :entity="entity"
        :editMode="editMode"
        :editField="editField"
        :systemId="systemId"
        :diff="diff"
        v-else-if="showFieldForWidget('checkbox', editField)"
      />
      <SelectField
        :entity="entity"
        :editMode="editMode"
        :editField="editField"
        :systemId="systemId"
        :diff="diff"
        v-else-if="showFieldForWidget('select', editField)"
      />
      <SelectFromInputField
        :entity="entity"
        :editMode="editMode"
        :editField="editField"
        :systemId="systemId"
        :diff="diff"
        v-else-if="showFieldForWidget('selectFromInput', editField)"
      />
      <SelectFromSecretField
        :entity="entity"
        :editMode="editMode"
        :editField="editField"
        :systemId="systemId"
        :diff="diff"
        v-else-if="showFieldForWidget('selectFromSecret', editField)"
      />
      <MapField
        :entity="entity"
        :editMode="editMode"
        :editField="editField"
        :systemId="systemId"
        :diff="diff"
        v-else-if="showFieldForWidget('map', editField)"
      />
      <ArrayField
        :entity="entity"
        :editMode="editMode"
        :editField="editField"
        :systemId="systemId"
        :diff="diff"
        :backgroundColors="backgroundColors"
        :treeOpenState="treeOpenState"
        @toggle-path="togglePath"
        @set-tree-open-state="setTreeOpenState"
        v-else-if="showFieldForWidget('array', editField)"
      />
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import _ from "lodash";
import TextField from "@/organisims/AttributeViewer/TextField.vue";
import CheckboxField from "@/organisims/AttributeViewer/CheckboxField.vue";
import NumberField from "@/organisims/AttributeViewer/NumberField.vue";
import TextAreaField from "@/organisims/AttributeViewer/TextAreaField.vue";
import PasswordField from "@/organisims/AttributeViewer/PasswordField.vue";
import SelectField from "@/organisims/AttributeViewer/SelectField.vue";
import MapField from "@/organisims/AttributeViewer/MapField.vue";
import ArrayField from "@/organisims/AttributeViewer/ArrayField.vue";
import SelectFromInputField from "@/organisims/AttributeViewer/SelectFromInputField.vue";
import SelectFromSecretField from "@/organisims/AttributeViewer/SelectFromSecretField.vue";
import Header from "@/organisims/AttributeViewer/Header.vue";

import { EditField } from "si-entity/dist/siEntity";
import { Entity } from "@/api/sdf/model/entity";
import { Diff } from "@/api/sdf/model/diff";

// @ts-ignore
export default Vue.extend({
  name: "EditFields",
  components: {
    TextField,
    TextAreaField,
    CheckboxField,
    NumberField,
    Header,
    PasswordField,
    SelectField,
    MapField,
    ArrayField,
    SelectFromInputField,
    SelectFromSecretField,
  },
  props: {
    entity: {
      type: Object as PropType<Entity>,
      required: true,
    },
    editMode: {
      type: Boolean,
      required: true,
    },
    editFields: {
      type: Array as PropType<EditField[]>,
      required: true,
    },
    systemId: {
      type: String,
    },
    treeOpenState: {
      type: Object as PropType<{ [pathKey: string]: boolean }>,
      required: true,
    },
    backgroundColors: {
      type: Array as PropType<number[][]>,
      required: true,
    },
    diff: {
      type: Array as PropType<Diff>,
    },
    outdentCount: {
      type: Number,
    },
  },
  methods: {
    togglePath(pathKey: string) {
      this.$emit("toggle-path", pathKey);
    },
    setTreeOpenState(entry: { key: string; value: boolean }) {
      this.$emit("set-tree-open-state", entry);
    },
    showFieldForWidget(widget: string, editField: EditField): boolean {
      // Find all parent header paths from the `EditField`, sorted by hierarchy
      const pathKeys = Object.keys(this.treeOpenState)
        .filter(pathKey =>
          this.entity.subPath(editField.path, pathKey.split("::")),
        )
        .sort();

      // Check each parent header, starting at the top of the hierarchy
      for (const pathKey of pathKeys) {
        // If this path (one of the `EditField`'s parents) is closed...
        if (this.treeOpenState[pathKey] === false) {
          if (
            editField.widgetName == "header" &&
            widget == "header" &&
            _.isEqual(editField.path, pathKey.split("::"))
          ) {
            // If this path is the path for this `EditField`, we show it!
            return true;
          } else {
            // Otherwise a parent path for the `EditField` is closed and we
            // should not display ourselves
            return false;
          }
        }
      }

      // At this point, all parent header paths are open, so we should display
      // if the desired widget matches the `EditField`'s kind
      return editField.widgetName == widget;
    },
  },
});
</script>
