<template>
  <AppLayout>
    <div
      data-testid="home"
      :class="
        clsx(
          'absolute w-screen h-screen flex flex-col items-center justify-center',
          themeClasses('bg-neutral-100', 'bg-neutral-900'),
        )
      "
    >
      <!-- Floating panel (holds box shadow)  -->
      <div
        :class="
          clsx(
            'w-[720px] max-w-[80vw] min-h-[min(400px, 100vh)] max-h-[80vh] rounded',
            'shadow-[0_0_8px_0_rgba(255,255,255,0.08)]',
            'transition-opacity min-h-0 ',
            themeClasses('bg-white border', 'bg-neutral-800 text-white'),
            show ? 'opacity-100' : 'opacity-0',
          )
        "
      >
        <template v-if="workspacesReqStatus.isSuccess">
          <h1
            :class="
              clsx(
                'p-sm text-lg font-bold h-[60px] border-b flex flex-row items-center',
                themeClasses('border-neutral-300', 'border-neutral-600'),
              )
            "
          >
            <SiLogo
              class="block h-[30px] w-[30px] ml-[12px] mr-[12px] flex-none"
            />
            CHOOSE A WORKSPACE
          </h1>
          <ul class="h-[calc(100%-60px)] min-h-0 scrollable">
            <li
              v-for="(workspace, idx) in workspacesStore.allWorkspaces"
              :key="workspace.id"
              :class="
                clsx(
                  'cursor-pointer',
                  idx % 2 === 1
                    ? themeClasses('bg-neutral-200', 'bg-neutral-800')
                    : themeClasses('bg-neutral-100', 'bg-neutral-700'),
                  themeClasses('hover:bg-neutral-300', 'hover:bg-neutral-600'),
                )
              "
              @click="() => goto(workspace)"
            >
              <TruncateWithTooltip class="p-sm pl-md">
                <a :href="`${workspace.instanceUrl}/w/${workspace.id}`">
                  {{ workspace.displayName }}
                </a>
              </TruncateWithTooltip>
            </li>
          </ul>
        </template>
        <p v-else>Error loading workspaces, please try again.</p>
      </div>

      <NewButton
        v-if="featureFlagsStore.ADMIN_PANEL_ACCESS"
        class="absolute bottom-sm right-sm"
        :icon="theme === 'dark' ? 'moon' : 'sun'"
        @click="toggleTheme"
      />
    </div>
  </AppLayout>
</template>

<script setup lang="ts">
import { computed, onBeforeMount, ref } from "vue";
import { useRouter, useRoute } from "vue-router";
import clsx from "clsx";
import SiLogo from "@si/vue-lib/brand-assets/si-logo-symbol.svg?component";
import {
  NewButton,
  TruncateWithTooltip,
  themeClasses,
  useTheme,
  userOverrideTheme,
} from "@si/vue-lib/design-system";
import { useWorkspacesStore } from "@/store/workspaces.store";
import AppLayout from "@/components/layout/AppLayout.vue";
import { AuthApiWorkspace } from "@/newhotness/types";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";

const router = useRouter();
const route = useRoute();

const featureFlagsStore = useFeatureFlagsStore();
const { theme } = useTheme();

function toggleTheme() {
  userOverrideTheme.value = theme.value === "dark" ? "light" : "dark";
}

// assume we're redirecting initially
const redirecting = ref(true);

const workspacesStore = useWorkspacesStore();

const workspacesReqStatus = workspacesStore.getRequestStatus(
  "FETCH_USER_WORKSPACES",
);

const show = computed(
  () => workspacesReqStatus.value.completed && !redirecting.value,
);

async function autoSelectWorkspace() {
  const redirectPath = route.query.redirect as string;
  if (redirectPath) {
    const redirectObject = { path: redirectPath };
    await router.replace(redirectObject);
  }
  redirecting.value = false;
}

const goto = (workspace: AuthApiWorkspace) => {
  window.location.href = `${workspace.instanceUrl}/w/${workspace.id}`;
};

onBeforeMount(() => {
  autoSelectWorkspace();
});
</script>

<style lang="css">
.scrollable {
  overflow-y: auto;
  scrollbar-width: thin;
}
.scrollable-horizontal {
  overflow-x: auto;
  scrollbar-width: thin;
}
body.dark .scrollable,
body.dark .scrollable-horizontal {
  scrollbar-color: @colors-neutral-800 @colors-black;
}
</style>
