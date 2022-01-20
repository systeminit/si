<template>
  <div>
    <div v-if="props.result">
      <div class="flex flex-row">
        <div><VueFeather type="check" size="1.5rem" /></div>
        <div class="flex flex-row">
          <div>Qualification status: (</div>
          <span v-if="props.result.success">success</span>
          <span v-else>failure</span>
          <div>)</div>
        </div>
      </div>

      <div
        v-for="error in props.result.errors"
        :key="error.message"
        class="flex flex-col flex-grow"
      >
        <div v-if="!props.result.success">
          <div class="flex flex-row">
            <VueFeather type="alert-triangle" size="1.5rem" />
            <div>{{ error.message }}</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { QualificationResult } from "@/api/sdf/dal/qualification";
import VueFeather from "vue-feather";

const props = defineProps<{
  result: QualificationResult;
}>();
</script>
