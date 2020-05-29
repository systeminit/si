<template>
  <nav
    id="workspace-nav"
    class="flex flex-col flex-shrink-0 flex-no-wrap bg-primary w-56 "
  >
    <div id="workspace-switcher" class="flex px-4 py-2 m-2">
      <span
        class="self-center flex-1 font-source-code-pro font-medium subpixel-antialiased text-white text-xl tracking-tight align-middle"
        >Workspace</span
      >

      <div class="flex-1 px-2 py-2 ml-2">
        <menu-icon size="1.5x" class="text-white"></menu-icon>
      </div>
    </div>

    <div id="workspace-content" class="flex px-4 py-2 m-2">
  
      <span
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
      </span>
    </div>
  </nav>
</template>

<script>
import { MenuIcon } from "vue-feather-icons";

export default {
  name: "WorkspaceNav",
  components: {
    MenuIcon,
  },
  props: {
    organizationId: {
      type: String,
    },
    workspaceId: {
      type: String,
    },
  },
};
</script>
