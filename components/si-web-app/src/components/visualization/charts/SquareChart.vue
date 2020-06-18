<template>
  <svg :width="width" :height="height">
    <rect :width="width" :height="height" :style="style"/>
  </svg>
</template>

<script>
export default {
  name: "SquareChart",
  props: {
    width: {
      type: Number,
      default: 14
    },
    height: {
      type: Number,
      default: 14
    },
    rgbColor: {
      type: String,
      default: "0,0,255"
    }
  },
  computed: {
    style() {
      return ("fill:rgb(" + this.rgbColor + ")")
    }
  }, 
};
</script>