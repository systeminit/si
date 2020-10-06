<template>
  <div v-if="errors.length" class="pt-2">
    <ul>
      <li v-for="(error, index) of errors" :key="index" class="text-red-400">
        <div class="flex align-middle">
          <AlertTriangleIcon size="1x" />
          <div>
            {{ error.message }}
          </div>
        </div>
      </li>
    </ul>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import Joi from "joi";
import { AlertTriangleIcon } from "vue-feather-icons";
import { RegistryProperty } from "@/api/sdf/model/node";

export default Vue.extend({
  name: "ValidationWidget",
  components: {
    AlertTriangleIcon,
  },
  props: {
    value: [String, Number, Object, Array, Boolean],
    entityProperty: Object as () => RegistryProperty,
  },
  computed: {
    validation(): Joi.ValidationResult {
      const validation = this.entityProperty.prop
        .validation()
        .validate(this.value);
      return validation;
    },
    errors(): Joi.ValidationErrorItem[] {
      if (this.validation?.error) {
        return this.validation.error.details;
      } else {
        return [];
      }
    },
  },
});
</script>
