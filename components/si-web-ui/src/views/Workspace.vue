<template>
  <StandardLayout>
    <EntityList
      entityType="SSH Key"
      :organizationId="organizationId"
      :workspaceId="workspaceId"
    >
    </EntityList>
  </StandardLayout>
</template>

<script lang="ts">
import StandardLayout from "@/components/StandardLayout.vue";
import EntityList from "@/components/EntityList.vue";

export default {
  name: "workspace",
  props: {
    organizationId: String,
    workspaceId: String,
  },
  components: {
    StandardLayout,
    EntityList,
    //Editor,
  },
};
</script>
