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

    <vue-json-pretty
      class="text-white text-lg"
      :path="'res'"
      :data="kubernetesDeploymentEntityCreateVars"
      @click="handleClick">
      :v-model="kubernetesDeploymentEntityCreateVars"
    </vue-json-pretty>

  </div>
</template>

<script>
/* eslint-disable vue/no-unused-components */
import { registry } from "si-registry";
import { auth } from "@/utils/auth";

import PropObject from "./PropObject.vue";

// @ts-ignore
import VueJsonPretty from "vue-json-pretty"

export default {
  name: "PropertyList",
  components: {
    //LinkIcon,
    PropObject,
    VueJsonPretty,
  },
  mounted() {},
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
        // this.$apollo.mutate({
        //   mutation,
        //   variables: {
        //     name: this.kubernetesDeploymentEntityCreateVars.name,
        //     displayName: this.kubernetesDeploymentEntityCreateVars.displayName,
        //     description: this.kubernetesDeploymentEntityCreateVars.description,
        //     workspaceId: auth.getProfile().workspaceDefault.id,
        //     properties: {
        //       kubernetesObject: {
        //         kind: "your butt",
        //         apiVersion: "1.0",
        //       },
        //     },
        //     constraints: {
        //       kubernetesVersion: "V1_15",
        //     },
        //   },
        // });

        console.log("PropertyList.methods.createEntity() with:", this.kubernetesDeploymentEntityCreateVars)
        // pass object to mutate here
        this.$apollo.mutate({
          mutation,
          variables: this.kubernetesDeploymentEntityCreateVars
      
        });


      } catch (error) {
        console.log("not today, homie", { error });
      }

      // try {
      //   console.log(this.kubernetesDeploymentEntityList);
      //   // let objE = registry.objects
      //   // let listTest = this.kubernetesDeploymentEntityList()

      //   // let KubernetesDeploymentEntityListRequest = registry.KubernetesDeploymentEntityListRequest()
      //   // let KubernetesDeploymentEntityList = this.kubernetesDeploymentEntity.methods.getEntry("list");

      //   // let KubernetesDeploymentEntityList = registry.KubernetesDeploymentEntityListKubernetesDeploymentEntityListRequest)
      //   // let listA = registry.get("kubernetesDeploymentEntityList", );
      //   // console.log(KubernetesDeploymentEntityList)
      // } catch (error) {
      //   console.log("oops", { error });
      // }

      console.log("done");
    },
    onKeyUp(event) {
      if (event.key == "Enter") {
        console.log("PropertyList.methods.onKeyUp() :: Enter");
        // console.log(event);
        // console.log(event.target);
        // console.log(event.target["aria-label"]);
      }
    },
  },
  apollo: {
    kubernetesDeploymentEntityList: {
      query() {
        console.log("PropertyList.apollo.kubernetesDeploymentEntityList.query()");
        return this.kubernetesDeploymentEntity.graphql.query({
          methodName: "list",
        });
      },
      variables() {
        console.log("PropertyList.apollo.kubernetesDeploymentEntityList.variables()");
        return {
          pageSize: "1000",
        };
      },
      update(data) {
        console.log("PropertyList.apollo.kubernetesDeploymentEntityList.update()");
      },
    },
    // kubernetesDeploymentEntityGet: {
    //   query() {
    //     return this.kubernetesDeploymentEntity.graphql.query({
    //       methodName:"get"
    //     });
    //   },
    //   variables(myVar) {
    //     return {
    //       pageSize: myVar
    //     }
    //   },
    //   update(data) {
    //     console.log(data)
    //     // this.me == data.what.I.want
    //   }
    // }
    // -> Item List... look into our item list
  },
};
</script>