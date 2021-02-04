<template>
  <div>
    <SignupWad />
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import SignupWad from "@/wads/SignupWad.vue";

export default Vue.extend({
  name: "SignupView",
  components: { SignupWad },
});
</script>
