<template>
  <nav
    id="workspace-nav"
    class="flex flex-col h-full select-none vld-parent nav"
  >
    <div class="flex flex-col w-full h-full">
      <div id="brand-content" class="flex w-full h-12 nav-header">
        <button
          class="flex items-center justify-center w-full my-3 hover:none focus:outline-none"
          @click="isMaximized = !isMaximized"
        >
          <SysinitIcon v-show="isBrandLogoVisible" :size="1.15" class="" />
          <div v-show="isBrandTitleVisible" class="brand-title">
            System Init
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
        <VueFeather type="menu" class="color-grey-medium" />
        <div
          v-if="currentWorkspace"
          v-show="isLinkTitleVisible"
          class="ml-4 text-xs subpixel-antialiased font-normal color-grey-medium"
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
            <VueFeather type="activity" size="1.1rem" class="" />
            <div v-show="isLinkTitleVisible" class="link-title">Dashboard</div>
          </div>

          <!-- Applications Link -->
          <div class="container-link">
            <router-link
              v-if="currentOrganization && currentWorkspace"
              data-cy="application-nav-link"
              :to="{
                name: 'application',
                params: {
                  organizationId: currentOrganization.id,
                  workspaceId: currentWorkspace.id,
                },
              }"
            >
              <div class="flex items-center justify-start cursor-pointer">
                <VueFeather type="code" size="1.1rem" class="" />
                <div v-show="isLinkTitleVisible" class="link-title">
                  Applications
                </div>
              </div>
            </router-link>
          </div>

          <!-- Systems Link -->
          <div class="container-link">
            <div class="flex items-center justify-start">
              <VueFeather type="share-2" size="1.1rem" class="transform rotate-90" />
              <div v-show="isLinkTitleVisible" class="link-title">Systems</div>
            </div>
          </div>

          <!-- Components Link -->
          <div class="container-link">
            <div class="flex items-center justify-start">
              <VueFeather type="box" size="1.1rem" class="" />
              <div v-show="isLinkTitleVisible" class="link-title">
                Components
              </div>
            </div>
          </div>

          <!-- Resources Link -->
          <div class="container-link">
            <div class="flex items-center justify-start">
              <VueFeather type="grid" size="1.1rem" class="" />
              <div v-show="isLinkTitleVisible" class="link-title">
                Resources
              </div>
            </div>
          </div>

          <!-- Environment Link  AKA computing environment -->
          <div class="container-link">
            <div class="flex items-center justify-start">
              <VueFeather type="layers" size="1.1rem" class="" />
              <div v-show="isLinkTitleVisible" class="link-title">
                Environment
              </div>
            </div>
          </div>

          <!-- Catalogue Link -->
          <div class="container-link">
            <div class="flex items-center justify-start">
              <VueFeather type="book-open" size="1.1rem" class="" />
              <div v-show="isLinkTitleVisible" class="link-title">
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
              v-if="currentOrganization && currentWorkspace"
              data-cy="secret-nav-link"
              :to="{
                name: 'secret',
                params: {
                  organizationId: currentOrganization.id,
                  workspaceId: currentWorkspace.id,
                },
              }"
            >
              <div class="flex items-center justify-start cursor-pointer">
                <VueFeather type="key" size="1.1rem" class="" />
                <div v-show="isLinkTitleVisible" class="link-title">
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
              <VueFeather type="hexagon" size="1.1rem" class="" />
              <div v-show="isLinkTitleVisible" class="link-title">Clients</div>
            </div>
            <!-- </router-link> -->
          </div>

          <!-- Schema Link -->
          <div class="container-link">
            <router-link
              class="w-9/12"
              data-test="schema-nav-link"
              :to="{ name: 'schema' }"
            >
              <div class="flex items-center justify-start cursor-pointer">
                <VueFeather type="moon" size="1rem" class="" />
                <div v-show="isLinkTitleVisible" class="link-title">Schema</div>
              </div>
            </router-link>
          </div>

        </div>

        <div class="flex flex-col justify-end flex-grow">
          <!-- Settings Link -->
          <div class="container-link">
            <div
              class="flex items-center justify-start cursor-pointer focus:text-white"
            >
              <VueFeather type="settings" size="1.1rem" class="" />
              <div v-show="isLinkTitleVisible" class="link-title">Settings</div>
            </div>
          </div>
        </div>
      </div>

      <div class="self-center mt-3" :class="separatorClasses">
        <div class="menu-separator" />
      </div>

      <div class="flex items-center w-full mx-6 my-4 color-grey-medium">
        <button data-test="logout" aria-label="Logout" @click="onLogout">
          <VueFeather type="log-out"
                      size="1.1rem"
                      class="text-center cursor-pointer logout-button"
          />
        </button>
      </div>
    </div>
  </nav>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { refFrom } from "vuse-rx";
import VueFeather from "vue-feather";
import SysinitIcon from "@/atoms/SysinitIcon.vue";
import { SessionService } from "@/api/sdf/service/session";
import { useRouter } from "vue-router";
import { workspace$ } from "@/observable/workspace";
import { organization$ } from "@/observable/organization";

const isMaximized = ref(false);
const currentWorkspace = refFrom(workspace$);
const currentOrganization = refFrom(organization$);

const isLinkTitleVisible = computed(() => isMaximized.value);
const isBrandLogoVisible = computed(() => !isMaximized.value);
const isBrandTitleVisible = computed(() => isMaximized.value);
const separatorClasses = computed(() => {
  const classes: Record<string, true> = {};
  if (!isMaximized.value) {
    classes["w-10/12"] = true;
  } else {
    classes["w-11/12"] = true;
  }
  return classes;
});

const router = useRouter();
const onLogout = async () => {
  await SessionService.logout();
  await router.push({ name: "login" });
};
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
