<template>
  <!-- eslint-disable vue/no-unused-components -->
  <div id="property-panel-list" class="w-full h-full">
    
    <PropObjectView
      :propObject="kubernetesDeploymentEntity"
      :propObjectModel="kubernetesDeploymentEntityGet.item"
    />

<!--     <vue-json-pretty
      :path="'res'"
      :data="kubernetesDeploymentEntityGet.item"
      @click="handleClick">
    </vue-json-pretty> -->

  </div>
</template>

<script>
/* eslint-disable vue/no-unused-components */
import { auth } from "@/utils/auth";
import { registry } from "si-registry";
import PropObjectView from "./PropObjectView.vue";

// @ts-ignore
import VueJsonPretty from "vue-json-pretty"

export default {
  name: "PropertyListView",
  components: {
    PropObjectView,
    VueJsonPretty,
  },
  props: {
    nodeId: String // make this more generic later...
  },
  data() {

    // const kubernetesDeploymentEntity = registry.get("kubernetesDeployment")
    const kubernetesDeploymentEntity = {
      properties:  registry.get("kubernetesDeploymentEntity").fields
    }

    // const kubernetesDeploymentEntity = registry.get("kubernetesDeploymentEntity").graphql
    // const kubernetesDeploymentEntity = registry.get("kubernetesDeploymentEntity").graphql.systemObject.methodsProp
    // const kubernetesDeploymentEntity = registry.get("kubernetesDeploymentEntity").methods.getEntry("get")


    // const kubernetesDeploymentEntityCreate = registry.get("kubernetesDeploymentEntity").methods.getEntry("create");
    // const kubernetesDeploymentEntityCreateVars = registry.get("kubernetesDeploymentEntity").graphql.variablesObject(
    //   { methodName: "create" },
    // );
    return {
      kubernetesDeploymentEntity,
      // kubernetesDeploymentEntityCreate,
      // kubernetesDeploymentEntityCreateVars,
      kubernetesDeploymentEntityGet: {
        item: {}
      },
    };
  },
  methods: {
  },
  apollo: {
    kubernetesDeploymentEntityGet: {
      query() {
        console.log("query with: ", this.nodeId)
        let result = registry.get("kubernetesDeploymentEntity").graphql.query({methodName: "get"});
        return result;
      },
      fetchPolicy: "no-cache",
      variables() {
        return {
          id: "kubernetes_deployment_entity:f17c2635-ce32-4a17-857d-033d68b62ba7", // this.nodeId,
        }
      },
      // update(data) {
      //   this.viewData = data.item
      // }
    }
  }
};
</script>