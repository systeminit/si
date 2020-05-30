<template>
  <!-- eslint-disable vue/no-unused-components -->
  <div>
    <div class="node draggable absolute">{{nodeObject.id}}</div>
  </div>
</template>

<script>
import { registry } from "si-registry";

export default {
  name: "NodeObject",
  props: {
    nodeObject: {},
  },
  data() {
    // const kubernetesDeploymentEntity = registry.get("kubernetesDeploymentEntity");

    // // const kubernetesDeploymentEntityObject = kubernetesDeploymentEntity.methods.getEntry("get");

    // const kubernetesDeploymentEntityObject = kubernetesDeploymentEntity.graphql.query({
    //   methodName: "get",
    //   associations: {
    //     id: this.nodeObject.id
    //   }
    // });
  
    return {
      kubernetesDeploymentEntityGet: {
        item: null
      }
    };
  },
  apollo: {
    kubernetesDeploymentEntityGet: {
      query() {
        // kubernetesDeploymentEntity
        // si storable.typeName

        return registry.get("kubernetesDeploymentEntity").graphql.query({methodName: "get",});
      },
      variables() {
        return {
          id: this.nodeObject.id,
        }
      },
    }
  }
}
// fullstack-schema.graphql
// NodeObject.vue
</script>

<style type="text/css" scoped>
.node {
  width: 140px;
  height: 100px;
  background-color: teal;
  color: #fff;
}
</style>