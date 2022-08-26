<template>
  <ul v-if="props.errors.length">
    <li
      v-for="(error, index) in props.errors"
      :key="index"
      class="text-destructive-400"
    >
      <div class="flex flex-row items-center">
        <VueFeather
          type="alert-triangle"
          :class="strokeColorForLevel(error.level)"
          size="1.0em"
        />
        <div class="ml-1 text-xs" :class="strokeColorForLevel(error.level)">
          {{ error.message }}
        </div>
        <div v-if="error.link" class="ml-1 align-top">
          <SiLink :uri="error.link" blank-target>
            <VueFeather type="external-link" stroke="grey" size="1.0em" />
          </SiLink>
        </div>
      </div>
    </li>
  </ul>
</template>

<script setup lang="ts">
import VueFeather from "vue-feather";
import SiLink from "@/atoms/SiLink.vue";
import type { ValidationErrors } from "@/api/sdf/dal/edit_field";

const props = defineProps<{
  errors: ValidationErrors;
}>();

const strokeColorForLevel = (errorLevel: string | undefined): string => {
  if (errorLevel === undefined) {
    return "text-destructive-400";
  }
  switch (errorLevel) {
    case "warning":
      return "text-warning-400";
    case "info":
      return "text-neutral-400";
    default:
      return "text-destructive-400";
  }
};
</script>
