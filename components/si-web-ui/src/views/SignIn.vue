<template>
  <CenteredLayout>
    <Login2></Login2>
  </CenteredLayout>
</template>

<script lang="ts">
import Vue from "vue";

import Login2 from "@/components/billingAccount/login.vue";
import CenteredLayout from "@/components/CenteredLayout.vue";

export default Vue.extend({
  name: "signin",
  components: {
    Login2,
    CenteredLayout,
  },
});
</script>
