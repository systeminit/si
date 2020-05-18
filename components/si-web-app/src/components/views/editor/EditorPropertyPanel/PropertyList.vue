<template>
  <div id="property-panel-list" class="flex-grow property-editor-bg-color h-full">

    <!-- <div v-for="field of kubernetesDeploymentEntity.fields.attrs" v-bind:key="field.name"> -->
      
    <table class="table-auto" v-on:keyup="onKeyUp">
    <tr>
      <th class="w-1/2 px-4 py-2"></th>
      <th class="w-1/4 px-4 py-2"></th>
      <th class="w-1/4 px-4 py-2"></th>
    </tr>

      <tbody v-for="field of kubernetesDeploymentEntity.fields.attrs" v-bind:key="field.name">
        <tr>
          <td class="text-right px-2 py-2 text-gray-400">{{field.name}}</td>
          
          <td class="px-1 py-2 group-hover:border-teal-500">
            <div v-if="field.kind() == 'text'">
              <input
                  class="appearance-none input-bg-color border-none w-full text-gray-400 mr-3 py-1 px-2 leading-tight focus:outline-none"
                  type="text"
                  :aria-label="field.name"
                  placeholder="text"
                />
            </div>

            <div v-else-if="field.kind() == 'link'">
              <input
                  class="appearance-none input-bg-color border-none w-full text-gray-400 mr-3 py-1 px-2 leading-tight focus:outline-none"
                  type="text"
                  :aria-label="field.name"
                  placeholder="link"
                />
            </div> 

            <div v-else-if="field.kind() == 'object'">
              <input
                  class="appearance-none input-bg-color border-none w-full text-gray-400 mr-3 py-1 px-2 leading-tight focus:outline-none"
                  type="text"
                  :aria-label="field.name"
                  placeholder="object"
                />
            </div> 

          </td>
          
          
          <td class="text-left px-2 py-2">
            <link-icon size="1x" class="text-left text-white"></link-icon>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>



<script>
import { registry } from "si-registry";
// console.log(k8sDeploymentE.fields.attrs)
// k8sDeploymentE.fields.attrs
// hidden on fields (render it or not)
// kind (text, objects, enums,...) - link kind... (resole link to show content by calling lookup() or some)
// field constraints, propObject.  
// create(), editingK8sProps and yaml.



import { LinkIcon } from 'vue-feather-icons'

export default {
  name: "PropertyList",
  components: {
    LinkIcon
  },
  mounted() {
    const kubernetesDeploymentEntity = registry.get("kubernetesDeploymentEntity");
    let i;

    console.log(kubernetesDeploymentEntity.fields.attrs)
    for (i = 0; i < kubernetesDeploymentEntity.length; i++) { 
        console.log("AA")
        console.log(kubernetesDeploymentEntity[i])
    }

  },
  data() {
    const kubernetesDeploymentEntity = registry.get("kubernetesDeploymentEntity");

    return {
      kubernetesDeploymentEntity,
    }
  },
  methods :{
    onKeyUp(event) {
      if(event.key == "Enter")
       {
         console.log("Enter was pressed");
         console.log(event);
         console.log(event.target);
         console.log(event.target.label);

       }

    }
  }
};
</script>

<style>

.property-editor-bg-color {
  background-color: #212324
}

.input-bg-color {
  background-color: #25788A
}

</style>