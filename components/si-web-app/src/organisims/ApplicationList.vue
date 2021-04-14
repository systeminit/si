<template>
  <div class="flex w-full h-full">
    <SiError
      testId="application-list-wad-error"
      :message="errorMessage"
      @clear="clearErrorMessage"
    />

    <div class="flex flex-col w-full" v-if="applicationList.length">
      <div
        v-for="appEntry in applicationList"
        :key="appEntry.application.id"
        class="mb-6"
      >
        <router-link :to="cardLink(appEntry.application.id)">
          <ApplicationDetailCard
            :linkTo="cardLink(appEntry.application.id)"
            :applicationEntry="appEntry"
            :cardLink="cardLink(appEntry.application.id)"
          />
        </router-link>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import { mapState } from "vuex";
import { RawLocation } from "vue-router";

import { Workspace } from "@/api/sdf/model/workspace";
import { Organization } from "@/api/sdf/model/organization";

import SiError from "@/atoms/SiError.vue";
import ApplicationDetailCard from "@/molecules/ApplicationDetailCard.vue";

import {
  ApplicationStore,
  ISetListApplicationsReply,
  ISetListApplicationsRequest,
} from "@/store/modules/application";
import { SessionStore } from "@/store/modules/session";

interface IData {
  errorMessage: string;
  isLoading: boolean;
}

export default Vue.extend({
  name: "ApplicationList",
  props: {
    linkTo: {
      type: String,
    },
  },
  data(): IData {
    return {
      errorMessage: "",
      isLoading: true,
    };
  },
  components: {
    SiError,
    ApplicationDetailCard,
  },
  computed: {
    ...mapState({
      applicationList: (state: any): ApplicationStore["applicationList"] =>
        state.application.applicationList,
      currentWorkspace(
        state: Record<string, any>,
      ): SessionStore["currentWorkspace"] {
        return state["session"]["currentWorkspace"];
      },
      currentOrganization(
        state: Record<string, any>,
      ): SessionStore["currentOrganization"] {
        return state["session"]["currentOrganization"];
      },
    }),
  },
  methods: {
    cardLink(applicationId: string): RawLocation | null {
      if (this.currentWorkspace && this.currentOrganization) {
        return {
          name: "applicationDetails",
          params: {
            workspaceId: this.currentWorkspace.id,
            organizationId: this.currentOrganization.id,
            applicationId: applicationId,
          },
        };
      }
      return null;
    },
    async setListApplications(
      workspace: SessionStore["currentWorkspace"],
    ): Promise<void> {
      if (workspace) {
        this.isLoading = true;
        let reply: ISetListApplicationsReply = await this.$store.dispatch(
          "application/setListApplications",
          {
            workspaceId: workspace.id,
          },
        );
        if (reply.error) {
          this.errorMessage = reply.error.message;
        }
        this.isLoading = false;
      }
    },
    clearErrorMessage() {
      this.errorMessage = "";
    },
  },
  async created() {
    await this.$store.dispatch("application/activate", "ApplicationListWad");
    await this.setListApplications(this.currentWorkspace);
  },
  async beforeDestroy() {
    await this.$store.dispatch("application/deactivate", "ApplicationListWad");
  },
  watch: {
    async currentWorkspace(workspace: SessionStore["currentWorkspace"]) {
      await this.setListApplications(workspace);
    },
  },
});
</script>
