<template>
  <!-- eslint-disable vue/no-unused-components -->
  <div id="node-list">
    
    <div v-for="item in nodeList" :key="item.id">
      
      <div v-if="itemIsKubernetesEntity(item)">

      <NodeObject
        :nodeObject="item"
       />

      </div>
    </div>

  </div>
</template>

<script>
/* eslint-disable vue/no-unused-components */
import { auth } from "@/utils/auth";
import { registry } from "si-registry";

// @ts-ignore
import VueJsonPretty from "vue-json-pretty"
import NodeObject from "./NodeObject.vue"

import { mapState, mapActions } from 'vuex'

export default {
  name: "NodeList",
  components: {
    VueJsonPretty,
    NodeObject
  },
  methods: {
    itemIsKubernetesEntity(item) {
      // item.id.includes("kubernetes_deployment_entity:")
      return true
    }
  },
  data() {
    return {
      kubernetesDeploymentEntityList: {
        items: []
      }
    }
  },
  computed: {
    ...mapState({
      nodeList: state => state.editor.nodeList
    }),
  },
  watch: {
    nodeList (newState, previousState) {
      console.log("allo")
    }
  }
}
</script>