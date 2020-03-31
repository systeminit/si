<template>
  <StandardLayout>
    <EntityCreate
      :entityType="entityType"
      :organizationId="organizationId"
      :workspaceId="workspaceId"
    ></EntityCreate>
  </StandardLayout>
</template>

<script lang="ts">
import StandardLayout from "@/components/StandardLayout.vue";
import EntityCreate from "@/components/entity/create.vue";

export default {
  name: "workspaceCreateEntity",
  props: {
    organizationId: String,
    workspaceId: String,
    entityType: String,
  },
  components: {
    StandardLayout,
    EntityCreate,
  },
};
</script>
