<template>
  <div ref="schematic-panel" class="flex flex-col w-full h-full">
    <div
      id="schematic-panel-menu"
      class="flex flex-row flex-no-wrap content-between justify-between w-full bg-black"
    >
      <div class="flex flex-row justify-start items-center mx-3">
        <NodeAddMenu
          :entityTypeList="entityTypeList"
          class="z-50"
          @selected="createNode"
          :disabled="!isEditMode"
        />

        <button
          class="px-4 py-2 focus:outline-none"
          v-bind:class="buttonClass"
          @click="sendAction()"
          :disabled="!isEditMode"
          type="button"
        >
          <CommandIcon size="1.1x" />
        </button>
        <div class="text-black">
          <select
            class="my-2 text-xs leading-tight text-gray-400 bg-gray-800 border focus:outline-none"
            :disabled="!isEditMode"
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
import { Maximize2Icon, CommandIcon } from "vue-feather-icons";
import NodeEditor from "./NodeEditor.vue";
import { registry } from "si-registry";
import _ from "lodash";
import { mapState } from "vuex";
import { RootStore } from "@/store";
import { camelCase } from "change-case";
import { NodeKind } from "@/api/sdf/model/node";
import NodeAddMenu from "./NodeAddMenu.vue";
import { EntityObject } from "si-registry/lib/systemComponent";

interface Data {
  selectedEntityType: string;
  selectedAction: string;
  entityTypeList: EntityObject[];
  addNodeMenuIsVisible: boolean;
}

export default Vue.extend({
  name: "EditorSchematicPanel",
  components: {
    Maximize2Icon,
    NodeEditor,
    CommandIcon,
    NodeAddMenu,
  },
  data(): Data {
    const entityTypeList = _.sortBy(registry.listEntities(), ["typeName"]);
    return {
      selectedEntityType: "service",
      selectedAction: "delete",
      entityTypeList,
      addNodeMenuIsVisible: false,
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
    toggleAddNodeMenu(): void {
      this.addNodeMenuIsVisible = !this.addNodeMenuIsVisible;
    },
    async createNode(entity: EntityObject): Promise<void> {
      await this.$store.dispatch("editor/nodeCreate", {
        kind: NodeKind.Entity,
        objectType: entity.typeName,
      });
    },
    async sendAction(): Promise<void> {
      await this.$store.dispatch("editor/sendAction", {
        action: this.selectedAction,
      });
    },
  },
  computed: {
    isEditMode(): boolean {
      return this.$store.state.editor.mode == "edit";
    },
    buttonClass(): Record<string, boolean> {
      if (this.isEditMode) {
        return {
          "text-white": true,
        };
      } else {
        return {
          "text-gray-600": true,
        };
      }
    },
    actionList(): string[] {
      if (this.$store.state.editor.node) {
        return this.$store.state.editor.node.actionList();
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
</style>
