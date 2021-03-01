<template>
  <Panel
    initialPanelType="systemSchematic"
    :panelRef="panelRef"
    :panelContainerRef="panelContainerRef"
    :initialMaximizedContainer="initialMaximizedContainer"
    :initialMaximizedFull="initialMaximizedFull"
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
      />
      <NodeAddMenu
        class="pl-1"
        @selected="nodeCreate"
        :disabled="!isEditable"
      />
    </template>
    <template v-slot:content>
      <CodeLoader
        v-if="isLoading"
        viewBox="0 0 300 200"
        primaryColor="#2d3748"
        secondaryColor="#000000"
      ></CodeLoader>
      <div class="relative w-full h-full" v-else>
        <div v-if="schematic">
          <GraphViewer
            class="absolute z-10"
            ref="graphViewer"
            :graph="schematic"
            :schematicStoreCtx="schematicStoreCtx"
            :storesCtx="storesCtx"
          />
        </div>
      </div>
    </template>
  </Panel>
</template>

<script lang="ts">
import Vue from "vue";

import _ from "lodash";

import Panel from "@/molecules/Panel.vue";
import NodeAddMenu from "@/molecules/NodeAddMenu.vue";
import { EntityObject } from "si-registry/lib/systemComponent";
import { PanelEventBus } from "@/atoms/PanelEventBus";
import { INodeCreateReply } from "@/api/sdf/dal/editorDal";
import {
  ctxMapState,
  InstanceStoreContext,
  registerStore,
  unregisterStore,
} from "@/store";
import {
  SchematicStore,
  schematicStore,
  NodeSelectWithIdPayload,
  schematicStoreSubscribeEvents,
} from "@/store/modules/schematic";
import { mapGetters, mapState } from "vuex";
import { SessionStore } from "@/store/modules/session";
import SiLoader from "@/atoms/SiLoader.vue";
import { CodeLoader } from "vue-content-loader";
import SiSelect, { SelectProps } from "@/atoms/SiSelect.vue";
import { ISchematicNode, SchematicKind } from "@/api/sdf/model/schematic";
import { IGetApplicationContextRequest } from "@/api/sdf/dal/applicationContextDal";
import { IGetApplicationSystemSchematicRequest } from "@/api/sdf/dal/schematicDal";
import { EditorStore, NodeCreatePayload } from "@/store/modules/editor";

import GraphViewer, { StoresCtx, StoreCtx } from "@/organisims/GraphViewer.vue";

import { Cg2dCoordinate } from "@/api/sicg";

interface IData {
  schematicStoreCtx: InstanceStoreContext<SchematicStore>;
  isLoading: boolean;
  schematicKind: SchematicKind;
  storesCtx: StoresCtx;
  id: string;
}

export default Vue.extend({
  name: "SchematicPanel",
  props: {
    panelRef: String,
    panelContainerRef: String,
    initialMaximizedFull: Boolean,
    initialMaximizedContainer: Boolean,
  },
  components: {
    Panel,
    GraphViewer,
    NodeAddMenu,
    CodeLoader,
    SiSelect,
  },
  data(): IData {
    let id = _.uniqueId("schematicPanel:");
    let schematicStoreCtx: InstanceStoreContext<SchematicStore> = new InstanceStoreContext(
      {
        storeName: "schematic",
        componentId: "schematicPanel",
        instanceId: id,
      },
    );
    let storesCtx: StoresCtx = {};
    storesCtx["schematicStoreCtx"] = schematicStoreCtx;
    return {
      id: id,
      schematicStoreCtx,
      isLoading: true,
      schematicKind: SchematicKind.System,
      storesCtx: storesCtx,
    };
  },
  computed: {
    ...mapState({
      currentWorkspace: (state: any): SessionStore["currentWorkspace"] =>
        state.session.currentWorkspace,
      sessionContext: (state: any): SessionStore["sessionContext"] =>
        state.session.sessionContext,
      currentChangeSet: (state: any): EditorStore["currentChangeSet"] =>
        state.editor.currentChangeSet,
      currentSystem: (state: any): SessionStore["currentSystem"] =>
        state.session.currentSystem,
      editMode(): boolean {
        return this.$store.getters["editor/inEditable"];
      },
    }),
    ...mapGetters({
      isEditable: "editor/inEditable",
    }),
    schematicKinds(): SelectProps["options"] {
      return [
        { label: "System", value: SchematicKind.System },
        { label: "Deployment", value: SchematicKind.Deployment },
        { label: "Implementation", value: SchematicKind.Implementation },
      ];
    },
    rootObjectId(): SchematicStore["rootObjectId"] {
      return this.schematicStoreCtx.state.rootObjectId;
    },
    schematic(): SchematicStore["schematic"] {
      return this.schematicStoreCtx.state.schematic;
    },
  },
  methods: {
    async nodeSelect(schematicNode: ISchematicNode) {
      console.log("selected", { schematicNode });
    },
    async nodeCreate(entityObject: EntityObject) {
      let payload: NodeCreatePayload = {
        entityObject: entityObject,
        sourcePanelId: this.schematicStoreCtx.instanceId,
      };

      let reply: INodeCreateReply = await this.$store.dispatch(
        "editor/nodeCreate",
        payload,
      );
      if (!reply.error) {
        // @ts-ignore
        this.$refs.graphViewer.setIsNodeCreate();
        // set
      } else {
        PanelEventBus.$emit("editor-error-message", reply.error.message);
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
          let reply = await this.schematicStoreCtx.dispatch(
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
      this.schematicStoreCtx,
      schematicStore,
      schematicStoreSubscribeEvents,
    );
  },
  async mounted() {
    if (this.sessionContext) {
      this.isLoading = true;
      await this.schematicStoreCtx.dispatch(
        "setRootObjectId",
        this.sessionContext.applicationId,
      );
      this.loadSchematic();
    }
  },
  async beforeDestroy() {
    unregisterStore(this.schematicStoreCtx);
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
      await this.schematicStoreCtx.dispatch("setRootObjectId", applicationId);
      await this.loadSchematic();
    },
    async currentChangeSet() {
      this.isLoading = true;
      await this.loadSchematic();
    },
  },
});
</script>
