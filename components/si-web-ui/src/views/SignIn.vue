<template>
  <CenteredLayout>
    <Login></Login>
  </CenteredLayout>
</template>

<script lang="ts">
import Vue from "vue";

import Login from "@/components/Login.vue";
import CenteredLayout from "@/components/CenteredLayout.vue";

export default Vue.extend({
  name: "signin",
  components: {
    Login,
    CenteredLayout,
  },
});
</script>
