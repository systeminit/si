<template>
  <modal
    :name="name"
    draggable
    height="auto"
    styles="background-color:#313436"
    @before-close="beforeClose"
  >
    <div class="flex flex-col overflow-visible">
      <div
        class="flex items-center justify-between pl-1 text-sm text-white bg-black"
      >
        <div>
          {{ title }}
        </div>
        <div>
          <button @click="hideModal" class="flex">
            <XIcon @click="hideModal"></XIcon>
          </button>
        </div>
      </div>

      <div class="h-full p-4 overflow-visible">
        <div class="flex flex-row mx-2 my-2 overflow-visible">
          <div class="w-full overflow-visible text-white">
            <slot />
          </div>
        </div>
        <div class="flex flex-row justify-end w-full mr-2">
          <div>
            <slot name="buttons" />
          </div>
        </div>
      </div>
    </div>
  </modal>
</template>

<script lang="ts">
import Vue from "vue";
import { XIcon } from "vue-feather-icons";

export default Vue.extend({
  name: "SiModal",
  components: {
    XIcon,
  },
  props: {
    name: {
      type: String,
      required: true,
    },
    title: {
      type: String,
      required: true,
    },
  },
  methods: {
    hideModal() {
      this.$modal.hide(this.name);
      this.$emit("hideModal", this.name);
    },
    beforeClose(event: any) {
      this.$emit("hideModal", this.name);
    },
  },
});
</script>
