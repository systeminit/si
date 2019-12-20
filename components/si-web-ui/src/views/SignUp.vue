<template>
  <CenteredLayout>
    <SignUpForm></SignUpForm>
  </CenteredLayout>
</template>

<script lang="ts">
import Vue from "vue";
import SignUpForm from "@/components/SignUpForm.vue";
import CenteredLayout from "@/components/CenteredLayout.vue";

export default Vue.extend({
  name: "signup",
  components: {
    CenteredLayout,
    SignUpForm,
  },
});
</script>
