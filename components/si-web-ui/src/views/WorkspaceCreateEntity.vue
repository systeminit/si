<template>
  <StandardLayout>
    <Editor entityName="SSH Key"></Editor>
  </StandardLayout>
</template>

<script lang="ts">
import StandardLayout from "@/components/StandardLayout.vue";
import Editor from "@/components/Editor.vue";

export default {
  name: "workspaceCreateEntity",
  props: {
    organizationId: String,
    workspaceId: String,
    entityType: String,
  },
  components: {
    StandardLayout,
    Editor,
  },
};
</script>
