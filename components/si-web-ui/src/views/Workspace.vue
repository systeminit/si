<template>
  <StandardLayout>
    <EntityList
      entityName="AWS EKS Cluster Runtime"
      entityType="awsEksClusterRuntime"
      :organizationId="organizationId"
      :workspaceId="workspaceId"
    >
    </EntityList>

    <EntityList
      entityName="SSH Key"
      entityType="sshKey"
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
