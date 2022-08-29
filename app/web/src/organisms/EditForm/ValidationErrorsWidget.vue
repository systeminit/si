<template>
  <ul v-if="props.errors.length">
    <li
      v-for="(error, index) in props.errors"
      :key="index"
      class="text-destructive-400"
    >
      <div
        class="flex flex-row items-center"
        :class="strokeColorForLevel(error.level)"
      >
        <Icon name="alert-triangle" />
        <div class="ml-1 text-xs">
          {{ error.message }}
        </div>
        <div v-if="error.link" class="ml-1 align-top">
          <SiLink :uri="error.link" blank-target>
            <Icon name="link" />
          </SiLink>
        </div>
      </div>
    </li>
  </ul>
</template>

<script setup lang="ts">
import SiLink from "@/atoms/SiLink.vue";
import Icon from "@/ui-lib/Icon.vue";
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
