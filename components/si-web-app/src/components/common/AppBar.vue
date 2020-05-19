<template>
  <nav
    id="app-bar"
    class="flex items-center justify-between flex-no-wrap items-baseline h-12 bg-black"
  >
    <div class="flex self-center flex-shrink-0 ml-20">
      <span
        class="self-center flex-1 font-source-code-pro font-medium subpixel-antialiased text-white text-2xl tracking-tight align-middle"
        >system init</span
      >
    </div>

    <div class="flex self-center m-4">
      <div class="self-center flex-1 px-1 py-1 mx-2">
        <help-circle-icon size="1.5x" class="text-center text-white" />
      </div>

      <div class="self-center flex-1 text-center px-2 py-2 ml-2">
        <user-icon size="1.5x" class="text-center text-white" @click="onLogout"/>
      </div>
    </div>
  </nav>
</template>

<script>
import { UserIcon, HelpCircleIcon } from "vue-feather-icons";
import { auth } from "@/auth";

export default {
  name: "AppBar",
  components: {
    UserIcon,
    HelpCircleIcon,
  },
  methods: {
    onLogout(event) {
      console.log("logout clicked")
      auth.logout();
      this.$router.push({ name: 'signin' });
    }
  }
};
</script>

<style type="text/css" scoped>
.font-source-code-pro {
  font-family: "Source Code Pro", monospace;
}
</style>
