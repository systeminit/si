<template>
  <div>
    <div class="px-1 py-2 group-hover:border-teal-500">
      <div v-if="propObjectProperty.repeated">
        <button
          class="bg-teal-700 px-4 py-2 text-white hover:bg-teal-600"
          type="button"
        >
          Add Item
        </button>
      </div>
      <div v-else-if="propObjectProperty.kind() == 'text'">
        <input
          class="appearance-none input-bg-color border-none text-gray-400 mr-3 py-1 px-2 leading-tight focus:outline-none"
          type="text"
          :aria-label="propObjectProperty.name"
          v-model="objectModel"
          placeholder="text"
        />
      </div>

      <div v-else-if="propObjectProperty.kind() == 'code'">
        Derived!
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
        <button
          class="bg-teal-700 px-4 py-2 text-white hover:bg-teal-600"
          type="button"
        >
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
          <PropObject
            :propObject="propObjectProperty.lookupMyself()"
            :propObjectModel="objectModel"
          />
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

    <div class="text-left px-2 py-2">
      <link-icon size="1x" class="text-left text-white"></link-icon>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { registry } from "si-registry";

import { LinkIcon } from "vue-feather-icons";

import { auth } from "@/auth";

//import PropObject from "./PropObject.vue";
//console.log("you didn't get a real propobject", { PropObject });

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
    LinkIcon,
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
