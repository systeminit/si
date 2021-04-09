<template>
  <div
    id="secret"
    class="flex flex-col w-full h-full select-none page-background"
  >
    <div
      class="flex flex-row items-center justify-between flex-grow-0 flex-shrink-0 h-12 header-background"
    >
      <div class="mt-1 ml-8 font-medium align-middle">Secrets</div>

      <div class="mt-1 mr-8 align-middle ">
        <SiButton icon="plus" label="New" size="xs" @click.native="showModal" />
      </div>

      <modal
        name="new-secret"
        adaptive
        width="500"
        height="275"
        draggable
        styles="background-color:#313436"
      >
        <div class="flex flex-col">
          <div
            class="flex items-center justify-between pl-1 text-sm text-white bg-black"
          >
            <div>Create new secret</div>
            <div>
              <button
                @click="hideModal"
                class="flex"
                aria-label="Close New Secret Modal"
              >
                <XIcon @click="hideModal"></XIcon>
              </button>
            </div>
          </div>

          <div class="p-4">
            <div class="flex flex-row mx-2 my-2">
              <SecretCreate @cancel="hideModal" @submit="hideModal" />
            </div>
          </div>
        </div>
      </modal>
    </div>

    <div class="flex h-full mx-12 mt-4 overflow-hidden">
      <SecretList />
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import SiButton from "@/atoms/SiButton.vue";
import SecretCreate from "@/organisims/SecretCreate.vue";
import SecretList from "@/organisims/SecretList.vue";
import { XIcon } from "vue-feather-icons";

export default Vue.extend({
  name: "Secret",
  components: {
    SecretCreate,
    SecretList,
    SiButton,
    XIcon,
  },
  methods: {
    showModal() {
      this.$modal.show("new-secret");
    },
    hideModal() {
      this.$modal.hide("new-secret");
    },
  },
});
</script>

<style type="text/css" scoped>
.page-background {
  background-color: #1e1e1e;
}

.header-background {
  background-color: #171717;
}
</style>
