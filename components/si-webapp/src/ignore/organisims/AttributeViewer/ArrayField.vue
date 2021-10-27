<template>
  <Field
    :name="fieldName"
    :showField="showField"
    :errors="errors"
    :editMode="editMode"
    :nameClasses="fieldNameColor"
    v-on="$listeners"
  >
    <template slot="widget">
      <div class="flex flex-row w-full border border-gray-700">
        <div class="flex flex-col px-1 py-1">
          <div
            class="flex mt-1"
            v-for="(editFields, index) in items"
            :key="index"
          >
            <div
              class="flex flex-row justify-between w-full mx-1 border border-gray-500"
            >
              <ArrayEditFields
                :entity="entity"
                :editMode="editMode"
                :editFields="editFields"
                :systemId="systemId"
                :backgroundColors="backgroundColors"
                :diff="diff"
                :outdentCount="outdentCount"
                :treeOpenState="treeOpenState"
                @toggle-path="togglePath"
                @set-tree-open-state="setTreeOpenState"
              />
              <div class="flex w-5 bg-gray-600">
                <Unset
                  :entity="entity"
                  :editField="arrayEntryEditField(index)"
                  :systemId="systemId"
                  @unset="unset(arrayEntryEditField(index))"
                />
              </div>
            </div>
          </div>
          <div class="flex flex-row mt-1 ml-1">
            <ArrayAddEntry
              :entity="entity"
              :editField="editField"
              :systemId="systemId"
              :items="items"
              @add-item-edit-fields="addItemEditFields"
            />
          </div>
        </div>

        <div class="flex flex-row items-center justify-center w-10 bg-gray-800">
          <div class="flex">
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
          </div>
        </div>
      </div>
    </template>
    <template slot="value">
      <div class="flex flex-col mt-4">
        <div
          class="flex flex-row px-1 py-1 border border-gray-700"
          v-for="(editFields, index) in items"
          :key="index"
        >
          <ArrayEditFields
            :entity="entity"
            :editMode="editMode"
            :editFields="editFields"
            :systemId="systemId"
            :backgroundColors="backgroundColors"
            :treeOpenState="treeOpenState"
            :diff="diff"
            @toggle-path="togglePath"
            @set-tree-open-state="setTreeOpenState"
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
import ArrayAddEntry from "@/molecules/ArrayAddEntry.vue";
import { EditPartial } from "si-registry/dist/registryEntry";

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
    ArrayAddEntry,
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
  computed: {
    schemaEditPartials(): EditPartial[] | undefined {
      if (
        this.editField.schema.type == "array" &&
        this.editField.schema.itemProperty.type == "object"
      ) {
        return this.editField.schema.itemProperty.editPartials;
      } else {
        return undefined;
      }
    },
    schemaHasEditPartials(): boolean {
      if (this.schemaEditPartials) {
        return true;
      } else {
        return false;
      }
    },
    outdentCount(): number | undefined {
      if (this.editField.type == "array") {
        // outdent for child headers is the path length plus 2, where 1
        // is for the array index entry and the other 1 is for the first key
        // under the index entry.
        return this.editField.path.length + 2;
      } else {
        return undefined;
      }
    },
  },
  methods: {
    togglePath(pathKey: string) {
      this.$emit("toggle-path", pathKey);
    },
    setTreeOpenState(entry: { key: string; value: boolean }) {
      this.$emit("set-tree-open-state", entry);
    },
    addItemEditFields(fields: EditField[]) {
      this.items.push(fields);
    },
    arrayEntryEditField(index: number): EditField {
      let editField = _.cloneDeep(this.editField);
      editField.path.push(`${index}`);
      return editField;
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
    setItems() {
      const items = [];
      if (!_.isUndefined(this.currentValue)) {
        const parentPath = this.editField.path;
        for (let index = 0; index < this.currentValue.length; index++) {
          let editFields = this.entity.arrayEditFields(this.editField, index);
          if (this.schemaHasEditPartials) {
            // @ts-ignore
            const opSetPartials: OpSet[] = _.filter(
              this.entity.ops,
              o =>
                o.system == this.systemId &&
                o.op == "set" &&
                o.path.length == parentPath.length + 1 &&
                o.editPartial,
            );
            for (const op of opSetPartials) {
              if (op.editPartial) {
                editFields = this.filterEditFieldsForPartial(
                  editFields,
                  op.editPartial,
                  op.path,
                );
              }
            }
          }
          items.push(editFields);
        }
      }
      this.items = items;
    },
    filterEditFieldsForPartial(
      arrayEditFields: EditField[],
      editPartialName: string,
      pathPrefix: string[],
    ): EditField[] {
      const result = [];
      for (const editField of arrayEditFields) {
        if (this.entity.subPath(editField.path, pathPrefix)) {
          // If the editField falls under the array item path, then we need to
          // filter
          const pathPrefixes = this.propertyPathPrefixes(
            editPartialName,
            pathPrefix,
          );
          if (
            _.some(pathPrefixes, prefix =>
              this.entity.subPath(editField.path, prefix),
            )
          ) {
            result.push(editField);
          }
        } else {
          // Otherwise keep the editField
          result.push(editField);
        }
      }
      return result;
    },
    propertyPathPrefixes(
      name: string,
      pathRoot: string[],
      schemaEditPartials?: EditPartial[],
    ): string[][] | null {
      if (!schemaEditPartials) {
        schemaEditPartials = this.schemaEditPartials;
      }
      if (!schemaEditPartials) {
        return null;
      }

      for (const editPartial of schemaEditPartials) {
        if (editPartial.kind == "item") {
          if (editPartial.name == name) {
            return editPartial.propertyPaths.map(e => pathRoot.concat(e));
          }
        } else {
          const result = this.propertyPathPrefixes(
            name,
            pathRoot,
            editPartial.items,
          );
          if (result) {
            return result;
          }
        }
      }
      return null;
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
