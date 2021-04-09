<template>
  <div id="application-editor" class="flex flex-col w-full h-full select-none">
    <div class="flex flex-col w-full h-full">
      <StatusBar :instanceId="applicationContextCtx.instanceId" />
      <ApplicationContext
        :applicationContextCtx="applicationContextCtx"
        :workspaceId="workspaceId"
        :applicationId="applicationId"
        @update-query-param="updateQueryParam"
        @remove-query-param="removeQueryParam"
      />
      <div id="editor" class="flex w-full h-full overflow-hidden">
        <Editor
          :workspaceId="workspaceId"
          :applicationId="applicationId"
          :context="editorContext"
          :applicationContextCtx="applicationContextCtx"
        />
      </div>
      <EventBar />
      <!--
    <div id="eventBar" class="w-full">
      <EventBar />
    </div>
      -->
    </div>
    <!-- this one is extra -->

    <SiModal name="leave" title="Alert" class="">
      <div class="flex flex-col items-center w-full h-full mb-2">
        <div class="text-base font-normal text-red-500">
          You have unsaved changes!
        </div>
        <div class="text-sm text-white">Are you sure you want to leave?</div>
      </div>
      <template v-slot:buttons>
        <SiButton
          size="sm"
          label="leave"
          class="mx-1"
          icon="null"
          kind="cancel"
          @click.native="leave"
        />
        <SiButton
          size="sm"
          label="stay"
          class="mx-1"
          icon="null"
          kind="save"
          @click.native="stay"
        />
      </template>
    </SiModal>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import Route from "vue-router";

import {
  registerStatusBar,
  StatusBarStore,
  unregisterStatusBar,
} from "@/store/modules/statusBar";
import {
  ApplicationContextStore,
  registerApplicationContext,
  unregisterApplicationContext,
} from "@/store/modules/applicationContext";

import StatusBar from "@/organisims/StatusBar.vue";
import EventBar from "@/organisims/EventBar.vue";
import ApplicationContext from "@/organisims/ApplicationContext.vue";
import Editor from "@/organisims/Editor.vue";
import SiModal from "@/molecules/SiModal.vue";
import SiButton from "@/atoms/SiButton.vue";

import { ctxMapState, InstanceStoreContext } from "@/store";
import {
  ISessionContextApplicationSystem,
  ISessionContextKind,
  SessionStore,
} from "@/store/modules/session";
import { mapState } from "vuex";
import { System } from "@/api/sdf/model/system";
import { IEditorContextApplication } from "@/store/modules/editor";
import { Persister } from "@/api/persister";
import Bottle from "bottlejs";

interface IData {
  applicationContextCtx: InstanceStoreContext<ApplicationContextStore>;
  statusBarCtx: InstanceStoreContext<StatusBarStore>;
  navDestination: any | null;
}

export default Vue.extend({
  name: "ApplicationDetails",
  components: {
    StatusBar,
    EventBar,
    ApplicationContext,
    Editor,
    SiModal,
    SiButton,
  },
  data(): IData {
    return {
      applicationContextCtx: new InstanceStoreContext({
        storeName: "applicationContext",
        componentId: "ApplicationDetails",
        instanceId: "applicationDetails",
      }),
      statusBarCtx: new InstanceStoreContext({
        storeName: "statusBar",
        componentId: "ApplicationDetails",
        instanceId: "applicationDetails",
      }),
      navDestination: null,
    };
  },
  props: {
    organizationId: {
      type: String,
    },
    workspaceId: {
      type: String,
    },
    applicationId: {
      type: String,
    },
  },
  computed: {
    ...mapState({
      currentSystem: (state: any): SessionStore["currentSystem"] =>
        state["session"]["currentSystem"],
    }),
    editorContext(): IEditorContextApplication {
      return {
        applicationId: this.applicationId,
        contextType: "applicationSystem",
      };
    },
    editMode(): ApplicationContextStore["editMode"] {
      return ctxMapState(this.applicationContextCtx, "editMode");
    },
  },
  methods: {
    async updateQueryParam(payload: Record<string, any>) {
      let bottle = Bottle.pop("default");
      let persister: Persister = bottle.container.Persister;
      persister.updateQueryParam(payload);
    },
    async removeQueryParam(payload: string[]) {
      let bottle = Bottle.pop("default");
      let persister: Persister = bottle.container.Persister;
      persister.removeQueryParam(payload);
    },
    async wipeQueryParam() {
      let bottle = Bottle.pop("default");
      let persister: Persister = bottle.container.Persister;
      persister.wipeQueryParams();
    },
    leave() {
      this.$modal.hide("leave");
      this.navDestination();
    },
    stay() {
      this.$modal.hide("leave");
    },
  },
  async created() {
    registerStatusBar(this.applicationContextCtx.instanceId);

    await registerApplicationContext(
      this.applicationContextCtx,
      this.statusBarCtx,
    );

    await this.$store.dispatch(
      this.applicationContextCtx.dispatchPath("activate"),
      this.applicationContextCtx,
    );

    if (this.$route.query.changeSetId && this.$route.query.editSessionId) {
      let reply = await this.$store.dispatch(
        this.applicationContextCtx.dispatchPath("loadChangeSetAndEditSession"),
        {
          changeSetId: this.$route.query.changeSetId,
          editSessionId: this.$route.query.editSessionId,
        },
      );
      if (reply.error) {
        await this.wipeQueryParam();
      }
    }

    let editModeBool = false;
    if (this.$route.query.editMode == "true") {
      editModeBool = true;
    }
    await this.$store.dispatch(
      this.applicationContextCtx.dispatchPath("setEditMode"),
      editModeBool,
    );
    if (this.currentSystem) {
      let sessionContext: ISessionContextApplicationSystem = {
        kind: ISessionContextKind.ApplicationSystem,
        applicationId: this.applicationId,
        systemId: this.currentSystem.id,
      };
      await this.$store.dispatch("session/setSessionContext", sessionContext);
    }
  },
  async beforeDestroy() {
    unregisterStatusBar(this.applicationContextCtx.instanceId);
    unregisterApplicationContext(this.applicationContextCtx);
    await this.$store.dispatch("session/setSessionContext", null);
  },
  beforeRouteLeave(_to, _from, next: any) {
    if (this.editMode) {
      if (next != null) {
        this.navDestination = next;
        this.$modal.show("leave");
        next(false);
      }
    } else {
      next();
    }
  },
  watch: {
    async currentSystem(currentSystem: SessionStore["currentSystem"]) {
      if (currentSystem) {
        let sessionContext: ISessionContextApplicationSystem = {
          kind: ISessionContextKind.ApplicationSystem,
          applicationId: this.applicationId,
          systemId: currentSystem.id,
        };
        await this.$store.dispatch("session/setSessionContext", sessionContext);
      }
    },
  },
});
</script>
