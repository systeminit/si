<template>

<!-- eslint-disable vue/no-unused-components -->

  <div class="w-full">
    <div class="py-1">
      
      <div v-if="propObjectProperty.repeated">

        <div v-if="propObjectProperty.kind() == 'link'">

          <div v-if="propObjectProperty.lookupMyself().kind() == 'object'">
            
            <div class="flex flex-row">

              <div class="px-2 text-sm text-gray-400">
                {{ propObjectProperty.name }}
              </div>

              <div>

                <div v-for="(object, index) in propObjectPropertyModel" :key="index" class="flex pt-1">             
                  <PropObject
                    :propObject="propObjectProperty.lookupMyself()"
                    :propObjectModel="propObjectPropertyModel[index]"
                  />
                </div>

              </div>
            
            </div>

          </div>

          <div v-else>
            <PropObjectProperty
              :propObject="propObject"
              :propObjectProperty="propObjectProperty.lookupMyself()"
              :propObjectPropertyModel="propObjectPropertyModel"
            />
          </div>

        </div>
      
      </div>
      
      <div v-else-if="propObjectProperty.kind() == 'text'">

          <div class="flex">

          <div class="input-label">
            {{ propObjectProperty.name }}
          </div>

          <input
            class="appearance-none text-sm leading-tight focus:outline-none input-bg-color appearance-none border-none text-gray-400 pl-2 h-5 w-32"
            type="text"
            :aria-label="propObjectProperty.name"
            v-model="propObjectPropertyModel"
            placeholder="text"
            readonly
          />

        </div>
      </div>

      <div v-else-if="propObjectProperty.kind() == 'code'">
      <!-- do nothing -->
      </div>

      <div v-else-if="propObjectProperty.kind() == 'number'">
        <div class="flex items-center">

          <div class="input-label">
            {{ propObjectProperty.name }}
          </div>
 
          <input
            class="appearance-none text-sm leading-tight focus focus:outline-none input-bg-color border-none text-gray-400 pl-2 h-5 w-32"
            type="number"
            :aria-label="propObjectProperty.name"
            v-model.number="propObjectPropertyModel"
            placeholder="number"
            readonly
          />
        </div>
      </div>

      <div v-else-if="propObjectProperty.kind() == 'enum'">
        
        <div class="flex items-center">

          <div class="input-label">
            {{ propObjectProperty.name }}
          </div>

          <select 
            class="bg-gray-800 border text-gray-400 text-sm px-4 leading-tight focus:outline-none"
            :aria-label="propObjectProperty.name"
            v-model="propObjectPropertyModel"
            readonly
            >
            <option
              v-for="option in propObjectProperty.variants"
              v-bind:key="option"
              >{{ option }}</option
            >
          </select>

        </div>
      </div>

      <div v-else-if="propObjectProperty.kind() == 'map'">
        
        <div class="flex flex-row">

            <div class="input-label">
              {{propObjectProperty.name}}
             </div>

             <div>
                <div v-for="(object, index) in propObjectPropertyModel" :key="index" class="flex pb-2">  
                  <input
                    class="appearance-none text-sm leading-tight focus:outline-none input-bg-color appearance-none border-none text-gray-400 pl-2 h-5 w-32"
                    type="text"
                    :aria-label="key"
                    v-model="propObjectPropertyModel[index].key"
                    placeholder="key"
                    @change="onMetadataKeyChange($event, propObjectPropertyModel[index].key)"
                    readonly
                  />

                  <input
                    class="appearance-none text-sm leading-tight focus:outline-none input-bg-color appearance-none border-none text-gray-400 pl-2 ml-2 h-5 w-32"
                    type="text"
                    :aria-label="val"
                    v-model="propObjectPropertyModel[index].value"
                    placeholder="value"
                    readonly
                  />
                </div>
            </div>

        </div>
      </div>
      
      <div v-else-if="propObjectProperty.kind() == 'link'">

        <div v-if="propObjectProperty.lookupMyself().kind() == 'object'">

          <PropertySectionView
            class="flex flex-col"
            :sectionTitle="propObjectProperty.name"
            > 

            <PropObject
              :propObject="propObjectProperty.lookupMyself()"
              :propObjectModel="propObjectPropertyModel"
            />

          </PropertySectionView>
        </div>

        <div v-else>
          <PropObjectPropertyView
            :propObject="propObject"
            :propObjectProperty="propObjectProperty.lookupMyself()"
            :propObjectPropertyModel="propObjectPropertyModel"
          />
        </div>
      </div>

      <div v-else-if="propObjectProperty.kind() == 'object'">
          <PropertySectionView
            class="flex flex-col"
            :sectionTitle="propObjectProperty.name"
            > 

            <PropObject
              :propObject="propObjectProperty"
              :propObjectModel="propObjectPropertyModel"
            />

          </PropertySectionView>
      </div>

      <div v-else>
        Missing property {{ propObjectProperty.kind() }} for
        {{ propObjectProperty.name }}
      </div>
    </div>

  </div>
</template>

<script lang="ts">

/* eslint-disable vue/no-unused-components */
import Vue from "vue";
import { registry, variablesObjectForProperty } from "si-registry";
import { auth } from "@/utils/auth";
import { ChevronDownIcon, Trash2Icon, XIcon } from "vue-feather-icons"

import PropObject from "./PropObject.vue";
import PropertySectionView from "./PropertySectionView.vue";

// @ts-ignore
export default Vue.extend({
  name: "PropObjectPropertyView",
  props: {
    propObject: { type: Object, required: true },
    propObjectProperty: { type: Object, required: true },
    propObjectPropertyModel: {
      type: [Object, String, Number, Array],
      required: true,
    },
  },
  components: {
    ChevronDownIcon,
    PropertySectionView,
    Trash2Icon,
    XIcon,
    PropObject: () => import("./PropObject.vue"),
  },
  data() {
    const kubernetesMetadata = registry.get(
      "kubernetesMetadata",
    );

    return {
      items: []
    };
  },
});
</script>

<style scoped>
.property-editor-bg-color {
  background-color: #212324;
}

.property-title-bg-color {
  background-color: #292C2D;
}

.input-bg-color {
  background-color: #25788a;
}

.input-label {
  @apply pr-2 text-sm text-gray-400 text-right w-40
}

input[type=number]::-webkit-inner-spin-button, 
input[type=number]::-webkit-outer-spin-button { 
  -webkit-appearance: none; 
  margin: 0; 
}

</style>
