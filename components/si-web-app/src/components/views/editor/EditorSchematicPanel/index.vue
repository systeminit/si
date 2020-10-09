<template>
  <div ref="schematic-panel" class="flex flex-col w-full h-full">
    <div
      id="schematic-panel-menu"
      class="flex flex-row flex-no-wrap content-between justify-between w-full bg-black"
    >
      <div class="flex flex-row justify-start mx-3">
        <button
          class="px-4 py-2 focus:outline-none"
          v-bind:class="buttonClass"
          @click="createNode()"
          type="button"
          :disabled="!isEditMode"
        >
          <plus-square-icon size="1.1x" />
        </button>
        <div class="text-black">
          <select
            class="my-2 text-xs leading-tight text-gray-400 bg-gray-800 border focus:outline-none"
            v-model="selectedEntityType"
            :disabled="!isEditMode"
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
import { Maximize2Icon, PlusSquareIcon, CommandIcon } from "vue-feather-icons";
import NodeEditor from "./NodeEditor.vue";
import { registry } from "si-registry";
import _ from "lodash";
import { mapState } from "vuex";
import { RootStore } from "@/store";
import { camelCase } from "change-case";
import { NodeKind } from "@/api/sdf/model/node";

interface Data {
  selectedEntityType: string;
  selectedAction: string;
  entityTypeList: any[];
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
      selectedEntityType: "service",
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
      await this.$store.dispatch("editor/nodeCreate", {
        kind: NodeKind.Entity,
        objectType: this.selectedEntityType,
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

.is-hidden .schematic-editor {
  @apply overflow-hidden h-0;
}
</style>
