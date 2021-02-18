<template>
  <div class="flex flex-col h-full w-full">
    <PanelTree />
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import PanelTree from "@/organisims/PanelTree.vue";
import { IEditorContext, setupEditor } from "@/store/modules/editor";

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
    setupEditor();
  },
  async mounted() {
    await this.$store.dispatch("editor/setContext", this.context);
  },
  watch: {
    async context(newContext: IEditorContext) {
      await this.$store.dispatch("editor/setContext", newContext);
    },
  },
});
</script>
