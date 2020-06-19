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
    const organization = profile.organization.id;
    const workspace = profile.workspaceDefault.id;

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
