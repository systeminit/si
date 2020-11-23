<template>
  <div class="tooltip">
    <slot />
    <div ref="tooltip" class="w-auto p-2 pr-8 mt-1 border tooltip-text">
      <slot name="tooltip" />
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";

export default Vue.extend({
  name: "Tooltip",
  props: {
    alightRight: {
      type: Boolean,
      required: false,
    },
    offset: Number,
  },
  methods: {
    positionAlignRight() {
      // @ts-ignore
      let position = this.$refs.tooltip.offsetWidth - this.offset;
      // @ts-ignore
      this.$refs.tooltip.style.transform = "translateX(-" + position + "px)";
    },
  },
  mounted() {
    if (this.alightRight) {
      this.positionAlignRight();
    }
  },
});
</script>

<style scoped>
.tooltip .tooltip-text {
  visibility: hidden;
  /*text-align: center;*/
  /* padding: 2px 6px; */
  position: absolute;
  z-index: 100;
}

.tooltip:hover .tooltip-text {
  visibility: visible;
  border-color: #3a4145;
  background-color: #222629;
}
</style>
