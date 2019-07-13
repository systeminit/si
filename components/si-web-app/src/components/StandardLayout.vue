<template>
  <div>
    <GlobalToolbar />
    <slot></slot>
    <GlobalFooter />
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import GlobalToolbar from "@/components/GlobalToolbar.vue";
import GlobalFooter from "@/components/GlobalFooter.vue";

export default Vue.extend({
  name: "StandardLayout",
  components: {
    GlobalToolbar,
    GlobalFooter,
  },
});
</script>
