<template>
  <Field
    :name="fieldName"
    :showField="showField"
    :errors="errors"
    :editMode="editMode"
    v-on="$listeners"
  >
    <template slot="widget">
      <div class="flex flex-col flex-grow">
        <div
          class="flex flex-row border-b-2 pb-2 border-gray-800"
          v-for="(editFields, index) in items"
          :key="index"
        >
          <ArrayEditFields
            :entity="entity"
            :editMode="editMode"
            :editFields="editFields"
            :systemId="systemId"
            :backgroundColors="backgroundColors"
            :closedPaths="closedPaths"
            :diff="diff"
            @toggle-path="togglePath"
          />
          <Unset
            :entity="entity"
            :editField="arrayEntryEditField(index)"
            :systemId="systemId"
            @unset="unset(arrayEntryEditField(index))"
          />
        </div>
        <div class="flex flex-row pl-2 pt-2">
          <button>
            <PlusIcon @click="addItem" size="1x" />
          </button>
        </div>
      </div>
      <TombstoneEdit
        :entity="entity"
        :editField="editField"
        :systemId="systemId"
        @toggleTombstone="toggleTombstone"
      />
      <Unset
        :entity="entity"
        :editField="editField"
        :systemId="systemId"
        @unset="unset"
      />
    </template>
    <template slot="value">
      <div class="flex flex-col">
        <div
          class="flex flex-row border-b-2 border-gray-800"
          v-for="(editFields, index) in items"
          :key="index"
        >
          <ArrayEditFields
            :entity="entity"
            :editMode="editMode"
            :editFields="editFields"
            :systemId="systemId"
            :backgroundColors="backgroundColors"
            :closedPaths="closedPaths"
            :diff="diff"
            @toggle-path="togglePath"
          />
        </div>
      </div>
    </template>
  </Field>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import _ from "lodash";
import {
  EditField,
  OpSet,
  OpType,
  OpSource,
  OpUnset,
  OpTombstone,
} from "si-entity/dist/siEntity";

import TombstoneEdit from "@/organisims/AttributeViewer/Tombstone.vue";
import Unset from "@/organisims/AttributeViewer/Unset.vue";
import Field from "@/organisims/AttributeViewer/Field.vue";
import { ValidateFailure } from "si-entity/dist/validation";
import BaseField from "./BaseField.vue";
import { Entity } from "@/api/sdf/model/entity";

import { PlusIcon } from "vue-feather-icons";
import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";
import { updateEntity } from "@/observables";
import { Diff } from "@/api/sdf/model/diff";

interface Data {
  startValue: unknown[];
  currentValue: unknown[];
  items: EditField[][];
  updating: boolean;
  errors: ValidateFailure["errors"];
}

// @ts-ignore
export default BaseField.extend({
  name: "ArrayField",
  components: {
    TombstoneEdit,
    Unset,
    Field,
    PlusIcon,
    ArrayEditFields: () => import("./EditFields.vue"),
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
    editField: {
      type: Object as PropType<EditField>,
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
  data(): Data {
    return {
      startValue: [],
      currentValue: [],
      updating: false,
      errors: [],
      items: [],
    };
  },
  methods: {
    togglePath(event: any) {
      this.$emit("toggle-path", event);
    },
    arrayEntryEditField(index: number): EditField {
      let editField = _.cloneDeep(this.editField);
      editField.path.push(`${index}`);
      return editField;
    },
    arrayEditFields(): EditField[] {
      if (this.entity) {
        let nextIndex = this.nextIndex();
        return this.entity.arrayEditFields(this.editField, nextIndex);
      } else {
        return [];
      }
    },
    nextIndex(): number {
      let fullPath = [this.entity.entityType].concat(this.editField.path);
      let arrayMetaKey = this.entity.pathToString(fullPath);
      let arrayLength = this.entity.arrayMeta[arrayMetaKey]?.length;
      if (!arrayLength) {
        arrayLength = 0;
      }
      return arrayLength;
    },
    addItem() {
      this.items.push(this.arrayEditFields());
      let path = _.cloneDeep(this.editField.path);
      let nextIndex = this.nextIndex();
      path.push(`${nextIndex}`);
      let value: unknown = "";
      if (this.editField.schema.type == "array") {
        if (this.editField.schema.itemProperty.type == "string") {
          value = "";
        } else if (this.editField.schema.itemProperty.type == "number") {
          value = 0;
        } else if (this.editField.schema.itemProperty.type == "boolean") {
          value = false;
        } else if (this.editField.schema.itemProperty.type == "object") {
          value = {};
        } else if (this.editField.schema.itemProperty.type == "array") {
          value = [];
        } else if (this.editField.schema.itemProperty.type == "map") {
          value = {};
        }
      }
      const opSet: OpSet = {
        op: OpType.Set,
        source: OpSource.Manual,
        path,
        // @ts-ignore
        value: _.cloneDeep(value),
        system: this.systemId,
      };
      const result = this.entity.addOpSet(opSet);
      if (!result.success) {
        emitEditorErrorMessage(result.errors.join("\n"));
      }
      this.entity.computeProperties();
      updateEntity(this.entity).subscribe(reply => {
        if (reply.error) {
          emitEditorErrorMessage(reply.error.message);
        }
      });
    },
    setItems() {
      const items = [];
      if (!_.isUndefined(this.currentValue)) {
        for (let index = 0; index < this.currentValue.length; index++) {
          const editFields = this.entity.arrayEditFields(this.editField, index);
          items.push(editFields);
        }
      }
      this.items = items;
    },
    updateOnPropChanges() {
      if (!this.updating && this.entity) {
        const startValue: string = this.entity.getProperty({
          system: this.systemId,
          path: this.editField.path,
        });
        this.setCurrentValue(_.cloneDeep(startValue));
        this.setStartValueToCurrentValue();
        this.setItems();
      }
    },
  },
});
</script>
