<template>
  <Loading
    :active.sync="isLoading"
    loader="bars"
    color="#ffffff"
    backgroundColor="#000000"
    :opacity="opacity"
  ></Loading>
</template>

<script lang="ts">
import Vue from "vue";
import Loading from "vue-loading-overlay";
import "vue-loading-overlay/dist/vue-loading.css";

export default Vue.extend({
  name: "SiLoader",
  components: {
    // @ts-ignore
    Loading,
  },
  data() {
    return {
      opacity: 1,
    };
  },
  props: {
    isLoading: {
      type: Boolean,
    },
  },
});
</script>
