<template>
  <Panel
    initialPanelType="attribute"
    :panelRef="panelRef"
    :panelContainerRef="panelContainerRef"
    :initialMaximizedContainer="initialMaximizedContainer"
    :initialMaximizedFull="initialMaximizedFull"
    v-on="$listeners"
  >
    <template v-slot:menuButtons>
      <div class="flex w-20">
        <SiSelect
          size="xs"
          id="attributePanelObjectSelect"
          name="attributePanelObjectSelect"
          :options="objectList"
          v-model="selectedObjectId"
          class="pl-1"
          :disabled="selectionLocked"
        />
      </div>
      <button class="pl-1 focus:outline-none" @click="toggleSelectionLock()">
        <UnlockIcon size="1.1x" v-if="selectionLocked" />
        <LockIcon size="1.1x" v-else />
      </button>
      <button
        class="pl-1 focus:outline-none"
        :class="attributeViewClasses()"
        @click="switchToAttributeView()"
      >
        <DiscIcon size="1.1x" />
      </button>
      <button
        class="pl-1 focus:outline-none"
        :class="codeViewClasses()"
        @click="switchToCodeView()"
      >
        <CodeIcon size="1.1x" />
      </button>
      <button
        class="pl-1 text-white focus:outline-none"
        :class="eventViewClasses()"
        @click="switchToEventView()"
      >
        <RadioIcon size="1.1x" />
      </button>
      <!--
      <button
        class="pl-1 text-white focus:outline-none"
        :class="refreshClasses"
        @click="refreshObject()"
        :disabled="!needsRefresh"
      >
        <RefreshCwIcon size="1.1x" />
      </button>
        -->
    </template>
    <template v-slot:content>
      <div class="flex flex-row w-full h-full" v-if="currentObject">
        <AttributeViewer
          v-if="activeView == 'attribute'"
          :attributeStoreCtx="attributeStoreCtx"
        />
        <CodeViewer
          v-else-if="activeView == 'code'"
          :attributeStoreCtx="attributeStoreCtx"
        />
        <div v-else>
          Not implemented
          <div>
            <VueJsonPretty :data="currentObject" />
          </div>
        </div>
      </div>
      <div class="flex w-full" v-else>
        <h2>No object selected</h2>
      </div>
    </template>
  </Panel>
</template>

<script lang="ts">
import Vue from "vue";

import Panel from "@/molecules/Panel.vue";
import { InstanceStoreContext, registerStore, unregisterStore } from "@/store";
import {
  attributeStore,
  AttributeStore,
  attributeStoreSubscribeEvents,
} from "@/store/modules/attribute";
import {
  AttributeDal,
  IGetEntityReply,
  IGetEntityRequest,
  IGetObjectListReply,
  IGetObjectListRequest,
} from "@/api/sdf/dal/attributeDal";
import { mapState } from "vuex";
import { SessionStore } from "@/store/modules/session";
import { EditorStore } from "@/store/modules/editor";
import { PanelEventBus } from "@/atoms/PanelEventBus";
import SiSelect from "@/atoms/SiSelect.vue";
import {
  UnlockIcon,
  LockIcon,
  CodeIcon,
  RadioIcon,
  DiscIcon,
} from "vue-feather-icons";
import VueJsonPretty from "vue-json-pretty";
import "vue-json-pretty/lib/styles.css";
import { Entity } from "@/api/sdf/model/entity";
import Bottle from "bottlejs";
import { Persister } from "@/api/persister";
import { SchematicNodeSelectedEvent } from "@/api/partyBus/SchematicNodeSelectedEvent";
import AttributeViewer from "@/organisims/AttributeViewer.vue";
import CodeViewer from "@/organisims/CodeViewer.vue";

interface IData {
  isLoading: boolean;
  attributeStoreCtx: InstanceStoreContext<AttributeStore>;
  selectedObjectId: string;
  selectionIsLocked: boolean;
  activeView: "attribute" | "code" | "event";
}

