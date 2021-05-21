<template>
  <Field
    :name="fieldName"
    :showField="showField"
    :errors="errors"
    :editMode="editMode"
  >
    <template slot="widget">
      <select
        class="flex-grow pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none input-bg-color-grey input-border-grey si-property disabled:opacity-50"
        placeholder="text"
        v-model="currentValue"
        :disabled="isDisabled"
        @change="onInputSelect"
        @focus="onFocus"
        @blur="onBlur"
      >
        <option
          v-for="option in selectOptions"
          :key="option.value"
          :value="option.value"
          >{{ option.label }}
        </option>
      </select>
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
    <template slot="value"> {{ currentValue }} </template>
  </Field>
</template>

<script lang="ts">
import _ from "lodash";

import TombstoneEdit from "@/organisims/AttributeViewer/Tombstone.vue";
import Unset from "@/organisims/AttributeViewer/Unset.vue";
import Field from "@/organisims/AttributeViewer/Field.vue";
import { ValidateFailure } from "si-entity/dist/validation";
import BaseField from "./BaseField.vue";

interface Data {
  startValue: string;
  currentValue: string;
  updating: boolean;
  errors: ValidateFailure["errors"];
}

interface SelectItem {
  label: string;
  value: string | number;
}

export default BaseField.extend({
  name: "SelectField",
  mixins: [BaseField],
  components: {
    TombstoneEdit,
    Unset,
    Field,
  },
  data(): Data {
    return {
      startValue: "",
      currentValue: "",
      updating: false,
      errors: [],
    };
  },
  computed: {
    selectOptions(): SelectItem[] {
      if (this.editField.schema.widget?.name == "select") {
        return this.editField.schema.widget.options.items;
      } else {
        return [];
      }
    },
  },
});
</script>
