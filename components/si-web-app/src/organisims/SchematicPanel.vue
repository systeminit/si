<template>
  <Panel
    initialPanelType="schematic"
    :panelIndex="panelIndex"
    :panelRef="panelRef"
    :panelContainerRef="panelContainerRef"
    :initialMaximizedContainer="initialMaximizedContainer"
    :initialMaximizedFull="initialMaximizedFull"
    :isVisible="isVisible"
    :isMaximizedContainerEnabled="isMaximizedContainerEnabled"
    v-on="$listeners"
  >
    <template v-slot:menuButtons>
      <SiSelect
        size="xs"
        id="schematicSelect"
        name="schematicSelect"
        :options="schematicKinds"
        v-model="schematicKind"
        class="pl-1"
        :styling="schematicSelectorStyling()"
        @input="schematicSelected($event)"
      />
      <SiSelect
        size="xs"
        id="systemSelect"
        name="systemSelect"
        :options="systemsList"
        v-model="currentSystemId"
        class="pl-1"
        :styling="schematicSelectorStyling()"
        v-if="schematicKind === 'deployment'"
      />

      <div class="flex flex-row" v-if="schematicKind === 'component'">
        <SiSelect
          size="xs"
          id="schematicPanelNodePin"
          name="schematicPanelNodePin"
          :options="nodeList()"
          v-if="nodeList()"
          v-model="pinnedNodeId"
          class="pl-1"
          :disabled="selectionIsTracked"
        />
        <button class="pl-1 focus:outline-none" @click="toggleSelectionTrack()">
          <TargetIcon size="0.9x" :class="targetIconStyling()" />
        </button>
      </div>

      <NodeAddMenu
        class="pl-2"
        :filter="addMenuFilter"
        @selected="nodeCreate"
        :disabled="!isEditable"
      />
    </template>
    <template v-slot:content>
      <div class="relative w-full h-full">
        <SchematicViewer
          class="absolute z-10"
          ref="graphViewer"
          :schematic="schematic"
          :schematicKind="schematicKind"
          :schematicPanelStoreCtx="schematicPanelStoreCtx"
          :storesCtx="storesCtx"
        />
      </div>
    </template>
  </Panel>
</template>

<script lang="ts">
import Vue from "vue";
import { mapGetters, mapState } from "vuex";

import { InstanceStoreContext, registerStore, unregisterStore } from "@/store";
import { PanelEventBus } from "@/atoms/PanelEventBus";

import { SessionStore } from "@/store/modules/session";
import { ApplicationContextStore } from "@/store/modules/applicationContext";
import {
  SchematicPanelStore,
  schematicPanelStore,
  schematicPanelStoreSubscribeEvents,
} from "@/store/modules/schematicPanel";
import { EditorStore } from "@/store/modules/editor";
import { NodeCreatePayload } from "@/store/modules/schematicPanel";

import { ISchematicNode, SchematicKind } from "@/api/sdf/model/schematic";

import { INodeCreateReply } from "@/api/sdf/dal/schematicDal";

import { MenuFilter } from "@/molecules/NodeAddMenu.vue";

import SiSelect, { SelectProps } from "@/atoms/SiSelect.vue";
import NodeAddMenu from "@/molecules/NodeAddMenu.vue";
import Panel from "@/molecules/Panel.vue";
import SchematicViewer, { StoresCtx } from "@/organisims/SchematicViewer.vue";

import { TargetIcon } from "vue-feather-icons";

import _ from "lodash";

interface IData {
  schematicPanelStoreCtx: InstanceStoreContext<SchematicPanelStore>;
  schematicKind: SchematicKind;
  storesCtx: StoresCtx;
  id: string;
  selectionIsTracked: Boolean;
  pinnedNodeId: string;
}

