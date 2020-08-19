<template>
  <div ref="schematic-panel" class="flex flex-col w-full h-full">
    <div
      id="schematic-panel-menu"
      class="flex flex-row flex-no-wrap content-between justify-between w-full bg-black"
    >
      <div class="flex flex-row justify-start mx-3">
        <button
          class="px-4 py-2 text-white focus:outline-none"
          @click="createNode()"
          type="button"
        >
          <plus-square-icon size="1.1x" />
        </button>
        <div class="text-black">
          <select
            class="my-2 text-xs leading-tight text-gray-400 bg-gray-800 border focus:outline-none"
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
        <button
          class="px-4 py-2 text-white focus:outline-none"
          @click="sendAction()"
          type="button"
        >
          <CommandIcon size="1.1x" />
        </button>
        <div class="text-black">
          <select
            class="my-2 text-xs leading-tight text-gray-400 bg-gray-800 border focus:outline-none"
            v-model="selectedAction"
          >
            <option key="delete" value="delete">
              delete (node)
            </option>
            <option v-for="action in actionList" :key="action" :value="action">
              {{ action }}
            </option>
          </select>
        </div>
      </div>

      <div class="mx-3">
        <button
          class="px-4 py-2 text-white focus:outline-none"
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

<script lang="ts">
import Vue from "vue";
import { Maximize2Icon, PlusSquareIcon, CommandIcon } from "vue-feather-icons";
import NodeEditor from "./NodeEditor.vue";
import { registry, EntityObject } from "si-registry";
import _ from "lodash";
import { mapState } from "vuex";
import { RootStore } from "@/store";
import { camelCase } from "change-case";
import { NodeNodeKind } from "@/graphql-types";

interface Data {
  selectedEntityType: string;
  selectedAction: string;
  entityTypeList: EntityObject[];
}

export default Vue.extend({
  name: "EditorSchematicPanel",
  components: {
    Maximize2Icon,
    NodeEditor,
    PlusSquareIcon,
    CommandIcon,
  },
  data(): Data {
    const entityTypeList = _.sortBy(registry.listEntities(), ["typeName"]);
    return {
      selectedEntityType: "serviceEntity",
      selectedAction: "delete",
      entityTypeList,
    };
  },
  methods: {
    maximizePanel(): void {
      this.$emit("maximizePanelMsg", {
        panel: {
          id: "schematic",
        },
      });
    },
    async createNode(): Promise<void> {
      await this.$store.dispatch("node/create", {
        nodeType: NodeNodeKind.Entity,
        typeName: this.selectedEntityType,
      });
    },
    async sendAction(): Promise<void> {
      await this.$store.dispatch("node/sendAction", {
        action: this.selectedAction,
      });
    },
  },
  computed: {
    actionList(): string[] {
      if (this.$store.state.node.current) {
        const actionList = registry.listActions();
        const currentNode = this.$store.state.node.current;
        let result =
          actionList[camelCase(currentNode.stack[0].siStorable.typeName)];
        return result;
      } else {
        return [];
      }
    },
  },
});
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
