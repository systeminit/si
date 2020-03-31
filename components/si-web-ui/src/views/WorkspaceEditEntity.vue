<template>
  <StandardLayout>
    <Editor
      operation="edit"
      :entityType="entityType"
      :entityId="entityId"
      :workspaceId="workspaceId"
      :organizationId="organizationId"
    ></Editor>
  </StandardLayout>
</template>

<script lang="ts">
import StandardLayout from "@/components/StandardLayout.vue";
import Editor from "@/components/Editor.vue";

export default {
  name: "workspaceEditEntity",
  props: {
    organizationId: String,
    workspaceId: String,
    entityType: String,
    entityId: String,
  },
  components: {
    StandardLayout,
    Editor,
  },
};
</script>
