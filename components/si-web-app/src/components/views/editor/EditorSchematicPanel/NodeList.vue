<template>
  <!-- eslint-disable vue/no-unused-components -->
  <div id="node-list">
    

<!--     kubernetesDeploymentEntityList
    <div v-for="field of propObject.properties.attrs.filter(i => !i.hidden)" v-bind:key="field.name" class="flex flex-row">
      
      <PropObjectProperty
        :propObject="propObject"
        :propObjectProperty="field"
        :propObjectPropertyModel="objectModel[field.name]"
        @propChangeMsg="propChangeMsg"
      />
 -->
        <vue-json-pretty
        class ="text-white overflow-auto"
        :path="'res'"
        :data="kubernetesDeploymentEntityList"
        />

    <div v-for="item in kubernetesDeploymentEntityList.items" :key="item.id">
      
      {{item}}
      
      <div v-if="itemIsKubernetesEntity(item)">

<!--       <NodeObject
        :nodeObject="item"
       /> -->
       <vue-json-pretty
        class ="text-white overflow-auto"
        :path="'res'"
        :data="item"
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

        // KubernetesDeploymentEntityListRequest from fullstack-schema.graphql
        let result = registry.get("kubernetesDeploymentEntity").graphql.query({methodName: "list",});
        console.log("result with:", result)
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