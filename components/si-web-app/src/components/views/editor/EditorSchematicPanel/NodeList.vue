<template>
  <div id="node-list">
    <div v-for="item in nodeList" :key="item.id">
      <div v-if="itemIsKubernetesEntity(item)">
        <NodeObject :nodeObject="item" />
      </div>
    </div>
  </div>
</template>

<script>
import { registry } from "si-registry";

// @ts-ignore
// import VueJsonPretty from "vue-json-pretty";
import NodeObject from "./NodeObject.vue";

import { mapGetters, mapActions } from "vuex";

export default {
  name: "NodeList",
  components: {
    NodeObject,
  },
  methods: {
    itemIsKubernetesEntity(item) {
      // item.id.includes("kubernetes_deployment_entity:")
      return true;
    },
  },
  data() {
    return {
      kubernetesDeploymentEntityList: {
        items: [],
      },
    };
  },
  computed: {
    ...mapGetters({
      nodeList: "editor/nodeList",
    }),
    //...mapState({
    //  nodeList: state => state.editor.nodeList,
    //}),
  },
  watch: {
    nodeList(newState, previousState) {
      console.log("allo");
    },
  },
};
</script>
