<template>
  <div class="tooltip" @click="toggleVisibility()">
    <slot />
    <div
      ref="tooltip"
      class="w-auto p-2 pr-8 mt-1 border tooltip-text"
      :class="{
        'tooltip-sticky': isVisible,
      }"
    >
      <slot name="tooltip" />
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";

interface Data {
  isVisible: boolean;
}

export default Vue.extend({
  name: "Tooltip",
  props: {
    alignRight: {
      type: Boolean,
      required: false,
    },
    sticky: {
      type: Boolean,
      required: false,
    },
    offset: {
      type: Number,
      required: false,
    },
  },
  data(): Data {
    return {
      isVisible: false,
    };
  },
  methods: {
    positionAlignRight() {
      // @ts-ignore
      let position = this.$refs.tooltip.offsetWidth - this.offset;
      // @ts-ignore
      this.$refs.tooltip.style.transform = "translateX(-" + position + "px)";
    },
    toggleVisibility() {
      if (this.sticky) {
        this.isVisible = !this.isVisible;
      }
    },
  },
  mounted() {
    if (this.alignRight) {
      this.positionAlignRight();
    }
  },
});
</script>

<style scoped>
.tooltip .tooltip-text {
  visibility: hidden;
  position: absolute;
  z-index: 100;
  border-color: #3a4145;
  background-color: #222629;
}

.tooltip:hover .tooltip-text {
  visibility: visible;
}

.tooltip .tooltip-sticky {
  visibility: visible;
}
</style>
