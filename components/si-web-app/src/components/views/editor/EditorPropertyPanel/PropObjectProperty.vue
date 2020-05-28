<template>

<!-- eslint-disable vue/no-unused-components -->

  <div class="w-full">
    <div class="pl-1 py-1">
      
      <div v-if="propObjectProperty.repeated">

        <div class="px-2 text-sm text-gray-400">
          {{ propObjectProperty.name }}
        </div>

        <div v-if="propObjectProperty.kind() == 'link'">

          <div v-if="propObjectProperty.lookupMyself().kind() == 'object'">
            <div ref="container" class="flex flex-col">
              <button class="text-red-500 text-center w-4 focus:outline-none" type="button" @click="onClickB($event, propObjectProperty)">                
                <plus-square-icon size="1.25x" class=""></plus-square-icon>
              </button>

              <ul>
                <li v-for="(object, index) in objectModel" :key="index">             
                  <PropObject
                    :propObject="propObjectProperty.lookupMyself()"
                    :propObjectModel="objectModel[index]"
                  />
                </li>
              </ul>

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

        <div v-else-if="propObjectProperty.kind() == 'object'">
          <button class="text-green-500 text-center w-4 focus:outline-none" type="button" @click="onClickB($event, propObjectProperty)">   
            <plus-square-icon size="1.25x" class=""></plus-square-icon>
          </button>

          <!-- Not working -->
          <ul>
            <li v-for="(object) in objectModel" :key="object.name">       
<!--               <PropObject
                :propObject="propObjectProperty.lookupMyself()"
                :propObjectModel="objectModel[index][propObjectProperty.name]"
              /> -->
            </li>
          </ul>

        </div>

        <div v-else-if="propObjectProperty.kind() == 'text'">
          <div class="flex items-center">
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
            <div class="px-2 text-sm text-gray-400">
              {{ propObjectProperty.name }}
            </div>
          </div>
        </div>

        <div v-else-if="propObjectProperty.kind() == 'number'">
          <div class="flex items-center">
            <div class="px-2 text-sm text-gray-400">
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
            class="block appearance-none bg-gray-200 border border-gray-200 text-gray-700 px-4 rounded leading-tight focus:outline-none "
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
        </div>
      
      </div>
      
      <div v-else-if="propObjectProperty.kind() == 'text'">

        <div class="flex items-center">

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
          <div class="px-2 text-sm text-gray-400">
            {{ propObjectProperty.name }}
          </div>
        </div>
      </div>

      <div v-else-if="propObjectProperty.kind() == 'number'">
        <div class="flex items-center">
          
          <div class="px-2 text-sm text-gray-400">
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
          class="block appearance-none bg-gray-200 border border-red-200 text-gray-700 px-4 rounded leading-tight focus:outline-none "
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

      <!-- A map has some number of Key/Value pairs. -->
      <div v-else-if="propObjectProperty.kind() == 'map'">
        
        <div class="flex flex-col">
            <button class="text-blue-500 text-center w-4 focus:outline-none" type="button" @click="onClickB($event, propObjectProperty)">
              <plus-square-icon size="1.25x" class=""></plus-square-icon>
            </button>

            <ul>

              <li v-for="(object, index) in objectModel" :key="index">  
                <input
                  class="appearance-none input-bg-color border-none text-gray-400 ml-3 pl-2 h-5 text-sm leading-tight focus:outline-none"
                  type="text"
                  :aria-label="key"
                  v-model="objectModel[index].key"
                  placeholder="key"
                  @change="onMetadataKeyChange($event, objectModel[index].key)"
                />

                <input
                  class="appearance-none input-bg-color border-none text-gray-400 ml-3 pl-2 h-5 text-sm leading-tight focus:outline-none"
                  type="text"
                  :aria-label="val"
                  v-model="objectModel[index].value"
                  placeholder="value"
                />
              </li>

            </ul>

        </div>
      </div>

      <div v-else-if="propObjectProperty.kind() == 'link'">
        <div v-if="propObjectProperty.lookupMyself().kind() == 'object'">

          <div class="flex flex-col"> 
            
            <div class="flex pl-2 text-sm text-white property-title-bg-color">
              <chevron-down-icon size="1.5x" class="custom-class"></chevron-down-icon>
              {{ propObjectProperty.label }}
            </div>

            <PropObject
              :propObject="propObjectProperty.lookupMyself()"
              :propObjectModel="objectModel"
            />

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

      <div v-else-if="propObjectProperty.kind() == 'object'">
        <PropObject
          :propObject="propObjectProperty"
          :propObjectModel="objectModel"
        />
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
import { PlusSquareIcon, ChevronDownIcon } from "vue-feather-icons"

import Button  from "./Button.vue"

import PropObject from "./PropObject.vue";

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
    onClickB(event, propObjectProperty) {
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

          // let keyValueMap = new Map()
          // keyValueMap.set('key', '')
          // keyValueMap.set('value', '')

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

</style>
