<template>
  <!-- eslint-disable vue/no-unused-components -->
  <div id="property-panel-list" class="w-full h-full property-bg-color">
    
    <div class="flex flex-row-reverse pr-8 pb-4">
      <button class="bg-teal-700 px-4 py-2 text-white hover:bg-teal-600" @click="createEntity()" type="button">
        Create
      </button>
    </div>

    <PropObject
      :propObject="kubernetesDeploymentEntityCreate.request"
      :propObjectModel="kubernetesDeploymentEntityCreateVars"
      @propChangeMsg="propChangeMsg"
    />



  </div>
</template>

<script>
/* eslint-disable vue/no-unused-components */
import { auth } from "@/utils/auth";
import { registry } from "si-registry";
import PropObject from "./PropObject.vue";

// @ts-ignore
import VueJsonPretty from "vue-json-pretty"

export default {
  name: "PropertyList",
  props: {
    node: {}
  },
  components: {
    PropObject,
    VueJsonPretty,
  },
  data() {
    const kubernetesDeploymentEntity = registry.get(
      "kubernetesDeploymentEntity",
    );

    const kubernetesDeploymentEntityCreate = kubernetesDeploymentEntity.methods.getEntry(
      "create",
    );

    const kubernetesDeploymentEntityCreateVars = kubernetesDeploymentEntity.graphql.variablesObject(
      { methodName: "create" },
    );

    return {
      kubernetesDeploymentEntity,
      kubernetesDeploymentEntityCreate,
      kubernetesDeploymentEntityCreateVars,
    };
  },
  methods: {
    propChangeMsg(event) {
      this.kubernetesDeploymentEntityCreateVars = event["value"];
    },
    createEntity() {
      const mutation = this.kubernetesDeploymentEntity.graphql.mutation({
        methodName: "create",
      });
      // console.log(mutation);

      try {
        // pass object to mutate here
        
        this.kubernetesDeploymentEntityCreateVars.workspaceId = auth.getProfile().workspaceDefault.id

        delete this.kubernetesDeploymentEntityCreateVars.properties.kubernetesObjectYaml

        // We need to make sure the changesetId is valid, invalida changset ID is bad,...
        // Should error on invalid changeSetId.
        delete this.kubernetesDeploymentEntityCreateVars.changeSetId
  
        // DEBUG logging to confirm what we are passing to the mutation below
        // console.log("kubernetesDeploymentEntityCreateVars as json below:")
        // console.log(console.log(JSON.stringify(this.kubernetesDeploymentEntityCreateVars)))


        this.$apollo.mutate({
          mutation,
          variables: this.kubernetesDeploymentEntityCreateVars
        });

        // clearVueX cache
        this.$store.dispatch('editor/removeNode', this.node)


      } catch (error) {
        console.log("error", { error });
      }

      // console.log("done");
    },
  },
};
</script>

<style scoped>
.property-bg-color {
  background-color: #212324;
}

.property-title-bg-color {
  background-color: #292C2D;
}

</style>

