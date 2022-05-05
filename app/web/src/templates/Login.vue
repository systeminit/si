<template>
  <div class="h-full w-full">
    <LoginForm @success="routeToHome" @signup="routeToSignup" />
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";

import LoginForm from "@/organisims/LoginForm.vue";

export default defineComponent({
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
