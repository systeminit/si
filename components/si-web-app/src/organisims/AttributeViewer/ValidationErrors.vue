<template>
  <div v-if="errors.length" class="pt-2">
    <ul>
      <li v-for="(error, index) of errors" :key="index" class="text-red-400">
        <div class="flex items-center align-middle">
          <AlertTriangleIcon size="1x" />
          <div class="ml-2">
            {{ error.message }}
          </div>
          <div class="ml-2" v-if="error.link">
            <a target="_blank" :href="error.link">
              <ExternalLinkIcon size="1x" />
            </a>
          </div>
        </div>
      </li>
    </ul>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import { AlertTriangleIcon, ExternalLinkIcon } from "vue-feather-icons";
import { ValidateFailure } from "si-entity/dist/validation";

export default Vue.extend({
  name: "ValidationErrors",
  components: {
    AlertTriangleIcon,
    ExternalLinkIcon,
  },
  props: {
    errors: {
      type: Array as PropType<ValidateFailure["errors"]>,
      required: true,
    },
  },
});
</script>
