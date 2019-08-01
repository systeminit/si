<template>
  <StandardLayout>
    <GraphEditor />
  </StandardLayout>
</template>

<script>
import Vue from "vue";
import StandardLayout from "@/components/StandardLayout.vue";
import GraphEditor from "@/components/GraphEditor.vue";

export default Vue.extend({
  name: "Graph",
  components: {
    StandardLayout,
    GraphEditor,
  },
});
</script>
