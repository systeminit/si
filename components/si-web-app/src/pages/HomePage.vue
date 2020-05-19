<template>
  <WorkspacePage :organizationId="organizationId" :workspaceId="workspaceId" />
</template>

<script lang="ts">
import WorkspacePage from "@/pages/WorkspacePage/index.vue";
import { auth } from "@/auth";

export default {
  name: "home",
  data() {
    const profile = auth.getProfile();
    const organization = profile.organization;
    const workspace = profile.workspaceDefault;

    console.log(profile);

    return {
      organizationId: organization.id,
      workspaceId: workspace.id,
    };
  },
  components: {
    WorkspacePage,
  },
};
</script>