export default Vue.extend({
  name: "AttributePanel",
  props: {
    panelRef: String,
    panelContainerRef: String,
    initialMaximizedFull: Boolean,
    initialMaximizedContainer: Boolean,
  },
  components: {
    Panel,
    SiSelect,
    DiscIcon,
    RadioIcon,
    CodeIcon,
    LockIcon,
    UnlockIcon,
    VueJsonPretty,
    AttributeViewer,
    CodeViewer,
  },
  data(): IData {
    let attributeStoreCtx: InstanceStoreContext<AttributeStore> = new InstanceStoreContext(
      {
        storeName: "attribute",
        componentId: "attributePanel",
        instanceId: this.panelRef,
      },
    );
    let bottle = Bottle.pop("default");
    let persister: Persister = bottle.container.Persister;
    let persistedData = persister.getData(`${this.panelRef}-data`);
    if (persistedData) {
      return persistedData;
    } else {
      return {
        isLoading: false,
        selectedObjectId: "",
        attributeStoreCtx,
        selectionIsLocked: true,
        activeView: "attribute",
      };
    }
  },
  computed: {
    ...mapState({
      currentWorkspace: (state: any): SessionStore["currentWorkspace"] =>
        state.session.currentWorkspace,
      currentChangeSet: (state: any): EditorStore["currentChangeSet"] =>
        state.editor.currentChangeSet,
      sessionContext: (state: any): SessionStore["sessionContext"] =>
        state.session.sessionContext,
    }),
    selectionLocked(): AttributeStore["selectionLocked"] {
      return this.attributeStoreCtx.state.selectionLocked;
    },
    objectList(): AttributeStore["objectList"] {
      if (this.attributeStoreCtx) {
        return this.attributeStoreCtx.state.objectList;
      } else {
        return [{ label: "", value: "" }];
      }
    },
    currentObject(): AttributeStore["currentObject"] {
      if (this.attributeStoreCtx) {
        return this.attributeStoreCtx.state.currentObject;
      } else {
        return null;
      }
    },
    needsRefresh(): AttributeStore["needsRefresh"] {
      if (this.attributeStoreCtx) {
        return this.attributeStoreCtx.state.needsRefresh;
      } else {
        return false;
      }
    },
    refreshClasses(): Record<string, any> {
      if (this.needsRefresh) {
        return { "text-yellow-300": true };
      } else {
        return { "text-gray-700": true };
      }
    },
  },
  methods: {
    async refreshObject() {
      let reply: IGetEntityReply = await this.attributeStoreCtx.dispatch(
        "refreshEntity",
      );
      if (reply.error) {
        PanelEventBus.$emit("editor-error-message", reply.error.message);
      }
    },
    viewClasses(view: IData["activeView"]): Record<string, any> {
      if (view == this.activeView) {
        return { "text-blue-300": true };
      } else {
        return {};
      }
    },
    attributeViewClasses(): Record<string, any> {
      return this.viewClasses("attribute");
    },
    codeViewClasses(): Record<string, any> {
      return this.viewClasses("code");
    },
    eventViewClasses(): Record<string, any> {
      return this.viewClasses("event");
    },
    switchToCodeView() {
      this.activeView = "code";
    },
    switchToAttributeView() {
      this.activeView = "attribute";
    },
    switchToEventView() {
      this.activeView = "event";
    },
    async toggleSelectionLock() {
      this.selectionIsLocked = await this.attributeStoreCtx.dispatch(
        "toggleSelectionLocked",
      );
    },
    async loadObject() {
      this.isLoading = true;
      let request: IGetEntityRequest;
      if (
        this.currentWorkspace &&
        this.currentChangeSet &&
        this.selectedObjectId
      ) {
        request = {
          workspaceId: this.currentWorkspace.id,
          changeSetId: this.currentChangeSet.id,
          entityId: this.selectedObjectId,
        };
      } else if (this.currentWorkspace) {
        request = {
          workspaceId: this.currentWorkspace.id,
          entityId: this.selectedObjectId,
        };
      } else {
        this.isLoading = false;
        return;
      }
      if (!request.entityId) {
        this.isLoading = false;
        this.attributeStoreCtx.dispatch("clearObject");
        return;
      }
      let reply: IGetEntityReply = await this.attributeStoreCtx.dispatch(
        "loadEntity",
        request,
      );
      if (reply.error) {
        PanelEventBus.$emit("editor-error-message", reply.error.message);
      }
    },
    async loadObjectList() {
      this.isLoading = true;
      let request: IGetObjectListRequest;
      if (
        this.currentWorkspace &&
        this.currentChangeSet &&
        this.sessionContext
      ) {
        request = {
          workspaceId: this.currentWorkspace.id,
          changeSetId: this.currentChangeSet.id,
          applicationId: this.sessionContext.applicationId,
        };
      } else if (this.currentWorkspace && this.sessionContext) {
        request = {
          workspaceId: this.currentWorkspace.id,
          applicationId: this.sessionContext.applicationId,
        };
      } else {
        this.isLoading = false;
        throw new Error(
          "cannot load node list for attribute panel; missing a workspace or session context! bug!",
        );
      }
      let reply: IGetObjectListReply = await this.attributeStoreCtx.dispatch(
        "loadObjectList",
        request,
      );
      if (reply.error) {
        PanelEventBus.$emit("editor-error-message", reply.error.message);
      }
      this.isLoading = false;
    },
  },
  async created() {
    registerStore(
      this.attributeStoreCtx,
      attributeStore,
      attributeStoreSubscribeEvents,
    );
  },
  async mounted() {
    if (this.sessionContext) {
      await this.loadObjectList();
    }
    if (this.selectedObjectId) {
      await this.loadObject();
    }
    if (!this.selectionIsLocked) {
      this.toggleSelectionLock();
    }
  },
  async beforeDestroy() {
    unregisterStore(this.attributeStoreCtx);
    let bottle = Bottle.pop("default");
    let persister: Persister = bottle.container.Persister;
    persister.removeData(`${this.panelRef}-data`);
  },
  watch: {
    async currentChangeSet() {
      await this.loadObjectList();
      await this.loadObject();
    },
    async currentWorkspace() {
      await this.loadObjectList();
      await this.loadObject();
    },
    async sessionContext() {
      await this.loadObjectList();
    },
    async selectedObjectId(newSelectedObject) {
      if (newSelectedObject != "") {
        await this.loadObject();
      } else {
        this.attributeStoreCtx.dispatch("setCurrentObject", null);
      }
    },
    $data: {
      handler: function(newData, oldData) {
        let bottle = Bottle.pop("default");
        let persister: Persister = bottle.container.Persister;
        persister.setData(`${this.panelRef}-data`, newData);
      },
      deep: true,
    },
  },
});
</script>
