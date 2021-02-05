<template>
  <LoginForm @success="routeToHome" @signup="routeToSignup" />
</template>

<script lang="ts">
import Vue from "vue";

import DebugRouter from "@/atoms/DebugRoute.vue";
import LoginForm from "@/organisims/LoginForm.vue";

export default Vue.extend({
  name: "Login",
  components: {
    LoginForm,
  },
  methods: {
    routeToHome() {
      this.$router.push({ name: "home" });
    },
    routeToSignup() {
      this.$router.push({ name: "signup" });
    },
  },
});
</script>
