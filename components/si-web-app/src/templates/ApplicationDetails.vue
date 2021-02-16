<template>
  <div id="application-editor" class="flex flex-col w-full h-full">
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
        <Editor />
      </div>
      <!--
    <div id="eventBar" class="w-full">
      <EventBar />
    </div>
      -->
    </div>
    <!-- this one is extra -->
  </div>
</template>

<script lang="ts">
import Vue from "vue";

import {
  registerStatusBar,
  unregisterStatusBar,
} from "@/store/modules/statusBar";
import {
  ApplicationContextStore,
  registerApplicationContext,
  unregisterApplicationContext,
} from "@/store/modules/applicationContext";

import StatusBar from "@/organisims/StatusBar.vue";
import ApplicationContext from "@/organisims/ApplicationContext.vue";
import Editor from "@/organisims/Editor.vue";

import { ctxMapState, InstanceStoreContext } from "@/store";

interface IData {
  applicationContextCtx: InstanceStoreContext;
  statusBarCtx: InstanceStoreContext;
}

export default Vue.extend({
  name: "ApplicationDetails",
  components: {
    StatusBar,
    ApplicationContext,
    Editor,
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
  methods: {
    async updateQueryParam(payload: Record<string, any>) {
      await this.$router
        .replace({
          query: Object.assign(
            {},
            { ...this.$router.currentRoute.query },
            payload,
          ),
        })
        .catch(() => {});
    },
    async removeQueryParam(payload: string[]) {
      const query = Object.assign({}, this.$route.query);
      for (const param of payload) {
        delete query[param];
      }
      await this.$router.replace({ query }).catch(() => {});
    },
    async wipeQueryParam() {
      await this.$router
        .replace({
          query: {},
        })
        .catch(() => {});
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
  },
  async beforeDestroy() {
    unregisterStatusBar(this.applicationContextCtx.instanceId);
    unregisterApplicationContext(this.applicationContextCtx);
  },
});
</script>
