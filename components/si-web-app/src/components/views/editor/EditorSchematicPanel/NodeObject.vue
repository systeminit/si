<template>
  <!-- eslint-disable vue/no-unused-components -->
  <div>
    <div :ref="entityId" class="node draggable absolute cursor-move border-solid">
      
      <div class="flex flex-col select-none">

        <div class="flex flex-col text-white ml-1 mt-1">
          <div class="font-light text-xs">name:</div>
          <div class="font-normal text-xs ml-2">{{entityName}}</div>
        </div>
      
      </div>

    </div>
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
      entityId: this.nodeObject.id,
      entityName: this.nodeObject.name
    };
  },
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