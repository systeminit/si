<template>
  <Field
    :name="fieldName"
    :showField="showField"
    :errors="errors"
    :editMode="editMode"
  >
    <template slot="widget">
      <textarea
        class="flex-grow pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none input-bg-color-grey input-border-grey si-property disabled:opacity-50"
        placeholder="text"
        v-model="currentValue"
        :disabled="isDisabled"
        @input="onInput"
        @focus="onFocus"
        @blur="onBlur"
      >
      </textarea>
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

export default BaseField.extend({
  name: "TextField",
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
