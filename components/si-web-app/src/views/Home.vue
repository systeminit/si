<template>
  <StandardLayout>
    <v-container fluid fill-height>
      <v-layout justify-center align-center>
        <v-flex text-xs-center>
          <img alt="Vue logo" src="../assets/logo.png" />
        </v-flex>
      </v-layout>
    </v-container>
  </StandardLayout>
</template>

<script lang="ts">
import Vue from "vue";
import StandardLayout from "@/components/StandardLayout.vue";

export default Vue.extend({
  name: "home",
  components: {
    StandardLayout,
  },
});
</script>
