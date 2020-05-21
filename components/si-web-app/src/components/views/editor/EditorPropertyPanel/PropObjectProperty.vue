<template>

<!-- eslint-disable vue/no-unused-components -->

  <div class="w-full">
    <div class="pl-1 py-1">
      
      <div v-if="propObjectProperty.repeated">
        <button class="text-teal-700 text-center" type="button">
          <plus-square-icon size="1.25x" class="custom-class"></plus-square-icon>
        </button>

        <vue-json-pretty
          class="text-white"
          :path="'res'"
          :data="propObjectProperty">
        </vue-json-pretty>

        <div class="px-2 text-sm text-gray-400">
          {{ propObjectProperty.name }}
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
          class="block appearance-none bg-gray-200 border border-gray-200 text-gray-700 px-4 rounded leading-tight focus:outline-none "
          :aria-label="propObjectProperty.name">
          <option
            v-for="option in propObjectProperty.variants"
            v-bind:key="option"
            >{{ option }}</option
          >
        </select>

      </div>

      <!-- A map has some number of Key/Value pairs. -->
      <div v-else-if="propObjectProperty.kind() == 'map'">
        
        <div class="flex items-center">
          <button class="text-teal-700 text-center" type="button">
            <plus-square-icon size="1.25x" class="custom-class"></plus-square-icon>
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

      <div v-else-if="propObjectProperty.kind() == 'link'">
        <div v-if="propObjectProperty.lookupMyself().kind() == 'object'">

          <div class="flex flex-col"> 
            
            <div class="flex pl-2 text-sm text-white property-title-bg-color">
              <chevron-down-icon size="1.5x" class="custom-class"></chevron-down-icon>
              {{ propObjectProperty.name }}
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
import { registry } from "si-registry";
import { auth } from "@/utils/auth";
import { PlusSquareIcon, ChevronDownIcon } from "vue-feather-icons"

// @ts-ignore
import VueJsonPretty from "vue-json-pretty"

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
    VueJsonPretty,
    PropObject: () => import("./PropObject.vue"),
  },
  data() {
    return {
      objectModel: this.propObjectPropertyModel,
    };
  },
  watch: {
    objectModel(newVal, _oldVal) {
      this.$emit("propChangeMsg", {
        fieldName: this.propObjectProperty.name,
        value: newVal,
      });
    },
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
