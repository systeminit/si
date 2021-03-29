<template>
  <div class="tooltip">
    <div @click="toggleVisibility()">
      <slot />
    </div>
    <div
      ref="tooltip"
      class="w-auto p-2 pr-8 mt-1 border tooltip-text"
      :class="hoverClasses"
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
    onlyOnClick: {
      type: Boolean,
      required: false,
    },
  },
  data(): Data {
    return {
      isVisible: false,
    };
  },
  computed: {
    hoverClasses(): Record<string, boolean> {
      let classes: Record<string, boolean> = {};
      if (this.isVisible) {
        classes["tooltip-sticky"] = true;
      }
      if (!this.onlyOnClick) {
        classes["tooltip-hover"] = true;
      }
      return classes;
    },
  },
  methods: {
    positionAlignRight() {
      // @ts-ignore
      let position = this.$refs.tooltip.offsetWidth - this.offset;
      // @ts-ignore
      this.$refs.tooltip.style.transform = "translateX(-" + position + "px)";
    },
    toggleVisibility() {
      if (this.sticky || this.onlyOnClick) {
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

.tooltip:hover .tooltip-text .tooltip-hover {
  visibility: visible;
}

.tooltip .tooltip-sticky {
  visibility: visible;
}
</style>
