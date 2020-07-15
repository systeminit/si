<template>
  <div ref="schematic-panel" class="flex flex-col h-full w-full">
    <div
      id="schematic-panel-menu"
      class="flex flex-row justify-between flex-no-wrap content-between bg-black w-full"
    >
      <div class="flex flex-row justify-start mx-3">
        <button
          class="text-white px-4 py-2 focus:outline-none"
          @click="createNode()"
          type="button"
        >
          <plus-square-icon size="1.1x" />
        </button>
        <div class="text-black">
          <select
            class="bg-gray-800 border text-gray-400 my-2 text-xs leading-tight focus:outline-none"
            v-model="selectedEntityType"
          >
            <option
              v-for="entity in entityTypeList"
              :key="entity.typeName"
              :value="entity.typeName"
            >
              {{ entity.typeName }}
            </option>
          </select>
        </div>
      </div>

      <div class="mx-3">
        <button
          class="text-white px-4 py-2 focus:outline-none"
          @click="maximizePanel()"
          type="button"
        >
          <maximize-2-icon size="1.1x"></maximize-2-icon>
        </button>
      </div>
    </div>

    <div class="flex w-full h-full">
      <NodeEditor />
    </div>
  </div>
</template>

<script>
import { Maximize2Icon, PlusSquareIcon } from "vue-feather-icons";
import NodeEditor from "./NodeEditor.vue";
import { registry } from "si-registry";
import _ from "lodash";

export default {
  name: "EditorSchematicPanel",
  components: {
    Maximize2Icon,
    NodeEditor,
    PlusSquareIcon,
  },
  data() {
    const entityTypeList = _.sortBy(registry.listEntities(), ["typeName"]);
    return {
      selectedEntityType: "serviceEntity",
      entityTypeList,
    };
  },
  methods: {
    maximizePanel() {
      this.$emit("maximizePanelMsg", {
        panel: {
          id: "schematic",
        },
      });
    },
    async createNode() {
      await this.$store.dispatch("node/create", {
        nodeType: "Entity",
        typeName: this.selectedEntityType,
      });
    },
  },
};
</script>

<style scoped>
.property-editor-bg-color {
  background-color: #212324;
}

.property-title-bg-color {
  background-color: #292c2d;
}

.is-hidden .schematic-editor {
  @apply overflow-hidden h-0;
}
</style>
