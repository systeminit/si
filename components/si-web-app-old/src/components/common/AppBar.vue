<template>
  <nav
    id="app-bar"
    class="flex flex-no-wrap items-center items-baseline justify-between h-12 bg-black"
  >
    <div class="flex self-center flex-shrink-0 ml-20">
      <span
        class="self-center flex-1 text-2xl subpixel-antialiased font-medium tracking-tight text-white align-middle font-source-code-pro"
        >system init</span
      >
    </div>

    <div class="flex self-center m-4">
      <div class="self-center flex-1 px-1 py-1 mx-2">
        <help-circle-icon size="1.5x" class="text-center text-white" />
      </div>

      <div class="self-center flex-1 px-2 py-2 ml-2 text-center">
        <user-icon
          size="1.5x"
          class="text-center text-white"
          @click="onLogout"
        />
      </div>
    </div>
  </nav>
</template>

<script>
import { UserIcon, HelpCircleIcon } from "vue-feather-icons";

export default {
  name: "AppBar",
  components: {
    UserIcon,
    HelpCircleIcon,
  },
  methods: {
    async onLogout(event) {
      await this.$store.dispatch("user/logout");
      this.$router.push({ name: "signin" });
    },
  },
};
</script>

<style type="text/css" scoped>
.font-source-code-pro {
  font-family: "Source Code Pro", monospace;
}
</style>
