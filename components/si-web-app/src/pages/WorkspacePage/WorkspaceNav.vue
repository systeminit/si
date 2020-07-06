<template>
  <nav
    id="workspace-nav"
    class="flex flex-col flex-shrink-0 flex-no-wrap bg-primary w-56"
  >
    <div class="flex justify-end mt-6">
      <Dropdown
        class="w-full mx-4"
        :optionDefault="currentWorkspace"
        :optionList="workspaces"
        menuStyle="standard-rs"
      />
    </div>

    <div id="workspace-content" class="flex flex-col mt-10 self-end mr-4">
      <div
        class="self-end font-source-code-pro font-medium subpixel-antialiased text-white text-l tracking-tight"
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

      <div
        class=" font-source-code-pro font-normal subpixel-antialiased text-white text-md tracking-tight"
      >
        <router-link
          :to="{
            name: 'application',
            params: {
              applicationId: 'my-app',
              organizationId: organizationId,
              workspaceId: workspaceId,
            },
          }"
          >applications</router-link
        >
      </div>
    </div>
  </nav>
</template>

<script>
// import WorkspaceSelector from "./WorkspaceSelector.vue";
import Dropdown from "@/components/ui/Dropdown";

export default {
  name: "WorkspaceNav",
  components: {
    // WorkspaceSelector,
    Dropdown,
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
      workspaces: ["my workspace", "another workspace"],
      currentWorkspace: "my workspace",
    };
  },
};
</script>
