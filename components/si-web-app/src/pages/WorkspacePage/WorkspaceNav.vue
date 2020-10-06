<template>
  <nav
    id="workspace-nav"
    class="flex flex-col flex-no-wrap items-center justify-between flex-shrink-0 bg-primary w-54"
  >
    <div class="flex justify-center w-full h-13 nav-header">
      <div class="flex items-center justify-between w-10/12">
        <SysinitIcon :size="1.4"> </SysinitIcon>
        <div class="text-xl font-medium text-white">System Init</div>
      </div>
    </div>

    <div class="flex justify-center w-full h-10 mt-6">
      <div class="flex items-center">
        <div class="text-base font-normal color-grey-medium">
          Demo Workspace
        </div>
        <menu-icon size="1.5x" class="ml-3 color-grey-medium" />
      </div>
    </div>

    <svg class="mt-6" viewBox="0 0 100 1" xmlns="http://www.w3.org/2000/svg">
      <line x1="5" y1="0" x2="95" y2="0" stroke="#313639" />
    </svg>

    <div id="workspace-content" class="flex flex-col flex-grow w-full mt-12">
      <div>
        <!-- Dashboard Link -->
        <div class="flex justify-center w-full h-10">
          <div class="w-9/12 cursor-pointer">
            <div
              class="flex items-center justify-start color-disabled focus:text-white"
            >
              <activity-icon size="1.1x" class="mr-3" />
              <div
                class="text-sm subpixel-antialiased font-normal tracking-tight font-source-code-pro"
              >
                Dashboard
              </div>
            </div>
          </div>
        </div>

        <!-- Applicasions Link -->
        <div class="container-link">
          <router-link
            class="w-9/12"
            data-cy="application-nav-link"
            :to="{
              name: 'application',
              params: {
                applicationId: 'my-app',
                organizationId: organization.id,
                workspaceId: workspace.id,
              },
            }"
          >
            <div class="flex items-center justify-start color-grey-light">
              <code-icon size="1.1x" class="mr-3" />
              <div
                class="text-sm subpixel-antialiased font-normal tracking-tight font-source-code-pro"
              >
                Applications
              </div>
            </div>
          </router-link>
        </div>

        <!-- Systems Link -->
        <div class="container-link">
          <div class="w-9/12 cursor-pointer">
            <div
              class="flex items-center justify-start color-disabled focus:text-white"
            >
              <share-2-icon size="1.2x" class="mr-3 transform rotate-90" />
              <div
                class="text-sm subpixel-antialiased font-normal tracking-tight font-source-code-pro"
              >
                Systems
              </div>
            </div>
          </div>
        </div>

        <!-- Components Link -->
        <div class="container-link">
          <div class="w-9/12 cursor-pointer">
            <div
              class="flex items-center justify-start color-disabled focus:text-white"
            >
              <box-icon size="1.1x" class="mr-3" />
              <div
                class="text-sm subpixel-antialiased font-normal tracking-tight font-source-code-pro"
              >
                Components
              </div>
            </div>
          </div>
        </div>

        <!-- Resources Link -->
        <div class="container-link">
          <div class="w-9/12 cursor-pointer">
            <div
              class="flex items-center justify-start color-disabled focus:text-white"
            >
              <grid-icon size="1.2x" class="mr-3" />
              <div
                class="text-sm subpixel-antialiased font-normal tracking-tight font-source-code-pro"
              >
                Resources
              </div>
            </div>
          </div>
        </div>

        <!-- Environment Link  AKA computing environment -->
        <div class="container-link">
          <div class="w-9/12 cursor-pointer">
            <div
              class="flex items-center justify-start color-disabled focus:text-white"
            >
              <layers-icon size="1.1x" class="mr-3" />
              <div
                class="text-sm subpixel-antialiased font-normal tracking-tight font-source-code-pro"
              >
                Environment
              </div>
            </div>
          </div>
        </div>

        <!-- Catalogue Link -->
        <div class="container-link">
          <div class="w-9/12 cursor-pointer">
            <div
              class="flex items-center justify-start color-disabled focus:text-white"
            >
              <book-open-icon size="1.1x" class="mr-3" />
              <div
                class="text-sm subpixel-antialiased font-normal tracking-tight font-source-code-pro"
              >
                Catalogue
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="flex flex-col justify-end flex-grow">
        <!-- Settings Link -->
        <div class="container-link">
          <div class="w-9/12 cursor-pointer">
            <div
              class="flex items-center justify-start color-disabled focus:text-white"
            >
              <settings-icon size="1.1x" class="mr-3" />
              <div
                class="text-sm subpixel-antialiased font-normal tracking-tight font-source-code-pro"
              >
                Settings
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <svg class="mt-6" viewBox="0 0 100 1" xmlns="http://www.w3.org/2000/svg">
      <line x1="5" y1="0" x2="95" y2="0" stroke="#313639" />
    </svg>

    <div class="flex items-center w-full h-12 justify-left color-grey-medium">
      <div class="flex self-center ml-8">
        <RefreshCwIcon
          size="1.2x"
          class="text-center cursor-pointer color-grey-light"
          @click="clearLogout"
        />
      </div>

      <div class="flex justify-end w-full">
        <div class="flex self-center mr-8">
          <div class="self-center flex-1 text-center">
            <user-icon
              size="1.2x"
              class="text-center cursor-pointer color-grey-light"
              @click="onLogout"
            />
          </div>
        </div>
      </div>
    </div>
  </nav>
</template>

<script lang="ts">
import Vue from "vue";
import {
  MenuIcon,
  Share2Icon,
  CodeIcon,
  UserIcon,
  ActivityIcon,
  BoxIcon,
  SettingsIcon,
  LayersIcon,
  BookOpenIcon,
  GridIcon,
  RefreshCwIcon,
} from "vue-feather-icons";

import SysinitIcon from "@/components/icons/SysinitIcon.vue";

import { mapGetters } from "vuex";

interface IData {
  workspaces: string[];
  currentWorkspace: string;
}

export default Vue.extend({
  name: "WorkspaceNav",
  components: {
    MenuIcon,
    Share2Icon,
    CodeIcon,
    UserIcon,
    SysinitIcon,
    ActivityIcon,
    BoxIcon,
    SettingsIcon,
    LayersIcon,
    BookOpenIcon,
    GridIcon,
    RefreshCwIcon,
  },
  data(): IData {
    return {
      workspaces: ["my workspace", "another workspace"],
      currentWorkspace: "my workspace",
    };
  },
  computed: {
    ...mapGetters({
      organization: "organization/current",
      workspace: "workspace/current",
    }),
  },
  methods: {
    async onLogout(): Promise<void> {
      await this.$store.dispatch("user/logout");
      this.$router.push({ name: "signin" });
    },
    async clearLogout(): Promise<void> {
      await this.$store.dispatch("user/logout");
      this.$router.push({ name: "signin" });
    },
  },
});
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

.container-link {
  @apply flex justify-center w-full h-10 mt-3;
}
</style>
