<template>
  <div id="property-panel-list" class="w-full h-full property-bg-color">
    <div class="flex flex-row-reverse pr-8 pb-4 text-white" v-if="errorMsg">
      {{ errorMsg }}
    </div>
    <div class="flex flex-row-reverse pr-8 pb-4">
      <button
        class="bg-teal-700 px-4 py-2 text-white hover:bg-teal-600"
        @click="createEntity()"
        type="button"
      >
        Create
      </button>
    </div>

    <PropObject
      :propObject="entityCreate.request"
      :propObjectModel="entityCreateVars"
      @propChangeMsg="propChangeMsg"
    />
  </div>
</template>

<script>
import { registry } from "si-registry";
import PropObject from "./PropObject.vue";

export default {
  name: "PropertyListCreate",
  props: {
    node: Object,
    typeName: String,
  },
  components: {
    PropObject,
  },
  data() {
    const entity = registry.get(this.typeName);

    const entityCreate = entity.methods.getEntry("create");

    const entityCreateVars = entity.graphql.variablesObject({
      methodName: "create",
    });

    return {
      entity,
      entityCreate,
      entityCreateVars,
      errorMsg: null,
    };
  },
  methods: {
    propChangeMsg(event) {
      this.entityCreateVars = event["value"];
    },
    async createEntity() {
      try {
        await this.$store.dispatch("entity/create", {
          typeName: this.typeName,
          data: this.entityCreateVars,
        });
        await this.$store.dispatch("editor/removeNode", this.node);
      } catch (err) {
        this.errorMsg = `${err.name}: ${err.message}`;
        throw err;
      }
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
