<template>
  <div class="w-full bg-red-500">
    <div class="pl-1 py-2">
      
      <div v-if="propObjectProperty.repeated">
        <button class="bg-teal-700 px-4 py-2 text-white" type="button"> Add Item </button>
      </div>
      
      <div v-else-if="propObjectProperty.kind() == 'text'">

        <div class="flex">
          <div class="px-2 py-2 text-gray-400">
            {{ propObjectProperty.name }}
          </div>

          <input
            class="appearance-none input-bg-color border-none text-gray-400 mr-3 py-1 px-2 leading-tight focus:outline-none"
            type="text"
            :aria-label="propObjectProperty.name"
            v-model="objectModel"
            placeholder="text"
          />
        </div>
      </div>

      <div v-else-if="propObjectProperty.kind() == 'code'">
        
        <div class="px-2 py-2 text-gray-400">
          {{ propObjectProperty.name }}
        </div>

        <p> Derived </p>

      </div>

      <div v-else-if="propObjectProperty.kind() == 'number'">
        <input
          class="appearance-none input-bg-color border-none text-gray-400 mr-3 py-1 px-2 leading-tight focus:outline-none"
          type="text"
          :aria-label="propObjectProperty.name"
          v-model="objectModel"
          placeholder="number"
        />
      </div>

      <div v-else-if="propObjectProperty.kind() == 'enum'">
        
        <select :aria-label="propObjectProperty.name">
          <option
            v-for="option in propObjectProperty.variants"
            v-bind:key="option"
            >{{ option }}</option
          >
        </select>

      </div>

      <!-- A map has some number of Key/Value pairs. -->
      <div v-else-if="propObjectProperty.kind() == 'map'">
        
        <button class="bg-teal-700 px-4 py-2 text-white hover:bg-teal-600" type="button">
          Add Row
        </button>
        
        <input
          class="appearance-none input-bg-color border-none text-gray-400 mr-3 py-1 px-2 leading-tight focus:outline-none"
          type="text"
          :aria-label="propObjectProperty.name + ' key'"
          v-model="objectModel"
          placeholder="text"
        />
        
        <input
          class="appearance-none input-bg-color border-none text-gray-400 mr-3 py-1 px-2 leading-tight focus:outline-none"
          type="text"
          :aria-label="propObjectProperty.name + ' value'"
          v-model="objectModel"
          placeholder="text"
        />

      </div>

      <div v-else-if="propObjectProperty.kind() == 'link'">
        <div v-if="propObjectProperty.lookupMyself().kind() == 'object'">

          <div class="flex flex-col">
            
            <div class="pl-2 py-2 text-white bg-gray-800">
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
import Vue from "vue";
import { registry } from "si-registry";
import { auth } from "@/utils/auth";

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

<style>
.property-editor-bg-color {
  background-color: #212324;
}

.input-bg-color {
  background-color: #25788a;
}
</style>
