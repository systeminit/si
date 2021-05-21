<template>
  <Field
    :name="fieldName"
    :showField="showField"
    :errors="errors"
    :editMode="editMode"
    :nameClasses="fieldNameColor"
  >
    <template slot="widget">
      <div class="flex flex-row w-full pl-5">
        <div class="flex flex-col">
          <div
            class="flex flex-row flex-grow pb-2"
            v-for="(mapValue, index) in sortedCurrentValue"
            :key="mapValue.key + index"
          >
            <div class="flex w-1/2 mr-2">
              <input
                type="text"
                @focus="onFocus"
                @blur="onBlurForKey(mapValue.key, index)"
                @keyup.enter="onEnterKey($event)"
                v-model="keyValue[index]"
                class="w-full pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none input-bg-color-grey si-property disabled:opacity-50"
                :class="borderColorForKey(mapValue.key)"
                placeholder="key"
              />
            </div>
            <div class="flex w-1/2 mr-2">
              <input
                type="text"
                @focus="onFocus"
                @blur="onBlurForKeyValue(mapValue.key)"
                @keyup.enter="onEnterKey($event)"
                v-model="currentValue[mapValue.key]"
                class="w-full pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none input-bg-color-grey si-property disabled:opacity-50"
                :class="borderColorForKey(mapValue.key)"
                placeholder="value"
              />
            </div>
            <div class="flex flex-row w-10">
              <TombstoneEdit
                :entity="entity"
                :editField="editFieldForKey(mapValue.key)"
                :systemId="systemId"
                @toggleTombstone="toggleTombstoneForKey(mapValue.key, $event)"
              />
              <Unset
                :entity="entity"
                :editField="editFieldForKey(mapValue.key)"
                :systemId="systemId"
                @unset="unsetForKey(mapValue.key)"
              />
            </div>
          </div>
          <div class="flex flex-row flex-grow border-t border-gray-800">
            <div class="flex w-1/2 mr-2">
              <input
                type="text"
                v-model="newKey"
                class="w-full pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none input-bg-color-grey input-border-grey si-property disabled:opacity-50"
                placeholder="key"
              />
            </div>
            <div class="flex w-1/2 mr-2">
              <input
                type="text"
                v-model="newValue"
                class="w-full pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none input-bg-color-grey input-border-grey si-property disabled:opacity-50"
                placeholder="value"
              />
            </div>
            <div class="flex flex-row w-10">
              <button @click="addToMap"><PlusIcon size="1x" /></button>
            </div>
          </div>
        </div>
      </div>
      <div class="flex flex-row w-10">
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
    </template>
    <template slot="value">
      <div class="flex flex-col">
        <div
          class="grid grid-cols-2 gap-1"
          :class="textColorForKey(key)"
          v-for="(value, key) in currentValue"
          :key="key"
        >
          <div class="flex flex-row justify-end">
            <span :class="fieldNameColorForKey(key)"> {{ key }}</span
            >:
          </div>
          <div class="flex flex-row">
            {{ value }}
          </div>
        </div>
      </div>
    </template>
  </Field>
</template>

<script lang="ts">
import _ from "lodash";
import {
  EditField,
  OpSet,
  OpType,
  OpSource,
  OpTombstone,
} from "si-entity/dist/siEntity";

import TombstoneEdit from "@/organisims/AttributeViewer/Tombstone.vue";
import Unset from "@/organisims/AttributeViewer/Unset.vue";
import Field from "@/organisims/AttributeViewer/Field.vue";
import { ValidateFailure } from "si-entity/dist/validation";
import BaseField from "./BaseField.vue";

import { PlusIcon } from "vue-feather-icons";
import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";
import { updateEntity } from "@/observables";
import { hasDiff } from "@/api/sdf/model/diff";

interface Data {
  newKey: string;
  newValue: string;
  startValue: Record<string, string>;
  currentValue: Record<string, string>;
  updating: boolean;
  errors: ValidateFailure["errors"];
  keyValue: Record<string, string>;
}

interface KvItem {
  key: string;
  value: string;
}

