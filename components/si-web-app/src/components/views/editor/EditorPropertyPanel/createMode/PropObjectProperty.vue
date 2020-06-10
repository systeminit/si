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

                <div v-for="(object, index) in objectModel" :key="index" class="flex pt-1">             
                  <PropObject
                    :propObject="propObjectProperty.lookupMyself()"
                    :propObjectModel="objectModel[index]"
                  />
                </div>

                <div class="flex text-gray-500 pt-1">
                  
                  <button class="text-center w-4 focus:outline-none" type="button" @click="onClickB($event, propObjectProperty)">          
                    <plus-square-icon size="1.25x" class=""></plus-square-icon>
                  </button>
                  
                  <p class="ml-2">{{ propObjectProperty.name }}</p>
                </div>

              </div>
            
            </div>

          </div>

          <div v-else>
            <PropObjectProperty
              :propObject="propObject"
              :propObjectProperty="propObjectProperty.lookupMyself()"
              :propObjectPropertyModel="objectModel"
            />
          </div>

        </div>

<!--         <div v-else-if="propObjectProperty.kind() == 'object'">
          <button class="text-yellow-500 text-center w-4 focus:outline-none" type="button" @click="onClickB($event, propObjectProperty)">   
            <plus-square-icon size="1.25x" class=""></plus-square-icon>
          </button>
        </div>

        <div v-else-if="propObjectProperty.kind() == 'text'">
          <div class="flex items-center text-yellow-500">
            <div class="px-2 text-sm text-gray-400">
              {{ propObjectProperty.name }}
            </div>
            <input
              class="appearance-none input-bg-color border-none text-gray-400 pl-2 h-5 text-sm leading-tight focus:outline-none"
              type="text"
              :aria-label="propObjectProperty.name"
              v-model="objectModel"
              placeholder="text"
            />
          </div>
        </div>

        <div v-else-if="propObjectProperty.kind() == 'code'">
          <div class="flex items-center">
            <div class="px-2 text-sm text-yellow-500">
              {{ propObjectProperty.name }}
            </div>
          </div>
        </div>

        <div v-else-if="propObjectProperty.kind() == 'number'">
          <div class="flex items-center">
            <div class="px-2 text-sm text-yellow-500">
              {{ propObjectProperty.name }}
            </div>
            <input
              class="appearance-none input-bg-color border-none text-gray-400 ml-3 pl-2 h-5 text-sm leading-tight focus:outline-none"
              type="text"
              :aria-label="propObjectProperty.name"
              v-model="objectModel"
              placeholder="number"
            />
          </div>
        </div>

        <div v-else-if="propObjectProperty.kind() == 'enum'">
          <select 
            class="block appearance-none bg-gray-200 border border-gray-200 text-yellow-500 px-4 rounded leading-tight focus:outline-none "
            :aria-label="propObjectProperty.name">
            <option
              v-for="option in propObjectProperty.variants"
              v-bind:key="option"
              >{{ option }}</option
            >
          </select>
        </div>

        <div v-else-if="propObjectProperty.kind() == 'map'">
          <div class="flex items-center">

            <button class="text-yellow-500 text-center w-4" type="button" @click="onClickB(propObjectProperty)">                
              <plus-square-icon size="1.25x" class=""></plus-square-icon>
            </button>
   
            <input
              class="appearance-none input-bg-color border-none text-gray-400 ml-3 pl-2 h-5 text-sm leading-tight focus:outline-none"
              type="text"
              :aria-label="propObjectProperty.name + ' key'"
              v-model="objectModel"
              placeholder="text"
            />
            
            <input
              class="appearance-none input-bg-color border-none text-gray-400 ml-3 pl-2 h-5 text-sm leading-tight focus:outline-none"
              type="text"
              :aria-label="propObjectProperty.name + ' value'"
              v-model="objectModel"
              placeholder="text"
            />
          </div>
        </div> -->
      
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
            v-model="objectModel"
            placeholder="text"
          />

<!--           <div v-if="!propObjectProperty.required">
            <button class="text-red-600 pl-1 focus:outline-none" type="button" @click="removeItem($event, index)">
              <x-icon size="0.8x"></x-icon>
            </button>
          </div> -->

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

<!-- 
  type="text"
  type="number"
  v-model.number="objectModel" below isn't wokring because the api want string.
  v-model="objectModel"
-->  
<!-- TYPE SHOULD BE NUMBER... but gql api doesn't seem to like numbers on numbers -->
          <input
            class="appearance-none text-sm leading-tight focus focus:outline-none input-bg-color border-none font-bold text-red-700 pl-2 h-5 w-32"
            type="number"
            :aria-label="propObjectProperty.name"
            v-model.number="objectModel"
            placeholder="number"
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
            v-model="objectModel"
            @change="formatSelector"
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
                <div v-for="(object, index) in objectModel" :key="index" class="flex pb-2">  
                  <input
                    class="appearance-none text-sm leading-tight focus:outline-none input-bg-color appearance-none border-none text-gray-400 pl-2 h-5 w-32"
                    type="text"
                    aria-label="key"
                    v-model="objectModel[index].key"
                    placeholder="key"
                    @change="onMetadataKeyChange($event, objectModel[index].key)"
                  />

                  <input
                    class="appearance-none text-sm leading-tight focus:outline-none input-bg-color appearance-none border-none text-gray-400 pl-2 ml-2 h-5 w-32"
                    type="text"
                    aria-label="val"
                    v-model="objectModel[index].value"
                    placeholder="value"
                  />

                  <button class="text-gray-600 pl-1 focus:outline-none" type="button" @click="removeItem($event, objectModel, index)">
                    <x-icon size="0.8x"></x-icon>
                  </button>

                </div>

              <div class="flex text-gray-500">
                <button class="focus:outline-none" type="button" @click="onClickB($event, propObjectProperty)">
                  <plus-square-icon size="1.25x"></plus-square-icon>
                </button>
              </div>

