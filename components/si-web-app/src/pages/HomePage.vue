<template>
  <WorkspacePage :organizationId="organizationId" :workspaceId="workspaceId" />
</template>

<script lang="ts">
import Vue from "vue";
import WorkspacePage from "@/pages/WorkspacePage/index.vue";

interface HomePageData {
  organizationId: string;
  workspaceId: string;
}

export default Vue.extend({
  name: "home",
  data(): HomePageData {
    const profile = this.$store.state.user.auth.profile;
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
});
</script>
