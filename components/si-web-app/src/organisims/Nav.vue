<template>
  <nav
    id="workspace-nav"
    class="flex flex-col h-full select-none vld-parent nav"
  >
    <div class="flex flex-col w-full h-full">
      <DebugRoute testId="location-display-nav" />
      <SiLoader :isLoading="isLoading" />

      <div id="brand-content" class="flex w-full h-12 nav-header">
        <button
          class="flex items-center justify-center w-full my-3 hover:none focus:outline-none"
          v-on:click="isMaximized = !isMaximized"
        >
          <SysinitIcon :size="1.15" class="" v-show="isBrandLogoVisible" />
          <div class="brand-title" v-show="isBrandTitleVisible">
            System Initiative
          </div>
        </button>
      </div>

      <div class="self-center" :class="separatorClasses">
        <div class="menu-separator" />
      </div>

      <div
        id="workspace-selector"
        class="flex items-center w-full h-4 mt-3 ml-6 justify-left"
      >
        <menu-icon size="1x" class="color-grey-medium" />
        <div
          class="ml-4 text-xs subpixel-antialiased font-normal color-grey-medium"
          v-if="currentWorkspace"
          v-show="isLinkTitleVisible"
        >
          {{ currentWorkspace.name }}
        </div>
      </div>

      <div class="self-center mt-3" :class="separatorClasses">
        <div class="menu-separator" />
      </div>

      <div id="workspace-content" class="flex flex-col flex-grow mx-6 mt-4">
        <div class="flex flex-col">
          <!-- Dashboard Link -->
          <div class="container-link">
            <activity-icon size="1.1x" class="" />
            <div class="link-title" v-show="isLinkTitleVisible">
              Dashboard
            </div>
          </div>

          <!-- Applications Link -->
          <div class="container-link">
            <router-link
              data-cy="application-nav-link"
              :to="{
                name: 'application',
                params: {
                  organizationId: currentOrganization.id,
                  workspaceId: currentWorkspace.id,
                },
              }"
              v-if="currentOrganization && currentWorkspace"
            >
              <div class="flex items-center justify-start cursor-pointer">
                <code-icon size="1.1x" class="" />
                <div class="link-title" v-show="isLinkTitleVisible">
                  Applications
                </div>
              </div>
            </router-link>
          </div>

          <!-- Systems Link -->
          <div class="container-link">
            <div class="flex items-center justify-start">
              <share-2-icon size="1.1x" class="transform rotate-90" />
              <div class="link-title" v-show="isLinkTitleVisible">
                Systems
              </div>
            </div>
          </div>

          <!-- Components Link -->
          <div class="container-link">
            <div class="flex items-center justify-start">
              <box-icon size="1.1x" class="" />
              <div class="link-title" v-show="isLinkTitleVisible">
                Components
              </div>
            </div>
          </div>

          <!-- Resources Link -->
          <div class="container-link">
            <div class="flex items-center justify-start">
              <grid-icon size="1.1x" class="" />
              <div class="link-title" v-show="isLinkTitleVisible">
                Resources
              </div>
            </div>
          </div>

          <!-- Environment Link  AKA computing environment -->
          <div class="container-link">
            <div class="flex items-center justify-start">
              <layers-icon size="1.1x" class="" />
              <div class="link-title" v-show="isLinkTitleVisible">
                Environment
              </div>
            </div>
          </div>

          <!-- Catalogue Link -->
          <div class="container-link">
            <div class="flex items-center justify-start">
              <book-open-icon size="1.1x" class="" />
              <div class="link-title" v-show="isLinkTitleVisible">
                Catalogue
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
            <router-link
              data-cy="secret-nav-link"
              :to="{
                name: 'secret',
                params: {
                  organizationId: currentOrganization.id,
                  workspaceId: currentWorkspace.id,
                },
              }"
              v-if="currentOrganization && currentWorkspace"
            >
              <div class="flex items-center justify-start cursor-pointer">
                <key-icon size="1.1x" class="" />
                <div class="link-title" v-show="isLinkTitleVisible">
                  Secrets
                </div>
              </div>
            </router-link>
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
            <div class="flex items-center justify-start cursor-pointer">
              <hexagon-icon size="1.1x" class="" />
              <div class="link-title" v-show="isLinkTitleVisible">
                Clients
              </div>
            </div>
            <!-- </router-link> -->
          </div>
        </div>

        <div class="flex flex-col justify-end flex-grow">
          <!-- Settings Link -->
          <div class="container-link">
            <div
              class="flex items-center justify-start cursor-pointer focus:text-white"
            >
              <settings-icon size="1.1x" class="" />
              <div class="link-title" v-show="isLinkTitleVisible">
                Settings
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="self-center mt-3" :class="separatorClasses">
        <div class="menu-separator" />
      </div>

      <div class="flex items-center w-full mx-6 my-4 color-grey-medium">
        <button @click="onLogout" aria-label="Logout">
          <LogOutIcon
            size="1.1x"
            class="text-center cursor-pointer logout-button"
          />
        </button>
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
  LogOutIcon,
  KeyIcon,
  HexagonIcon,
} from "vue-feather-icons";
import SysinitIcon from "@/atoms/SysinitIcon.vue";
import SiLoader from "@/atoms/SiLoader.vue";
import DebugRoute from "@/atoms/DebugRoute.vue";

import { mapState } from "vuex";
import { SiVuexStore } from "@/store";
import { Workspace } from "@/api/sdf/model/workspace";
import { Organization } from "@/api/sdf/model/organization";

interface IData {
  isMaximized: boolean;
}

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
    LogOutIcon,
    SiLoader,
    DebugRoute,
    KeyIcon,
    HexagonIcon,
  },
  data(): IData {
    return {
      isMaximized: false,
    };
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
    isLinkTitleVisible(): boolean {
      return this.isMaximized;
    },
    isBrandLogoVisible(): boolean {
      return !this.isMaximized;
    },
    isBrandTitleVisible(): boolean {
      return this.isMaximized;
    },
    separatorClasses(): Record<string, any> {
      let classes: Record<string, any> = {};
      if (!this.isMaximized) {
        classes["w-10/12"] = true;
      } else {
        classes["w-11/12"] = true;
      }
      return classes;
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
.nav {
  border-right: 1px solid #2a2a2a;
}

.menu-separator {
  background-color: #313639;
  height: 1px;
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
  @apply flex;
  @apply justify-start;
  @apply items-center;
  @apply w-full;
  @apply h-10;
  @apply mt-2;
  color: #4a4b4c;
}

.container-link:hover {
  @apply text-gray-400;
}

.brand-title {
  font-family: "Source Code Pro";
  @apply text-sm;
  @apply font-medium;
  @apply antialiased;
  @apply tracking-tighter;
}

.link-title {
  @apply text-sm;
  @apply subpixel-antialiased;
  @apply font-normal;
  @apply tracking-tight;
  @apply ml-3;
}

.router-link-active {
  color: #c7cacd;
}

.logout-button {
  color: #4a4b4c;
}

.logout-button:hover {
  @apply text-gray-400;
}
</style>
