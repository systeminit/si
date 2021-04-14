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
      />
      <SiSelect
        size="xs"
        id="systemSelect"
        name="systemSelect"
        :options="systemsList"
        v-model="currentSystemId"
        class="pl-1"
        :styling="schematicSelectorStyling()"
        v-if="schematicKind == 'system'"
      />
      <NodeAddMenu
        class="pl-1"
        @selected="nodeCreate"
        :disabled="!isEditable"
      />
    </template>
    <template v-slot:content>
      <div class="relative w-full h-full">
        <div v-if="schematic">
          <SchematicViewer
            class="absolute z-10"
            ref="graphViewer"
            :schematic="schematic"
            :schematicPanelStoreCtx="schematicPanelStoreCtx"
            :storesCtx="storesCtx"
          />
        </div>
      </div>
    </template>
  </Panel>
</template>

<script lang="ts">
import Vue from "vue";
import { mapGetters, mapState } from "vuex";

import {
  ctxMapState,
  InstanceStoreContext,
  registerStore,
  unregisterStore,
} from "@/store";
import { PanelEventBus } from "@/atoms/PanelEventBus";
import { Cg2dCoordinate } from "@/api/sicg";

import { SessionStore } from "@/store/modules/session";
import { ApplicationContextStore } from "@/store/modules/applicationContext";
import {
  SchematicPanelStore,
  schematicPanelStore,
  NodeSelectWithIdPayload,
  schematicPanelStoreSubscribeEvents,
} from "@/store/modules/schematicPanel";
import { EditorStore } from "@/store/modules/editor";
import { NodeCreatePayload } from "@/store/modules/schematicPanel";

import { ISchematicNode, SchematicKind } from "@/api/sdf/model/schematic";

import { INodeCreateReply } from "@/api/sdf/dal/schematicDal";
import { IGetApplicationContextRequest } from "@/api/sdf/dal/applicationContextDal";
import { IGetApplicationSystemSchematicRequest } from "@/api/sdf/dal/schematicDal";

import SchematicViewer, {
  StoresCtx,
  StoreCtx,
} from "@/organisims/SchematicViewer.vue";
import SiSelect, { SelectProps } from "@/atoms/SiSelect.vue";
import SiLoader from "@/atoms/SiLoader.vue";
import NodeAddMenu, {
  AddMenuSelectedPayload,
} from "@/molecules/NodeAddMenu.vue";
import Panel from "@/molecules/Panel.vue";

import _ from "lodash";

interface IData {
  schematicPanelStoreCtx: InstanceStoreContext<SchematicPanelStore>;
  isLoading: boolean;
  schematicKind: SchematicKind;
  storesCtx: StoresCtx;
  id: string;
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
      isLoading: true,
      schematicKind: SchematicKind.System,
      storesCtx: storesCtx,
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
    }),
    ...mapGetters({
      isEditable: "editor/inEditable",
    }),
    currentApplicationId(): string | undefined {
      return this.currentApplicationContext?.applicationId;
    },
    schematicKinds(): SelectProps["options"] {
      return [
        { label: "System", value: SchematicKind.System },
        { label: "Deployment", value: SchematicKind.Deployment },
        { label: "Implementation", value: SchematicKind.Implementation },
      ];
    },
    rootObjectId(): SchematicPanelStore["rootObjectId"] {
      return this.schematicPanelStoreCtx.state.rootObjectId;
    },
    schematic(): SchematicPanelStore["schematic"] {
      return this.schematicPanelStoreCtx.state.schematic;
    },
  },
  methods: {
    onInitialMaximizedFullUpdates(value: Boolean) {
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
    async nodeSelect(schematicNode: ISchematicNode) {
      console.log("selected (does nothing!!!", { schematicNode });
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
    async loadSchematic() {
      this.isLoading = true;
      if (this.currentWorkspace && this.rootObjectId && this.currentSystem) {
        if (this.schematicKind == SchematicKind.System) {
          let request: Record<string, any> = {
            workspaceId: this.currentWorkspace.id,
            rootObjectId: this.rootObjectId,
            systemId: this.currentSystem.id,
          };
          if (this.currentChangeSet) {
            request["changeSetId"] = this.currentChangeSet.id;
          }
          if (this.currentEditSession) {
            request["editSessionId"] = this.currentEditSession.id;
          }
          let reply = await this.schematicPanelStoreCtx.dispatch(
            "loadApplicationSystemSchematic",
            request,
          );
          if (reply.error) {
            PanelEventBus.$emit("editor-error-message", reply.error.message);
          }
        }
      }
      this.isLoading = false;
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
      this.isLoading = true;
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
      this.isLoading = true;
      await this.schematicPanelStoreCtx.dispatch(
        "setRootObjectId",
        applicationId,
      );
      await this.loadSchematic();
    },
    async currentChangeSet() {
      this.isLoading = true;
      await this.loadSchematic();
    },
    initialMaximizedFull(value) {
      this.onInitialMaximizedFullUpdates(value);
    },
  },
});
</script>
