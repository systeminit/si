<template>
  <div class="flex flex-col w-full h-full editor">
    <PanelTree />
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import PanelTree from "@/organisims/PanelTree.vue";
import { applicationId$ } from "@/observables";

export interface IEditorContext {
  applicationId: string;
  contextType: "applicationSystem";
}

export default Vue.extend({
  name: "Editor",
  props: {
    context: {
      type: Object as PropType<IEditorContext>,
    },
  },
  components: {
    PanelTree,
  },
  created() {
    applicationId$.next(this.context.applicationId);
  },
});
</script>

<style scoped>
.editor {
  border-top: 1px solid #242424;
}
</style>
