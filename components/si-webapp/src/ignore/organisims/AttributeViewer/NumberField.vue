<template>
  <Field
    :name="fieldName"
    :showField="showField"
    :errors="errors"
    :editMode="editMode"
    :nameClasses="fieldNameColor"
  >
    <template slot="widget">
      <input
        class="pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none input-bg-color-grey si-property disabled:opacity-50"
        :class="borderColor"
        type="number"
        aria-label="name"
        placeholder="number"
        v-model="currentValue"
        :disabled="isDisabled"
        @input="onInput"
        @keyup.enter="onEnterKey($event)"
        @focus="onFocus"
        @blur="onBlur"
      />
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
    </template>
    <template slot="value">
      <span :class="textColor"> {{ currentValue }} </span>
    </template>
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

export default BaseField.extend({
  name: "NumberField",
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
});
</script>
