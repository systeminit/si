<template>
  <div id="application-list" class="application-list-bg-color flex flex-col flex-no-wrap">
    <div class="flex flex-row mt-4 h-10 application-list-menu-bar">
      <button
        class="bg-teal-700 ml-4 px-2 h-8 mt-1 text-white hover:bg-teal-600"
        @click="showModal()"
        type="button"
      >
        new application
      </button>
    </div>

    <modal
      name="hello-world"
      width="500"
      height="100"
      draggable
      styles="background-color:#313436"
    >
      <div class="flex flex-col">
        <div class="flex flex-row mx-2 my-2">
          <div class="text-white">
            Application Name:
          </div>

          <input
            class="appearance-none bg-transparent border-none ml-4 text-gray-400 leading-tight focus:outline-none"
            type="text"
            placeholder="application name"
            v-model="applicationName"
          />

        </div>
        <button
          class="bg-teal-700 ml-4 mt-4 w-16 text-white hover:bg-teal-600"
          @click="newApp(applicationName)"
          type="button"
        >
          create
        </button>
      </div>
    </modal>

    <div v-if="applications">

      <div v-for="app in applications" :key="app.id">
          
          <ApplicationCard
            class="mx-8 my-4"
            :application=app
            :session=session
          />

      </div>
    </div>
  </div>
</template>

<script>
import { registry } from "si-registry";
import ApplicationCard from "./ApplicationCard.vue"

export default {
  name: "ApplicationList",
  components: {
    ApplicationCard
  },
  props: {
    organizationId: {
      type: String,
    },
    workspaceId: {
      type: String,
    },
  },
  data() {
    return {
      applicationName: "",
      session: {
        organizationId: this.organizationId,
        workspaceId: this.workspaceId
      }
    };
  },
  methods: {
    newApp(name) {
      let payload = {
        id: Date.now().toString(),
        name: name,
      };
      this.$store.dispatch("applications/addApplication", payload);
      this.hideModal();
    },
    showModal() {
      this.$modal.show("hello-world");
    },
    hideModal() {
      this.$modal.hide("hello-world");
    },
  },
  computed: {
    applications() {
      return this.$store.getters["applications/list"];
    },
  },
  created() {},
};
</script>

<style type="text/css" scoped>
.application-list-menu-bar {
  background-color: #2d3031;
}
.application-list-bg-color {
  background-color: #212324;
}
</style>
