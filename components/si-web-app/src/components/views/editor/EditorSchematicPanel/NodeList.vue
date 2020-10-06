<template>
  <div id="node-list">
    <div v-for="item in nodes" :key="item.id">
      <NodeObject :node="item" />
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { mapGetters, mapActions } from "vuex";
import { registry } from "si-registry";

import { RootStore } from "@/store";
import NodeObject from "./NodeObject.vue";

export default Vue.extend({
  name: "NodeList",
  components: {
    NodeObject,
  },
  computed: {
    nodes(): RootStore["editor"]["nodes"] {
      return this.$store.getters["editor/nodeList"];
    },
  },
});
</script>
