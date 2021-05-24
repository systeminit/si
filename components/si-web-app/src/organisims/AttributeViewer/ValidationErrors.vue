<template>
  <div v-if="errors.length" class="">
    <ul>
      <li v-for="(error, index) of errors" :key="index" class="text-red-400">
        <div class="flex flex-row items-center">
          <AlertTriangleIcon size="1x" />
          <div class="ml-1 text-xs">
            {{ error.message }}
          </div>
          <div class="ml-1 align-top" v-if="error.link">
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
