<template>
  <div id="property-panel-list" class="property-bg-color w-full h-full">
    <!--     <div class="mx-3">
      <button class="text-yellow-500 px-4 py-2 focus:outline-none" @click="onClick()" type="button">
        apollo.queries.kubernetesDeploymentEntityGet.refetch()
      </button>
    </div> -->

    <PropObject :propObject="entitySchema" :propObjectModel="entity" />
  </div>
</template>

<script>
import { registry } from "si-registry";
import PropObject from "./PropObject.vue";
import { mapGetters } from "vuex";

export default {
  name: "PropertyListView",
  components: {
    PropObject,
  },
  props: {
    nodeId: String, // make this more generic later...
    typeName: String,
  },
  data() {
    const entitySchema = {
      properties: registry.get(this.typeName).fields,
    };
    return {
      entitySchema,
      entity: {},
    };
  },
  watch: {
    async nodeId() {
      await this.$store.dispatch("entity/get", {
        id: this.nodeId,
        typeName: this.typeName,
      });
      const entity = this.$store.getters["entity/get"](this.nodeId);
      this.entity = entity;
    },
  },
};
</script>

<style scoped>
.property-bg-color {
  background-color: #212324;
}

.property-title-bg-color {
  background-color: #292c2d;
}
</style>
