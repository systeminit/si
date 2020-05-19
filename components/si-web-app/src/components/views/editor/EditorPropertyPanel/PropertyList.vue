<template>
  <div
    id="property-panel-list"
    class="flex-grow property-editor-bg-color h-full"
  >
    <!-- <div v-for="field of kubernetesDeploymentEntity.fields.attrs" v-bind:key="field.name"> -->

    <table class="table-auto" v-on:keyup="onKeyUp">
      <tr>
        <th class="w-1/2 px-4 py-2"></th>
        <th class="w-1/4 px-4 py-2"></th>
        <th class="w-1/4 px-4 py-2"></th>
      </tr>

      <tbody
        v-for="field of kubernetesDeploymentEntityCreate.request.properties.attrs.filter(
          i => !i.hidden,
        )"
        v-bind:key="field.name"
      >
        <tr>
          <td class="text-right px-2 py-2 text-gray-400">{{ field.name }}</td>

          <td class="px-1 py-2 group-hover:border-teal-500">
            <div v-if="field.kind() == 'text'">
              <input
                class="appearance-none input-bg-color border-none w-full text-gray-400 mr-3 py-1 px-2 leading-tight focus:outline-none"
                type="text"
                :aria-label="field.name"
                v-model="kubernetesDeploymentEntityCreateVars[field.name]"
                placeholder="text"
              />
            </div>

            <div v-else-if="field.kind() == 'link'">
              <input
                class="appearance-none input-bg-color border-none w-full text-gray-400 mr-3 py-1 px-2 leading-tight focus:outline-none"
                type="text"
                :aria-label="field.name"
                v-model="kubernetesDeploymentEntityCreateVars[field.name]"
                placeholder="link"
              />
            </div>

            <div v-else-if="field.kind() == 'object'">
              <input
                class="appearance-none input-bg-color border-none w-full text-gray-400 mr-3 py-1 px-2 leading-tight focus:outline-none"
                type="text"
                :aria-label="field.name"
                v-model="kubernetesDeploymentEntityCreateVars[field.name]"
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
    <div class="flex flex-row-reverse pr-8 pb-4">
      <button
        class="bg-teal-700 px-4 py-2 text-white hover:bg-teal-600"
        @click="createEntity()"
        type="button"
      >
        Create
      </button>
    </div>
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

import { LinkIcon } from "vue-feather-icons";

import { auth } from "@/auth";

export default {
  name: "PropertyList",
  components: {
    LinkIcon,
  },
  mounted() {
    const kubernetesDeploymentEntity = registry.get(
      "kubernetesDeploymentEntity",
    );
    let i;

    console.log(kubernetesDeploymentEntity.fields.attrs);
    for (i = 0; i < kubernetesDeploymentEntity.length; i++) {
      console.log("AA");
      console.log(kubernetesDeploymentEntity[i]);
    }
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
    console.log(kubernetesDeploymentEntityCreateVars);

    return {
      kubernetesDeploymentEntity,
      kubernetesDeploymentEntityCreate,
      kubernetesDeploymentEntityCreateVars,
    };
  },
  methods: {
    createEntity() {
      const mutation = this.kubernetesDeploymentEntity.graphql.mutation({
        methodName: "create",
      });
      console.log(mutation);
      try {
        this.$apollo.mutate({
          mutation,
          variables: {
            name: this.kubernetesDeploymentEntityCreateVars.name,
            displayName: this.kubernetesDeploymentEntityCreateVars.displayName,
            description: this.kubernetesDeploymentEntityCreateVars.description,
            workspaceId: auth.getProfile().workspaceDefault.id,
            properties: {
              kubernetesObject: {
                kind: "your butt",
                apiVersion: "1.0",
              },
            },
            constraints: {
              kubernetesVersion: "V1_15",
            },
          },
        });
      } catch (error) {
        console.log("not today, homie", { error });
      }
    },
    onKeyUp(event) {
      if (event.key == "Enter") {
        console.log("Enter was pressed");
        console.log(event);
        console.log(event.target);
        console.log(event.target["aria-label"]);
      }
    },
  },
};
</script>

<style>
.property-editor-bg-color {
  background-color: #212324;
}

.input-bg-color {
  background-color: #25788a;
}
</style>
