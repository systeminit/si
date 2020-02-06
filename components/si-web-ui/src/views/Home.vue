<template>
  <StandardLayout>
    <Editor entityName="SSH Key"></Editor>
  </StandardLayout>
</template>

<script>
import StandardLayout from "@/components/StandardLayout.vue";
import Editor from "@/components/Editor.vue";

export default {
  name: "home",
  components: {
    StandardLayout,
    Editor,
  },
};
</script>
