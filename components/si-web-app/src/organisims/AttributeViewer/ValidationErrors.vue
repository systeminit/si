<template>
  <div v-if="errors.length" class="pt-2">
    <ul>
      <li v-for="(error, index) of errors" :key="index" class="text-red-400">
        <div class="flex items-center align-middle">
          <AlertTriangleIcon size="1x" />
          <div class="ml-2">
            {{ error }}
          </div>
        </div>
      </li>
    </ul>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import { AlertTriangleIcon } from "vue-feather-icons";

export default Vue.extend({
  name: "ValidationErrors",
  components: {
    AlertTriangleIcon,
  },
  props: {
    errors: {
      type: Array as PropType<string[]>,
      required: true,
    },
  },
});
</script>
