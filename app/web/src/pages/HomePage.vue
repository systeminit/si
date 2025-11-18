<template>
  <AppLayout>
    <div
      data-testid="home"
      class="absolute w-screen h-screen bg-neutral-900 flex flex-col items-center justify-center"
    >
      <!-- Floating panel (holds box shadow)  -->
      <div
        :class="
          clsx(
            'w-[720px] max-w-[70vw] h-[400px] rounded bg-neutral-800',
            'shadow-[0_0_8px_0_rgba(255,255,255,0.08)]',
            'transition-opacity min-h-0 text-white',
            show ? 'opacity-100' : 'opacity-0',
          )
        "
      >
        <template v-if="workspacesReqStatus.isSuccess">
          <h1
            class="p-sm text-lg font-bold h-[60px] border-b border-white flex flex-row items-center"
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
                  'p-sm pl-md hover:bg-neutral-600 cursor-pointer',
                  idx % 2 === 1 ? 'bg-neutral-800' : 'bg-neutral-700',
                )
              "
              @click="() => goto(workspace)"
            >
              <a :href="`${workspace.instanceUrl}/w/${workspace.id}`">{{
                workspace.displayName
              }}</a>
            </li>
          </ul>
        </template>
        <p v-else>Error loading workspaces, please try again.</p>
      </div>
    </div>
  </AppLayout>
</template>

<script setup lang="ts">
import { computed, onBeforeMount, ref } from "vue";
import { useRouter, useRoute } from "vue-router";
import clsx from "clsx";
import SiLogo from "@si/vue-lib/brand-assets/si-logo-symbol.svg?component";
import { useWorkspacesStore } from "@/store/workspaces.store";
import AppLayout from "@/components/layout/AppLayout.vue";
import { AuthApiWorkspace } from "@/newhotness/types";

const router = useRouter();
const route = useRoute();

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
