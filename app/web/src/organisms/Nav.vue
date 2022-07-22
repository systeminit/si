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
          <SysinitIcon v-show="isBrandLogoVisible" class="h-7" />
          <div v-show="isBrandTitleVisible" class="brand-title">
            System Init
          </div>
        </button>
      </div>

      <div class="self-center" :class="separatorClasses">
        <div class="menu-separator" />
      </div>

      <!--
      <div
        id="workspace-selector"
        class="flex items-center w-full h-4 mt-3 ml-6 justify-left"
      >
        <SiIcon tooltip-text="Menu">
          <MenuIcon class="color-grey-medium" />
        </SiIcon>
        <div
          v-if="currentWorkspace"
          v-show="isLinkTitleVisible"
          class="ml-4 text-xs subpixel-antialiased font-normal color-grey-medium"
        >
          {{ currentWorkspace.name }}
        </div>
      </div>
-->

      <div id="workspace-content" class="flex flex-col flex-grow mx-6">
        <div class="flex flex-col">
          <!-- Dashboard Link -->
          <!--
          <div class="container-link">
            <SiIcon tooltip-text="Dashboard">
              <LightningBoltIcon class="color-grey-medium" />
            </SiIcon>
            <div v-show="isLinkTitleVisible" class="link-title">Dashboard</div>
          </div>
          -->

          <!-- Applications Link -->
          <div class="container-link">
            <router-link :to="{ name: 'application-list' }">
              <div class="flex items-center justify-start cursor-pointer">
                <SiIcon tooltip-text="Applications">
                  <CodeIcon class="color-grey-medium" />
                </SiIcon>
                <div v-show="isLinkTitleVisible" class="link-title">
                  Applications
                </div>
              </div>
            </router-link>
          </div>

          <!-- Systems Link -->
          <!--
          <div class="container-link">
            <div class="flex items-center justify-start">
              <SiIcon tooltip-text="Systems">
                <ShareIcon class="color-grey-medium transform rotate-90" />
              </SiIcon>
              <div v-show="isLinkTitleVisible" class="link-title">Systems</div>
            </div>
          </div>
          -->

          <!-- Components Link -->
          <!--
          <div class="container-link">
            <div class="flex items-center justify-start">
              <SiIcon tooltip-text="Components">
                <CubeIcon class="color-grey-medium" />
              </SiIcon>
              <div v-show="isLinkTitleVisible" class="link-title">
                Components
              </div>
            </div>
          </div>
          -->

          <!-- Resources Link -->
          <!--
          <div class="container-link">
            <div class="flex items-center justify-start">
              <SiIcon tooltip-text="Resources">
                <ViewGridIcon class="color-grey-medium" />
              </SiIcon>
              <div v-show="isLinkTitleVisible" class="link-title">
                Resources
              </div>
            </div>
          </div>
          -->

          <!-- Environment Link  AKA computing environment -->
          <!--
          <div class="container-link">
            <div class="flex items-center justify-start">
              <SiIcon tooltip-text="Environment">
                <CollectionIcon class="color-grey-medium" />
              </SiIcon>
              <div v-show="isLinkTitleVisible" class="link-title">
                Environment
              </div>
            </div>
          </div>
          -->

          <!-- Catalogue Link -->
          <!--
          <div class="container-link">
            <div class="flex items-center justify-start">
              <SiIcon tooltip-text="Catalogue">
                <BookOpenIcon class="color-grey-medium" />
              </SiIcon>
              <div v-show="isLinkTitleVisible" class="link-title">
                Catalogue
              </div>
            </div>
          </div>
          -->

          <!-- Secrets Link -->
          <!--
          <div class="container-link">
            <router-link
              v-if="currentOrganization && currentWorkspace"
              data-cy="secret-nav-link"
              to="notFound"
            >
              <div class="flex items-center justify-start cursor-pointer">
                <SiButtonIcon tooltip-text="Secrets">
                  <KeyIcon class="color-grey-medium" />
                </SiButtonIcon>
                <div v-show="isLinkTitleVisible" class="link-title">
                  Secrets
                </div>
              </div>
            </router-link>
          </div>
          -->
          <!-- Clients Link -->
          <!--
          <div class="container-link">
            <div class="flex items-center justify-start">
              <SiIcon tooltip-text="Clients">
                <GlobeAltIcon class="color-grey-medium" />
              </SiIcon>
              <div v-show="isLinkTitleVisible" class="link-title">Clients</div>
            </div>
          </div>
-->

          <!-- Schema Link -->
          <!--
          <div class="container-link">
            <router-link
              class="w-9/12"
              data-test="schema-nav-link"
              :to="{ name: 'schema' }"
            >
              <div class="flex items-center justify-start cursor-pointer">
                <SiButtonIcon tooltip-text="Schema">
                  <MoonIcon class="color-grey-medium" />
                </SiButtonIcon>
                <div v-show="isLinkTitleVisible" class="link-title">Schema</div>
              </div>
            </router-link>
          </div>
          -->
        </div>

        <!--
        <div class="flex flex-col justify-end flex-grow">
          <div class="container-link">
            <div class="flex items-center justify-start focus:text-white">
              <SiIcon tooltip-text="Settings">
                <CogIcon class="color-grey-medium" />
              </SiIcon>
              <div v-show="isLinkTitleVisible" class="link-title">Settings</div>
            </div>
          </div>
        </div>
          -->
      </div>

      <div v-if="workspaceId">
        <div class="self-center" :class="separatorClasses">
          <div class="menu-separator" />
        </div>

        <div class="flex items-center w-full mx-6 my-4 color-grey-light">
          <button data-test="new-home" @click="onWorkspace">
            <SiIcon tooltip-text="New Home">
              <SwitchHorizontalIcon class="color-grey-light" />
            </SiIcon>
          </button>
        </div>
      </div>

      <div class="self-center" :class="separatorClasses">
        <div class="menu-separator" />
      </div>

      <div class="flex items-center w-full mx-6 my-4 color-grey-light">
        <button data-test="logout" @click="onLogout">
          <SiIcon tooltip-text="Logout">
            <LogoutIcon class="color-grey-light" />
          </SiIcon>
        </button>
      </div>
    </div>
  </nav>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import SysinitIcon from "@/atoms/SysinitIcon.vue";
import { SessionService } from "@/service/session";
import { useRouter } from "vue-router";
import SiIcon from "@/atoms/SiIcon.vue";
import {
  CodeIcon,
  LogoutIcon,
  SwitchHorizontalIcon,
} from "@heroicons/vue/outline";
import { Workspace } from "@/api/sdf/dal/workspace";
import { WorkspaceService } from "@/service/workspace";
import { refFrom } from "vuse-rx/src";

const isMaximized = ref(false);

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
const onWorkspace = async () => {
  if (workspaceId.value) {
    await router.push({
      name: "workspace-single",
      path: "/new/w/:workspaceId",
      params: { workspaceId: workspaceId.value },
    });
  } else {
    // FIXME(nick): ensure that it's impossible for this to be executed
    // when "workspaceId" is undefined.
    console.log("workspace id is undefined");
  }
};

const workspace = refFrom<Workspace | null>(
  WorkspaceService.currentWorkspace(),
);
const workspaceId = computed((): number | undefined => {
  if (workspace.value) {
    return workspace.value.id;
  }
  return undefined;
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
