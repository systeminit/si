<template>
  <ul v-if="props.errors.length">
    <li
      v-for="(error, index) in props.errors"
      :key="index"
      class="text-red-400"
    >
      <div class="flex flex-row items-center">
        <VueFeather
          type="alert-triangle"
          :stroke="strokeColorForLevel(error.level)"
          size="1.5rem"
        />
        <div class="ml-1 text-xs">
          {{ error.message }}
        </div>
        <div v-if="error.link" class="ml-1 align-top">
          <a target="_blank" :href="error.link">
            <VueFeather type="external-link" stroke="grey" size="1.5rem" />
          </a>
        </div>
      </div>
    </li>
  </ul>
</template>

<script setup lang="ts">
import VueFeather from "vue-feather";
import type { ValidationErrors } from "@/api/sdf/dal/edit_field";

const props = defineProps<{
  errors: ValidationErrors;
}>();

function strokeColorForLevel(level: string): string {
  switch (level) {
    case "warning":
      return "yellow";
    case "info":
      return "grey";
    default:
      return "red";
  }
}
</script>
