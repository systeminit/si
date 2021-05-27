<template>
  <div class="flex flex-col w-full h-full overflow-hidden">
    <div class="flex flex-col w-full h-full overflow-auto overscroll-none">
      <CodeMirror
        class="w-full h-ful"
        :value="jsonString"
        readOnly
        mode="json"
      />
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import { Resource } from "@/api/sdf/model/resource";
import CodeMirror from "@/molecules/CodeMirror.vue";

export default Vue.extend({
  name: "ResourceVisualization",
  components: {
    CodeMirror,
  },
  props: {
    resource: {
      type: Object as PropType<Resource>,
    },
  },
  computed: {
    jsonString(): string {
      return JSON.stringify(this.resource, null, "\t");
    },
    jsonMode(): Object {
      return {
        name: "javascript",
        json: "true",
      };
    },
  },
});
</script>
