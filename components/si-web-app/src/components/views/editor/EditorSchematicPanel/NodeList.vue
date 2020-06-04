<template>
  <!-- eslint-disable vue/no-unused-components -->
  <div id="node-list">
    
    <div v-for="item in kubernetesDeploymentEntityList.items" :key="item.id">
      
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
  apollo: {
    kubernetesDeploymentEntityList: {
      query() {
        let result = registry.get("kubernetesDeploymentEntity").graphql.query({methodName: "list",});
        return result;
      },
      fetchPolicy: "no-cache",
      variables() {
        return {
          pageSize: "1000",
        }
      },
      // update(data) {
      //   console.log("NodeList.apollo.kubernetesDeploymentEntityGet.update()");
      //   console.log("my data: ", data)
      // }
    }
  }
}
</script>