<template>
  <div id="secret" class="flex flex-col flex-no-wrap select-none">
    <div class="flex flex-row h-4"></div>
    <div class="flex flex-row h-10 bg-gray-700">
      <div class="pt-1 pl-2 align-middle">
        <SiButton
          icon="plus"
          label="New Secret"
          size="base"
          @click.native="showModal"
        />
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

    <SecretList />
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
