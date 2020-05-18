<template>
  <div id="property-panel-list">

<!--     <div v-for="field of objMethod.properties.attrs" v-bind:key="field.name">
      <div v-if="field.kind() == 'text'">
        <v-text-field
          v-model="objVariables[field.name]"
          :label="field.label"
          :name="field.name"
        ></v-text-field>
      </div>
    </div> -->

  </div>
</template>

<script>
import { registry } from "si-registry";
const items = registry.get("item");
console.log(items)


export default {
  name: "PropertyList",
  components: {
  },
};
</script>