<template>
  <StandardLayout>
    <v-card>
      <v-card-title>Select a Workpsace from the sidebar</v-card-title>
    </v-card>
  </StandardLayout>
</template>

<script>
import StandardLayout from "@/components/StandardLayout.vue";
import Editor from "@/components/Editor.vue";

export default {
  name: "home",
  components: {
    StandardLayout,
    //Editor,
  },
};
</script>
