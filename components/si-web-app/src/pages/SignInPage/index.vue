<template>
  <CenteredLayout>
    <SignInForm/>
  </CenteredLayout>
</template>

<script>
import CenteredLayout from "@/components/layout/CenteredLayout.vue";
import SignInForm from "./SignInForm.vue";

export default {
  name: "SignInPage",
  components: {
    CenteredLayout,
    SignInForm
  },
}
</script>