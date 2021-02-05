<template>
  <nav id="workspace-nav" class="flex flex-col w-full h-full vld-parent">
    <DebugRoute testId="location-display-nav" />
    <SiLoader :isLoading="isLoading" />
    <div class="flex justify-center w-full h-13 nav-header">
      <div class="flex items-center justify-between w-10/12">
        <SysinitIcon :size="1.4"> </SysinitIcon>
        <div class="text-xl font-medium text-white">System Init</div>
      </div>
    </div>

    <div class="flex justify-center w-full h-10 mt-6">
      <div class="flex items-center">
        <div
          class="text-base font-normal color-grey-medium"
          v-if="currentWorkspace"
        >
          {{ currentWorkspace.name }} Workspace
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
        <div class="container-link">
          <div class="w-9/12">
            <div class="flex items-center justify-start">
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
          <!-- <router-link
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
            > -->
          <div class="w-9/12">
            <div class="flex items-center justify-start cursor-pointer">
              <code-icon size="1.1x" class="mr-3" />
              <div
                class="text-sm subpixel-antialiased font-normal tracking-tight font-source-code-pro"
              >
                Applications
              </div>
            </div>
          </div>
          <!-- </router-link> -->
        </div>

        <!-- Systems Link -->
        <div class="container-link">
          <div class="w-9/12">
            <div class="flex items-center justify-start">
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
          <div class="w-9/12">
            <div class="flex items-center justify-start">
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
          <div class="w-9/12">
            <div class="flex items-center justify-start">
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
          <div class="w-9/12">
            <div class="flex items-center justify-start">
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
          <div class="w-9/12">
            <div class="flex items-center justify-start">
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

      <!-- Secrets Link -->
      <div class="container-link">
        <!-- <router-link
          class="w-9/12"
          data-cy="secret-nav-link"
          :to="{
            name: 'secret',
            params: {
              organizationId: organization.id,
              workspaceId: workspace.id,
            },
          }"
          > -->
        <div class="w-9/12">
          <div class="flex items-center justify-start cursor-pointer">
            <file-icon size="1.1x" class="mr-3" />
            <div
              class="text-sm subpixel-antialiased font-normal tracking-tight font-source-code-pro"
            >
              Secrets
            </div>
          </div>
        </div>
        <!-- </router-link> -->
      </div>

      <!-- Clients Link -->
      <div class="container-link">
        <!-- <router-link
          class="w-9/12"
          data-cy="client-nav-link"
          :to="{
            name: 'client',
            params: {
              organizationId: organization.id,
              workspaceId: workspace.id,
            },
          }"
          > -->
        <div class="w-9/12">
          <div class="flex items-center justify-start cursor-pointer">
            <file-icon size="1.1x" class="mr-3" />
            <div
              class="text-sm subpixel-antialiased font-normal tracking-tight font-source-code-pro"
            >
              Clients
            </div>
          </div>
        </div>
        <!-- </router-link> -->
      </div>

      <div class="flex flex-col justify-end flex-grow">
        <!-- Settings Link -->
        <div class="container-link">
          <div class="w-9/12">
            <div
              class="flex items-center justify-start cursor-pointer color-disabled focus:text-white"
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
      <div class="flex justify-end w-full">
        <div class="flex self-center mr-8">
          <div class="self-center flex-1 text-center">
            <button @click="onLogout" aria-label="Logout">
              <LogOutIcon
                size="1.2x"
                class="text-center cursor-pointer color-grey-light"
              />
            </button>
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
  ActivityIcon,
  BoxIcon,
  SettingsIcon,
  LayersIcon,
  BookOpenIcon,
  GridIcon,
  FileIcon,
  LogOutIcon,
} from "vue-feather-icons";
import SysinitIcon from "@/atoms/SysinitIcon.vue";
import SiLoader from "@/atoms/SiLoader.vue";
import DebugRoute from "@/atoms/DebugRoute.vue";

import { mapState } from "vuex";
import { SiVuexStore } from "@/store";
import { Workspace } from "@/api/sdf/model/workspace";
import { Organization } from "@/api/sdf/model/organization";

export default Vue.extend({
  name: "WorkspaceNav",
  components: {
    MenuIcon,
    Share2Icon,
    CodeIcon,
    SysinitIcon,
    ActivityIcon,
    BoxIcon,
    SettingsIcon,
    LayersIcon,
    BookOpenIcon,
    GridIcon,
    FileIcon,
    LogOutIcon,
    SiLoader,
    DebugRoute,
  },
  computed: {
    ...mapState({
      currentWorkspace(state: Record<string, any>): Workspace {
        return state["session"]["currentWorkspace"];
      },
      currentOrganization(state: Record<string, any>): Organization {
        return state["session"]["currentOrganization"];
      },
    }),
    isLoading(): boolean {
      if (this.currentWorkspace && this.currentOrganization) {
        return false;
      } else {
        return true;
      }
    },
  },
  methods: {
    async onLogout(): Promise<void> {
      await this.$store.dispatch("session/logout");
      this.$router.push({ name: "login" });
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
  color: #4a4b4c;
}

.router-link-active {
  color: #c7cacd;
}
</style>

