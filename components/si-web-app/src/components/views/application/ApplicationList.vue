<template>
  <div
    id="application-list"
    class="flex flex-col flex-no-wrap application-list-bg-color"
  >
    <div class="flex flex-row h-10 mt-4 application-list-menu-bar">
      <button
        data-cy="new-application-button"
        class="h-8 px-2 mt-1 ml-4 text-white bg-teal-700 hover:bg-teal-600"
        @click="showModal()"
        type="button"
      >
        <div class="flex">
          <PlusSquareIcon size="1.25x" class="self-center text-gray-200" />
          <div class="ml-1 font-normal text-gray-100">new application</div>
        </div>
      </button>
    </div>

    <modal
      name="new-application"
      adaptive
      width="500"
      height="125"
      draggable
      styles="background-color:#313436"
    >
      <div class="flex flex-col">
        <div
          class="flex items-center justify-between pl-1 text-sm text-white bg-black"
        >
          <div>
            Create new application
          </div>
          <div>
            <button @click="hideModal" class="flex">
              <XIcon @click="hideModal"></XIcon>
            </button>
          </div>
        </div>

        <div class="p-4">
          <div class="flex flex-row mx-2 my-2">
            <div class="text-white">
              applicationName:
            </div>

            <input
              data-cy="new-application-form-application-name"
              class="ml-4 leading-tight text-gray-400 bg-transparent border-none appearance-none focus:outline-none input-bg-color"
              type="text"
              placeholder="application name"
              v-model="applicationName"
            />
          </div>
          <div class="flex flex-row">
            <button
              data-cy="new-application-form-create-button"
              class="w-16 mt-4 ml-4 text-white bg-teal-700 hover:bg-teal-600"
              @click="createApplication"
              type="button"
            >
              create
            </button>
          </div>
        </div>
      </div>
    </modal>

    <div v-if="applications">
      <div v-for="app in applications" :key="app.id">
        <router-link
          :to="applicationLink(app.id)"
          :data-cy="'application-list-link-' + app.name"
        >
          <ApplicationCard
            class="mx-8 my-4"
            :application="app"
            @click="goToApplication(app.id)"
          />
        </router-link>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { mapState } from "vuex";
import { registry } from "si-registry";
import { PlusSquareIcon, XIcon } from "vue-feather-icons";

import { RootStore } from "../../../store";
import ApplicationCard from "./ApplicationCard.vue";

interface Data {
  applicationName: string;
}

export default Vue.extend({
  name: "ApplicationList",
  components: {
    ApplicationCard,
    PlusSquareIcon,
    XIcon,
  },
  props: {
    organizationId: {
      type: String,
    },
    workspaceId: {
      type: String,
    },
  },
  data(): Data {
    return {
      applicationName: "",
    };
  },
  methods: {
    applicationLink(applicationId: string): Record<string, any> {
      const organization = this.$store.getters["organization/current"];
      const workspace = this.$store.getters["workspace/current"];
      return {
        name: "applicationDetails",
        params: {
          organizationId: organization.id,
          workspaceId: workspace.id,
          applicationId,
        },
      };
    },
    async createApplication() {
      let payload = {
        name: this.applicationName,
      };
      const newApp = await this.$store.dispatch("application/create", payload);
      this.hideModal();
      const workspace = this.$store.getters["workspace/current"];
      this.$router.push({
        name: "applicationDetails",
        params: {
          organizationId: workspace.siProperties.organizationId,
          workspaceId: workspace.id,
          applicationId: newApp.siStorable?.itemId,
        },
      });
    },
    showModal() {
      this.applicationName = "";
      this.$modal.show("new-application");
    },
    hideModal() {
      this.$modal.hide("new-application");
      this.applicationName = "";
    },
  },
  computed: {
    ...mapState({
      applications(state: RootStore): RootStore["application"]["list"] {
        return state.application.list;
      },
    }),
  },
  async mounted() {
    await this.$store.dispatch("application/list");
  },
});
</script>

<style type="text/css" scoped>
.application-list-menu-bar {
  background-color: #2d3031;
}
.application-list-bg-color {
  background-color: #212324;
}
.input-bg-color {
  background-color: #25788a;
}
</style>
