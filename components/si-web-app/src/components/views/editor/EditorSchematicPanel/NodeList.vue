<template>
  <div id="node-list">
    <div v-for="item in nodeList" :key="item.id">
      <NodeObject :nodeObject="item" />
    </div>
  </div>
</template>

<script>
import { registry } from "si-registry";

import NodeObject from "./NodeObject.vue";

import { mapGetters, mapActions } from "vuex";

export default {
  name: "NodeList",
  components: {
    NodeObject,
  },
  computed: {
    ...mapGetters({
      nodeList: "node/list",
    }),
  },
};
</script>
