<template>
  <nav
    id="workspace-nav"
    class="flex items-center justify-between flex-col flex-shrink-0 flex-no-wrap bg-primary w-56"
  >
    <div class="flex justify-center items-center h-12 nav-header w-full">
      <div class="text-white text-lg font-medium">The System Initiative</div>
    </div>

    <div class="flex items-center justify-center mt-6 h-10">
      <!-- <Dropdown
        class="w-full mx-4"
        :optionDefault="currentWorkspace"
        :optionList="workspaces"
        menuStyle="standard-rs"
      /> -->
      <div class="color-grey-medium text-lg font-normal">Demo Workspace</div>
      <menu-icon size="1.5x" class="ml-3 color-grey-medium" />
    </div>

    <svg class="mt-6" viewBox="0 0 100 1" xmlns="http://www.w3.org/2000/svg">
      <line x1="5" y1="0" x2="95" y2="0" stroke="#313639" />
    </svg>

    <div
      id="workspace-content"
      class="flex flex-col flex-grow mt-12 self-end mr-8"
    >
      <div
        class="self-end font-source-code-pro font-normal subpixel-antialiased text-white text-md tracking-tight"
      >
        <router-link
          data-cy="application-nav-link"
          :to="{
            name: 'application',
            params: {
              applicationId: 'my-app',
              organizationId: organizationId,
              workspaceId: workspaceId,
            },
          }"
        >
          <div class="flex flex-row items-center color-grey-light">
            <share-2-icon size="1.15x" class="mr-3"></share-2-icon>
            <div class="text-base font-normal">Applications</div>
          </div>
        </router-link>
      </div>

      <div
        class="mt-6 self-end font-source-code-pro font-normal subpixel-antialiased text-white text-md tracking-tight"
      >
        <router-link
          data-cy="system-nav-link"
          :to="{
            name: 'system',
            params: {
              systemId: 'demo',
              organizationId: organizationId,
              workspaceId: workspaceId,
            },
          }"
        >
          <div class="flex flex-row items-center color-disabled">
            <code-icon size="1.3x" class="mr-3" />
            <div class="text-base font-normal">Systems (dev)</div>
          </div>
        </router-link>
      </div>

      <div
        class="mt-6 self-end font-source-code-pro font-normal subpixel-antialiased text-white text-md tracking-tight"
      >
        <router-link
          data-cy="global-nav-link"
          :to="{
            name: 'global',
            params: {
              organizationId: organizationId,
              workspaceId: workspaceId,
            },
          }"
        >
          <div class="flex flex-row items-center color-disabled">
            <code-icon size="1.3x" class="mr-3" />
            <div class="text-base font-normal">Global (dev)</div>
          </div>
        </router-link>
      </div>
    </div>

    <svg class="mt-6" viewBox="0 0 100 1" xmlns="http://www.w3.org/2000/svg">
      <line x1="5" y1="0" x2="95" y2="0" stroke="#313639" />
    </svg>

    <div class="flex justify-end items-center h-12 color-grey-medium w-full">
      <div class="flex self-center mr-8">
        <div class="self-center flex-1 text-center">
          <user-icon
            size="1.2x"
            class="text-center color-grey-light"
            @click="onLogout"
          />
        </div>
      </div>
    </div>
  </nav>
</template>

<script lang="ts">
// import WorkspaceSelector from "./WorkspaceSelector.vue";
// import Dropdown from "@/components/ui/Dropdown/index.vue";
import { MenuIcon } from "vue-feather-icons";
import { Share2Icon } from "vue-feather-icons";
import { CodeIcon } from "vue-feather-icons";
import { UserIcon } from "vue-feather-icons";

export default {
  name: "WorkspaceNav",
  components: {
    // WorkspaceSelector,
    // Dropdown,
    MenuIcon,
    Share2Icon,
    CodeIcon,
    UserIcon,
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
  methods: {
    async onLogout(event) {
      console.log("logout clicked");
      await this.$store.dispatch("user/logout");
      this.$router.push({ name: "signin" });
    },
  },
};
</script>

<style scoped>
.router-link-active {
  @apply font-semibold;
}

.nav-header {
  background-color: #0d0d0d;
}

.color-disabled {
  color: #4a4b4c;
}

.color-grey-medium {
  color: #949698;
}

.color-grey-light {
  color: #c7cacd;
}
</style>
