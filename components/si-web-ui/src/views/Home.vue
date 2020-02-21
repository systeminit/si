<template>
  <Workspace :organizationId="organizationId" :workspaceId="workspaceId">
  </Workspace>
</template>

<script lang="ts">
import Workspace from "@/views/Workspace.vue";
import { auth } from "@/auth";

export default {
  name: "home",
  data() {
    const profile = auth.getProfile();
    const organization = (profile &&
      profile.billingAccount &&
      profile.billingAccount.organizations &&
      profile.billingAccount.organizations.items &&
      profile.billingAccount.organizations.items[0]) || { name: "busted" };
    const workspace = (organization &&
      organization.workspaces &&
      organization.workspaces.items &&
      organization.workspaces.items[0]) || { name: "busted" };

    return {
      organizationId: organization.id,
      workspaceId: workspace.id,
    };
  },
  components: {
    Workspace,
  },
};
</script>
