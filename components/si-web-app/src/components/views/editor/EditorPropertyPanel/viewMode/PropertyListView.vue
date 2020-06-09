<template>
  <!-- eslint-disable vue/no-unused-components -->
  <div id="property-panel-list" class="w-full h-full">
    
<!--     <div class="mx-3">
      <button class="text-yellow-500 px-4 py-2 focus:outline-none" @click="onClick()" type="button">
        apollo.queries.kubernetesDeploymentEntityGet.refetch()
      </button>
    </div> -->

    <PropObject
      :propObject="kubernetesDeploymentEntity"
      :propObjectModel="kubernetesDeploymentEntityGet.item"
    />

  </div>
</template>

<script>
/* eslint-disable vue/no-unused-components */
import { auth } from "@/utils/auth";
import { registry } from "si-registry";
import PropObject from "./PropObject.vue";

export default {
  name: "PropertyListView",
  components: {
    PropObject,
  },
  props: {
    nodeId: String // make this more generic later...
  },
  data() {
    // const kubernetesDeploymentEntity = registry.get("kubernetesDeployment")
    const kubernetesDeploymentEntity = {
      properties:  registry.get("kubernetesDeploymentEntity").fields
    }
    return {
      kubernetesDeploymentEntity,
      kubernetesDeploymentEntityGet: {
        item: {}
      },
    };
  },
  methods: {
    onClick() {
      this.$apollo.queries.kubernetesDeploymentEntityGet.refetch()
    }
  },
  apollo: {
    kubernetesDeploymentEntityGet: {
      query() {
        console.log("PropertyListView.kubernetesDeploymentEntityGet.query()")
        let result = registry.get("kubernetesDeploymentEntity").graphql.query({methodName: "get"});
        return result;
      },
      fetchPolicy: "no-cache",
      // fetchPolicy: "cache-first",
      variables() {
        return {
          id: this.nodeId, // this.nodeId,
        }
      },
      // update(data) {
      //   return data.kubernetesDeploymentEntityGet
      // },
    }
  },
  mounted() {
    console.log("PropertyListView.mounted()")
  },
  updated() {
    console.log("PropertyListView.updated()")
  }
};
</script>