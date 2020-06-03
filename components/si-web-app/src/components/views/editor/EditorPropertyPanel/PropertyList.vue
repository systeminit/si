<template>
  <!-- eslint-disable vue/no-unused-components -->
  <div id="property-panel-list" class="w-full h-full">
    
    <PropObject
      :propObject="kubernetesDeploymentEntityCreate.request"
      :propObjectModel="kubernetesDeploymentEntityCreateVars"
      @propChangeMsg="propChangeMsg"
    />

    <div class="flex flex-row-reverse pr-8 pb-4">
      <button class="bg-teal-700 px-4 py-2 text-white hover:bg-teal-600" @click="createEntity()" type="button">
        Create
      </button>
    </div>

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
      console.log("PropertyList.methods.propChangeMsg() with:", event["value"])
      this.kubernetesDeploymentEntityCreateVars = event["value"];
    },
    createEntity() {
      console.log("PropertyList.methods.createEntity()")
      const mutation = this.kubernetesDeploymentEntity.graphql.mutation({
        methodName: "create",
      });
      // console.log(mutation);

      try {
        console.log("PropertyList.methods.createEntity() with:", this.kubernetesDeploymentEntityCreateVars)
        // pass object to mutate here
        
        this.kubernetesDeploymentEntityCreateVars.workspaceId = auth.getProfile().workspaceDefault.id

        delete this.kubernetesDeploymentEntityCreateVars.properties.kubernetesObjectYaml


        // let targetVariables = {
        //   name: "motherLoveBone899",
        //   displayName: "Mother Love Bone",
        //   description: "Mother Love Bone",
        //   changeSetId: "",
        //   workspaceId: auth.getProfile().workspaceDefault.id,
        //   properties: {
        //     kubernetesObject: {
        //       apiVersion: "rr",
        //       kind: "rr",
        //       metadata: { name: "", labels: [] },
        //       spec: {
        //         replicas: 44,
        //         selector: { matchLabels: [] },
        //         template: {
        //           metadata: { name: "", labels: [] },
        //           spec: { containers: [] },
        //         },
        //       },
        //     },
        //   },
        //   constraints: {
        //     componentName: "",
        //     componentDisplayName: "",
        //     kubernetesVersion: "V1_15",
        //   },
        // }

        // console.log(console.log(JSON.stringify(this.kubernetesDeploymentEntityCreateVars)))
        // console.log("and")
        // console.log(console.log(JSON.stringify(targetVariables)))


        this.$apollo.mutate({
          mutation,
          variables: this.kubernetesDeploymentEntityCreateVars
        });


      } catch (error) {
        console.log("not today, homie", { error });
      }

      console.log("done");
    },
  }
};
</script>