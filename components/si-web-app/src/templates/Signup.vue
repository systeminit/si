<template>
  <SignupForm @success="routeToLogin" @back-to-login="routeToLogin" />
</template>

<script lang="ts">
import Vue from "vue";

import SignupForm from "@/organisims/SignupForm.vue";

export default Vue.extend({
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
