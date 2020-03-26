<template>
  <StandardLayout>
    <EntityShow :entityType="entityType" :entityId="entityId"></EntityShow>
  </StandardLayout>
</template>

<script lang="ts">
import StandardLayout from "@/components/StandardLayout.vue";
import EntityShow from "@/components/EntityShow.vue";

export default {
  name: "workspaceShowEntity",
  props: {
    organizationId: String,
    workspaceId: String,
    entityId: String,
    entityType: String,
  },
  components: {
    StandardLayout,
    EntityShow,
  },
};
</script>
