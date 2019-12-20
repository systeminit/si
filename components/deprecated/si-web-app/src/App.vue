<template>
  <v-app id="si">
    <v-content>
      <router-view></router-view>
    </v-content>
  </v-app>
</template>

<script>
export default {
  name: "App",
  async created() {
    try {
      await this.$auth.renewTokens();
    } catch {
      // You can ignore errors safely, because we get some when
      // processing the callback url
    }
  },
};
</script>
