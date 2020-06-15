<template>
  <div id="property-panel-list" class="property-bg-color w-full h-full">
    <!--     <div class="mx-3">
      <button class="text-yellow-500 px-4 py-2 focus:outline-none" @click="onClick()" type="button">
        apollo.queries.kubernetesDeploymentEntityGet.refetch()
      </button>
    </div> -->

    <div v-if="kubernetesDeploymentEntityGet">
      <PropObject
        :propObject="kubernetesDeploymentEntity"
        :propObjectModel="kubernetesDeploymentEntityGet.item"
      />
    </div>
  </div>
</template>

<script>
import { registry } from "si-registry";
import PropObject from "./PropObject.vue";

export default {
  name: "PropertyListView",
  components: {
    PropObject,
  },
  props: {
    nodeId: String, // make this more generic later...
  },
  data() {
    // const kubernetesDeploymentEntity = registry.get("kubernetesDeployment")
    const kubernetesDeploymentEntity = {
      properties: registry.get("kubernetesDeploymentEntity").fields,
    };
    return {
      kubernetesDeploymentEntity,
      kubernetesDeploymentEntityGet: {
        item: {},
      },
    };
  },
  methods: {
    onClick() {
      this.$apollo.queries.kubernetesDeploymentEntityGet.refetch();
    },
  },
  apollo: {
    kubernetesDeploymentEntityGet: {
      query() {
        console.log("PropertyListView.kubernetesDeploymentEntityGet.query()");
        let result = registry
          .get("kubernetesDeploymentEntity")
          .graphql.query({ methodName: "get" });
        return result;
      },
      fetchPolicy: "no-cache",
      // fetchPolicy: "cache-first",
      variables() {
        return {
          id: this.nodeId, // this.nodeId,
        };
      },
      // update(data) {
      //   return data.kubernetesDeploymentEntityGet
      // },
    },
  },
  mounted() {
    console.log("PropertyListView.mounted()");
  },
  updated() {
    console.log("PropertyListView.updated()");
  },
};
</script>

<style scoped>
.property-bg-color {
  background-color: #212324;
}

.property-title-bg-color {
  background-color: #292c2d;
}
</style>
