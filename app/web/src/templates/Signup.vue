<template>
  <div class="h-full w-full">
    <SignupForm @success="routeToLogin" @back-to-login="routeToLogin" />
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";

import SignupForm from "@/organisims/SignupForm.vue";

export default defineComponent({
  name: "Signup",
  components: {
    SignupForm,
  },
  methods: {
    routeToLogin() {
      this.$router.push({ name: "login" });
    },
  },
});
</script>
