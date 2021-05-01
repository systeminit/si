<template>
  <div
    id="applications"
    class="flex flex-col w-full h-full select-none page-background"
  >
    <div
      id="applications-header"
      class="flex flex-row items-center justify-between flex-grow-0 flex-shrink-0 h-12 header-background"
    >
      <div class="mt-1 ml-8 font-medium align-middle">Applications</div>
      <div class="mt-1 mr-8 align-middle ">
        <SiButton icon="plus" label="New" size="xs" @click.native="showModal" />
      </div>
      <modal
        name="new-application"
        adaptive
        width="400"
        height="auto"
        draggable
        styles="background-color:#313436"
      >
        <div class="flex flex-col">
          <div
            class="flex items-center justify-between pl-1 text-sm text-white bg-black"
          >
            <div>Create new application</div>
            <div>
              <button
                @click="hideModal"
                class="flex"
                aria-label="Close New Application Modal"
              >
                <XIcon @click="hideModal"></XIcon>
              </button>
            </div>
          </div>

          <div class="flex flex-row w-full p-2">
            <ApplicationCreate @cancel="hideModal" @submit="hideModal" />
          </div>
        </div>
      </modal>
    </div>

    <div class="flex h-full mx-4 mt-4 overflow-y-scroll">
      <ApplicationList linkTo="applicationEditor" />
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";

import { XIcon } from "vue-feather-icons";
import SiButton from "@/atoms/SiButton.vue";
import ApplicationList from "@/organisims/ApplicationList.vue";
import ApplicationCreate from "@/organisims/ApplicationCreate.vue";

export default Vue.extend({
  name: "Application",
  components: {
    ApplicationList,
    ApplicationCreate,
    SiButton,
    XIcon,
  },
  methods: {
    showModal() {
      this.$modal.show("new-application");
    },
    hideModal() {
      this.$modal.hide("new-application");
    },
  },
});
</script>

<style type="text/css" scoped>
.input-bg-color {
  background-color: #25788a;
}

.page-background {
  background-color: #1e1e1e;
}

.header-background {
  background-color: #171717;
}
</style>