export default BaseField.extend({
  name: "MapField",
  mixins: [BaseField],
  components: {
    TombstoneEdit,
    Unset,
    Field,
    PlusIcon,
  },
  data(): Data {
    return {
      newKey: "",
      newValue: "",
      startValue: {},
      currentValue: {},
      updating: false,
      errors: [],
      keyValue: {},
    };
  },
  computed: {
    sortedCurrentValue(): KvItem[] {
      const data: KvItem[] = [];
      if (this.currentValue && _.isObject(this.currentValue)) {
        for (let key of Object.keys(this.currentValue)) {
          // @ts-ignore
          data.push({ key, value: this.currentValue[key] });
        }
      }
      let sorted = _.sortBy(data, ["key"]);
      for (let index = 0; index < sorted.length; index++) {
        this.keyValue[index] = sorted[index].key;
      }
      return _.sortBy(data, ["key"]);
    },
  },
  methods: {
    fieldNameColorForKey(key: string): Record<string, boolean> {
      const opSet = this.entity.valueOpForPath({
        path: _.concat(this.editField.path, key),
        system: this.systemId,
      });
      if (opSet) {
        if (opSet.source == OpSource.Inferred) {
          return {
            "text-green": true,
          };
        } else {
          return {
            "text-green": false,
          };
        }
      } else {
        return {
          "text-green": false,
        };
      }
    },
    textColorForKey(key: string): Record<string, boolean> {
      let gold = hasDiff(
        this.diff,
        _.concat(["properties"], this.editField.path, key),
      );
      if (gold) {
        return {
          "text-gold": true,
        };
      }
      gold = hasDiff(
        this.diff,
        _.concat(["properties", this.systemId], this.editField.path, key),
      );
      if (gold) {
        return {
          "text-gold": true,
        };
      } else {
        return {
          "text-gold": false,
        };
      }
    },
    borderColorForKey(key: string): Record<string, boolean> {
      let gold = hasDiff(
        this.diff,
        _.concat(["properties"], this.editField.path, key),
      );
      if (gold) {
        return {
          "input-border-gold": true,
        };
      }
      gold = hasDiff(
        this.diff,
        _.concat(["properties", this.systemId], this.editField.path, key),
      );
      if (gold) {
        return {
          "input-border-gold": true,
        };
      } else {
        return {
          "input-border-grey": true,
        };
      }
    },
    editFieldForKey(key: string): EditField {
      const editField = _.cloneDeep(this.editField);
      editField.path.push(key);
      return editField;
    },
    toggleTombstoneForKey(
      key: string,
      event: { source: OpTombstone["source"]; system?: OpTombstone["system"] },
    ): void {
      let editField = this.editFieldForKey(key);
      this.toggleTombstone(event, editField);
    },
    unsetForKey(key: string): void {
      let editField = this.editFieldForKey(key);
      this.unset(editField);
    },
    onBlurForKey(oldKey: string, index: number): void {
      this.updating = false;
      if (_.isEqual(oldKey, this.keyValue[index])) {
        return;
      }
      let unsetKey = oldKey;
      let key = this.keyValue[index];
      let editField = this.editFieldForKey(key);
      let path = editField.path;
      let opSet: OpSet = {
        op: OpType.Set,
        source: OpSource.Manual,
        path,
        // @ts-ignore
        value: this.currentValue[unsetKey],
        system: this.systemId,
      };
      this.entity.addOpSet(opSet);
      let unsetEditField = this.editFieldForKey(unsetKey);
      this.unset(unsetEditField);
    },
    onBlurForKeyValue(key: string): void {
      let editField = this.editFieldForKey(key);
      this.onBlur(editField, this.currentValue[key]);
    },
    addToMap(): void {
      if (this.newKey && this.newValue) {
        if (this.currentValue && this.currentValue[this.newKey]) {
          emitEditorErrorMessage("key already exists in map; delete it first");
        } else {
          let path = _.cloneDeep(this.editField.path);
          path.push(this.newKey);
          let opSet: OpSet = {
            op: OpType.Set,
            source: OpSource.Manual,
            path,
            // @ts-ignore
            value: this.newValue,
            system: this.systemId,
          };
          this.entity.addOpSet(opSet);
          this.entity.computeProperties();
          updateEntity(this.entity).subscribe(reply => {
            if (reply.error) {
              emitEditorErrorMessage(reply.error.message);
            }
          });
          this.newKey = "";
          this.newValue = "";
        }
      }
    },
  },
});
</script>
