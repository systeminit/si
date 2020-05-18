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
    console.log(profile);
    const organization = (profile &&
      profile.billingAccount &&
      profile.billingAccount.associations.organizations &&
      profile.billingAccount.associations.organizations.items &&
      profile.billingAccount.associations.organizations.items[0]) || {
      name: "busted",
    };

    // Looks like workspcaces aren't under organization?
    // const workspace = (organization &&
    //   organization.workspaces &&
    //   organization.workspaces.items &&
    //   organization.workspaces.items[0]) || { name: "busted" };
    const workspace = (organization &&
      profile.workspaces &&
      profile.workspaces &&
      profile.workspaces[0]) || { name: "busted" };

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
