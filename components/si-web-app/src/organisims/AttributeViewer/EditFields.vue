<template>
  <div v-on="$listeners">
    <div v-for="editField in editFields" :key="editField.path.join('.')">
      <Header
        :entity="entity"
        :editMode="editMode"
        :editField="editField"
        :systemId="systemId"
        :backgroundColors="backgroundColors"
        @toggle-path="togglePath"
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
        :closedPaths="closedPaths"
        @toggle-path="togglePath"
        v-else-if="showFieldForWidget('array', editField)"
      />
      <div v-else>Widget type does not exist for {{ editField }}! Bug</div>
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
    closedPaths: {
      type: Array as PropType<string[][]>,
      required: true,
    },
    backgroundColors: {
      type: Array as PropType<number[][]>,
      required: true,
    },
    diff: {
      type: Array as PropType<Diff>,
    },
  },
  methods: {
    togglePath(event: any) {
      this.$emit("toggle-path", event);
    },
    showFieldForWidget(widget: string, editField: EditField): boolean {
      let closedByPath = _.find(this.closedPaths, p =>
        _.isEqual(p, editField.path.slice(0, p.length)),
      );
      if (closedByPath) {
        if (editField.widgetName == "header") {
          let isHeader = _.find(this.closedPaths, p =>
            _.isEqual(p, editField.path),
          );
          if (isHeader) {
            return true;
          } else {
            return false;
          }
        } else {
          return false;
        }
      }
      return editField.widgetName == widget;
    },
  },
});
</script>