export default Vue.extend({
  name: "SchematicPanel",
  props: {
    panelIndex: Number,
    panelRef: String,
    panelContainerRef: String,
    initialMaximizedFull: Boolean,
    initialMaximizedContainer: Boolean,
    isVisible: Boolean,
    isMaximizedContainerEnabled: Boolean,
  },
  components: {
    Panel,
    SchematicViewer,
    NodeAddMenu,
    SiSelect,
    TargetIcon,
  },
  data(): IData {
    let id = _.uniqueId("schematicPanel:");
    let schematicPanelStoreCtx: InstanceStoreContext<SchematicPanelStore> = new InstanceStoreContext(
      {
        storeName: "schematicPanel",
        componentId: "schematicPanel",
        instanceId: id,
      },
    );
    let storesCtx: StoresCtx = {};
    storesCtx["schematicPanelStoreCtx"] = schematicPanelStoreCtx;
    return {
      id: id,
      schematicPanelStoreCtx,
      schematicKind: SchematicKind.Deployment,
      storesCtx: storesCtx,
      selectionIsTracked: false,
      pinnedNodeId: "",
    };
  },
  computed: {
    ...mapState({
      currentApplicationContext: (state: any): EditorStore["context"] =>
        state.editor.context,
      currentWorkspace: (state: any): SessionStore["currentWorkspace"] =>
        state.session.currentWorkspace,
      sessionContext: (state: any): SessionStore["sessionContext"] =>
        state.session.sessionContext,
      currentChangeSet: (state: any): EditorStore["currentChangeSet"] =>
        state.editor.currentChangeSet,
      currentEditSession: (state: any): EditorStore["currentEditSession"] =>
        state.editor.currentEditSession,
      currentSystem: (state: any): SessionStore["currentSystem"] =>
        state.session.currentSystem,
      editMode(): boolean {
        return this.$store.getters["editor/inEditable"];
      },
      systemsList(): ApplicationContextStore["systemsList"] {
        return [
          {
            value: "",
            label: "production",
          },
        ];
      },
      currentSystemId(): string {
        return "production";
      },
      selectedNode(): ISchematicNode | null {
        return this.storesCtx.schematicPanelStoreCtx.state.selectedNode;
      },
    }),
    ...mapGetters({
      isEditable: "editor/inEditable",
    }),
    currentApplicationId(): string | undefined {
      return this.currentApplicationContext?.applicationId;
    },
    schematicKinds(): SelectProps["options"] {
      return [
        { label: "Deployment", value: SchematicKind.Deployment },
        { label: "Component", value: SchematicKind.Component },
      ];
    },
    rootObjectId(): SchematicPanelStore["rootObjectId"] {
      return this.schematicPanelStoreCtx.state.rootObjectId;
    },
    schematic(): SchematicPanelStore["schematic"] {
      return this.schematicPanelStoreCtx.state.schematic;
    },
    addMenuFilter(): MenuFilter | string {
      switch (this.schematicKind) {
        case SchematicKind.Deployment: {
          return MenuFilter.Deployment;
        }
        case SchematicKind.Component: {
          return MenuFilter.Implementation;
        }
      }
      return "";
    },
  },
  async created() {
    registerStore(
      this.schematicPanelStoreCtx,
      schematicPanelStore,
      schematicPanelStoreSubscribeEvents,
    );
  },
  async mounted() {
    if (this.sessionContext) {
      await this.schematicPanelStoreCtx.dispatch(
        "setRootObjectId",
        this.sessionContext.applicationId,
      );
      this.loadSchematic();
    }
  },
  async beforeDestroy() {
    unregisterStore(this.schematicPanelStoreCtx);
  },
  watch: {
    async sessionContext(sessionContext: SessionStore["sessionContext"]) {
      let applicationId = this.sessionContext?.applicationId;
      if (!applicationId) {
        throw new Error(
          "failed to extract a root object id from the session; you need to set the context",
        );
      }
      await this.schematicPanelStoreCtx.dispatch(
        "setRootObjectId",
        applicationId,
      );
      await this.loadSchematic();
    },
    async currentChangeSet() {
      await this.loadSchematic();
    },
    initialMaximizedFull(value) {
      this.onInitialMaximizedFullUpdates(value);
    },
  },
  methods: {
    targetIconStyling(): Record<string, any> {
      let classes: Record<string, any> = {};
      classes["track-selection"] = this.selectionIsTracked;
      classes["manual-selection"] = !this.selectionIsTracked;
      return classes;
    },
    async toggleSelectionTrack() {
      if (this.selectionIsTracked) {
        this.selectionIsTracked = false;
      } else {
        this.selectionIsTracked = true;
      }
    },
    nodeList(): string[] {
      let nodeList = [];
      console.log("this.schematic:", this.schematic);
      if (this.schematic) {
        for (let node in this.schematic.nodes) {
          console.log(node);
          nodeList.push(node);
        }
      }
      console.log("nodeList:", nodeList);
      return nodeList;
    },
    async schematicSelected(kind: string) {
      if (_.capitalize(kind) in SchematicKind && this.sessionContext) {
        switch (kind) {
          case SchematicKind.Deployment: {
            await this.schematicPanelStoreCtx.dispatch(
              "setRootObjectId",
              this.sessionContext.applicationId,
            );
            this.loadSchematic();
            break;
          }

          case SchematicKind.Component: {
            if (this.selectedNode) {
              await this.schematicPanelStoreCtx.dispatch(
                "setRootObjectId",
                this.selectedNode.object.id,
              );
              this.loadSchematic();
              break;
            } else {
              this.clearSchematic();
            }
          }
        }
      }
    },
    onInitialMaximizedFullUpdates(value: boolean) {
      // @ts-ignore
      this.$refs.graphViewer.updateCanvasPosition();
    },
    schematicSelectorStyling(): Record<string, any> {
      let classes: Record<string, any> = {};
      classes["bg-selectordark"] = true;
      classes["text-gray-400"] = true;
      classes["border-gray-800"] = true;
      return classes;
    },
    includeRootNode(): boolean {
      if (this.schematicKind == SchematicKind.Deployment) {
        return false;
      } else {
        return true;
      }
    },
    async loadSchematic() {
      if (this.currentWorkspace && this.rootObjectId && this.currentSystem) {
        let request: Record<string, any> = {
          workspaceId: this.currentWorkspace.id,
          rootObjectId: this.rootObjectId,
          systemId: this.currentSystem.id,
          includeRootNode: this.includeRootNode(),
        };
        if (this.currentChangeSet) {
          request["changeSetId"] = this.currentChangeSet.id;
        }
        if (this.currentEditSession) {
          request["editSessionId"] = this.currentEditSession.id;
        }
        let reply = await this.schematicPanelStoreCtx.dispatch(
          "loadSchematic",
          request,
        );
        if (reply.error) {
          PanelEventBus.$emit("editor-error-message", reply.error.message);
        }
      }
    },
    async clearSchematic() {
      await this.schematicPanelStoreCtx.dispatch("clearSchematic");
    },
    async nodeCreate(entityType: string, event: MouseEvent) {
      if (
        this.currentApplicationId &&
        this.currentWorkspace &&
        this.currentChangeSet &&
        this.currentEditSession
      ) {
        const payload: NodeCreatePayload = {
          entityType,
          sourcePanelId: this.schematicPanelStoreCtx.instanceId,
          applicationId: this.currentApplicationId,
          workspaceId: this.currentWorkspace.id,
          changeSetId: this.currentChangeSet.id,
          editSessionId: this.currentEditSession.id,
        };

        let reply: INodeCreateReply = await this.schematicPanelStoreCtx.dispatch(
          "nodeCreate",
          payload,
        );
        if (!reply.error) {
          // @ts-ignore
          this.$refs.graphViewer.onNodeCreate(reply.node.node.id, event);
          // set
        } else {
          PanelEventBus.$emit("editor-error-message", reply.error.message);
        }
      }
    },
  },
});
</script>

<style scoped>
.track-selection {
  color: orange;
}

.manual-selection {
  color: grey;
}
</style>
