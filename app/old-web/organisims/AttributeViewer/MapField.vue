<template>
  <Field
    :name="fieldName"
    :showField="showField"
    :errors="errors"
    :editMode="editMode"
    :nameClasses="fieldNameColor"
  >
    <template slot="widget">
      <div class="flex flex-row w-full">
        <div class="flex flex-col px-2 py-2 border border-gray-700">
          <div
            class="flex flex-row pb-2"
            v-for="(mapValue, index) in sortedCurrentValue"
            :key="mapValue.key + index"
          >
            <div class="flex">
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
            <div class="flex ml-2">
              <input
                v-if="valueWidget == 'text'"
                type="text"
                @focus="onFocus"
                @blur="onBlurForKeyValue(mapValue.key)"
                @keyup.enter="onEnterKey($event)"
                v-model="currentValue[mapValue.key]"
                class="w-full pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none input-bg-color-grey si-property disabled:opacity-50"
                :class="borderColorForKey(mapValue.key)"
                placeholder="value"
              />
              <textarea
                v-else
                @focus="onFocus"
                @blur="onBlurForKeyValue(mapValue.key)"
                @keyup.enter="onEnterKey($event)"
                v-model="currentValue[mapValue.key]"
                class="w-full h-5 pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none input-bg-color-grey si-property disabled:opacity-50"
                :class="borderColorForKey(mapValue.key)"
                placeholder="value"
              />
            </div>
            <div class="flex flex-row w-10 ml-1">
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
          <div class="flex flex-row flex-grow">
            <div class="flex flex-row mt-1">
              <div class="flex mr-2">
                <input
                  type="text"
                  v-model="newKey"
                  class="w-full pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none input-bg-color-grey input-border-grey si-property disabled:opacity-50"
                  placeholder="key"
                />
              </div>
              <div class="flex mr-2">
                <input
                  type="text"
                  v-if="valueWidget == 'text'"
                  v-model="newValue"
                  class="w-full pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none input-bg-color-grey input-border-grey si-property disabled:opacity-50"
                  placeholder="value"
                />
                <textarea
                  v-else
                  v-model="newValue"
                  class="w-full h-5 pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none input-bg-color-grey input-border-grey si-property disabled:opacity-50"
                  placeholder="value"
                />
              </div>
              <div class="flex w-10">
                <button @click="addToMap"><PlusIcon size="1x" /></button>
              </div>
            </div>
          </div>
        </div>

        <div class="flex flex-row w-10 ml-1">
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
    </template>
    <template slot="value">
      <div class="flex flex-col w-full mt-4">
        <div
          class="flex"
          :class="textColorForKey(key)"
          v-for="(value, key) in currentValue"
          :key="key"
        >
          <div class="flex flex-row w-full">
            <div class="text-right text-gray-300">
              <span :class="fieldNameColorForKey(key)"> {{ key }}</span
              >:
            </div>
            <div
              class="ml-1 font-light text-left"
              v-if="valueWidget == 'textArea'"
            >
              <pre>
              <code class="whitespace-pre">
{{ value }}
              </code>
              </pre>
            </div>
            <div class="ml-1 font-light text-left" v-else>
              {{ value }}
            </div>
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
    valueWidget(): "text" | "textArea" {
      if (this.editField.schema.type == "map") {
        if (this.editField.schema.valueProperty.widget?.name == "textArea") {
          return "textArea";
        }
      }
      return "text";
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
            "driven-field": true,
          };
        } else {
          return {
            "driven-field": false,
          };
        }
      } else {
        return {
          "driven-field": false,
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
