<template>
  <nav
    id="workspace-nav"
    class="flex flex-col flex-shrink-0 flex-no-wrap bg-primary w-56"
  >

    <!-- <WorkspaceSelector/> -->

    <Dropdown
      class=""
      :default=currentWorkspace
      :options=workspaces
    />

    <div class="text-white"> Changeset </div>
    <Dropdown
      class=""
      :default=currentWorkspace
      :options=workspaces
    />

    <div id="workspace-content" class="flex px-4 py-2 ml-2">
  
      <div
        class="self-center flex-1 font-source-code-pro font-medium subpixel-antialiased text-white text-l tracking-tight align-middle"
      >
        <router-link
          :to="{
            name: 'system',
            params: {
              systemId: 'demo',
              organizationId: organizationId,
              workspaceId: workspaceId,
            },
          }"
          >systems</router-link
        >
      </div>
    </div>
  </nav>
</template>

<script>
// import WorkspaceSelector from "./WorkspaceSelector.vue";
import Dropdown from "@/components/ui/Dropdown"

export default {
  name: "WorkspaceNav",
  components: {
    // WorkspaceSelector,
    Dropdown
  },
  props: {
    organizationId: {
      type: String,
    },
    workspaceId: {
      type: String,
    },
  },
  data() {
    return {
      workspaces: [
        "my workspace",
        "another workspace"
      ],
      currentWorkspace: "my workspace"
    }
  }
};
</script>
