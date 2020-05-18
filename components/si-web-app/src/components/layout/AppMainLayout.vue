<template>
  <div id="app-main-layout" class="flex flex-col h-screen w-screen">
    <AppBar/>

    <slot/>
  </div>
</template>

<script>
import AppBar from "@/components/common/AppBar.vue";

export default {
  name: "AppMainLayout",
  components: {
    AppBar,
  },
};
</script>