<!--               <div v-if="!propObjectProperty.required">
                <button class="text-gray-600 pl-1 focus:outline-none" type="button" @click="removeProperty(propObjectProperty)">
                  <x-icon size="0.8x"></x-icon>
                </button>
              </div> -->

            </div>

        </div>
      </div>
      
      <div v-else-if="propObjectProperty.kind() == 'link'">

        <div v-if="propObjectProperty.lookupMyself().kind() == 'object'">

          <PropertySection
            class="flex flex-col"
            :sectionTitle="propObjectProperty.name"
            > 

            <PropObject
              :propObject="propObjectProperty.lookupMyself()"
              :propObjectModel="objectModel"
            />

          </PropertySection>
        </div>

        <div v-else>
          <PropObjectProperty
            :propObject="propObject"
            :propObjectProperty="propObjectProperty.lookupMyself()"
            :propObjectPropertyModel="objectModel"
          />
        </div>
      </div>

      <div v-else-if="propObjectProperty.kind() == 'object'">
          <PropertySection
            class="flex flex-col"
            :sectionTitle="propObjectProperty.name"
            > 

            <PropObject
              :propObject="propObjectProperty"
              :propObjectModel="objectModel"
            />

          </PropertySection>
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
import { PlusSquareIcon, ChevronDownIcon, Trash2Icon, XIcon } from "vue-feather-icons"

import Button  from "./Button.vue"

import PropObject from "./PropObject.vue";
import PropertySection from "./PropertySection.vue";

import { constantCase } from "change-case";

// @ts-ignore
export default Vue.extend({
  name: "PropObjectProperty",
  props: {
    propObject: { type: Object, required: true },
    propObjectProperty: { type: Object, required: true },
    propObjectPropertyModel: {
      type: [Object, String, Number, Array],
      required: true,
    },
  },
  components: {
    PlusSquareIcon,
    ChevronDownIcon,
    PropertySection,
    Trash2Icon,
    XIcon,
    PropObject: () => import("./PropObject.vue"),
  },
  data() {
    const kubernetesMetadata = registry.get(
      "kubernetesMetadata",
    );

    return {
      objectModel: this.propObjectPropertyModel,
      items: []
    };
  },
  watch: {
    objectModel(newVal, _oldVal) {
      console.log("PropObjectProperty.watch.objectModel() with ::",this.propObjectProperty.name, newVal)
      this.$emit("propChangeMsg", {
        fieldName: this.propObjectProperty.name,
        value: newVal,
      });
    },
  },
  
  // watch: {
  //   objectModel(event){
  //     console.log("PropObjectProperty.watch(objectModel):", event)
  //   }
  // },

  methods: {
    formatSelector() {
      console.log("PropObjectProperty.methods.formatSelector()")

      console.log("aa with:", this.objectModel)
      // this.objectModel = constantCase(this.objectModel.replace(/\./g, '_'));
      this.objectModel = constantCase(this.objectModel);

    },
    // @ts-ignore
    onMetadataKeyChange(event, object) {
      // Add input validation - keys must be unique.
      console.log("Metadata key changed: ", event)
      console.log("Metadata Key value: ", object)

    },
    // @ts-ignore
    removeProperty(propObjectProperty) {
      console.log(propObjectProperty)
      if (!propObjectProperty.required) {
        console.log("not required", this.objectModel)
        // delete this.objectModel;
      } else {
        console.log("required")
      }
      // delete this.objectModel[propObjectProperty.name];
    },
    // @ts-ignore
    removeItem($event, index) {
      this.objectModel.splice(index, 1);
    },
    // @ts-ignore
    onClickB(event, propObjectProperty) {
      console.log("click")
      var varsObjectForProperty;
      var keyValueMapObject;

      switch (propObjectProperty.kind()) {
          case "link":

            console.log("we have a link")
            console.log("propObjectProperty:", propObjectProperty)
            console.log("objectModel:", this.objectModel)
            
            if (propObjectProperty.lookupMyself().kind() == "object") {
              
              console.log("we have a link and it's an object")
              varsObjectForProperty = variablesObjectForProperty(propObjectProperty.lookupMyself(), true)
              console.log("varsObjectForProperty:", varsObjectForProperty)
              
            try {
              this.objectModel.push(varsObjectForProperty)
              console.log("length: ", this.objectModel.length)
            }
            catch(err) {
              console.log("err: ", err)
            }

          }
          break;
        
        case "object":
          console.log("we have an object")
          console.log("propObjectProperty:", propObjectProperty)
          console.log(this.objectModel)
          varsObjectForProperty = variablesObjectForProperty(propObjectProperty)
          console.log("varsObjectForProperty:", varsObjectForProperty)
          
          keyValueMapObject = {
            [varsObjectForProperty.name]: ''
          }

          console.log("keyValueMapObject: ", keyValueMapObject)
          this.objectModel.push(keyValueMapObject)
          

          break;

        case "map":
          console.log("we have a map")
          console.log("propObjectProperty:", propObjectProperty)
          console.log("objectModel:", this.objectModel)
          varsObjectForProperty = variablesObjectForProperty(propObjectProperty)
          console.log("varsObjectForProperty:", varsObjectForProperty)

          keyValueMapObject = {
            'key': '',
            'value': ''
          }

          this.objectModel.push(keyValueMapObject)

          break;

        }
    }
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
