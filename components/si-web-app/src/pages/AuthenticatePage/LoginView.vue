<template>
  <LoginWad />
</template>

<script lang="ts">
import Vue from "vue";
import LoginWad from "@/wads/LoginWad.vue";

export default Vue.extend({
  name: "LoginView",
  components: {
    LoginWad,
  },
});
</script>
