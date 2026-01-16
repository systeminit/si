<template>
  <AppLayout>
    <div
      data-testid="home"
      :class="
        clsx(
          'absolute w-screen h-screen flex flex-col items-center justify-start pt-[10vh]',
          themeClasses('bg-neutral-100', 'bg-neutral-900'),
        )
      "
    >
      <!-- Floating panel (holds box shadow)  -->
      <div
        :class="
          clsx(
            'w-[720px] max-w-[80vw] min-h-[min(300px, 90vh)] max-h-[80vh] rounded',
            'shadow-[0_0_8px_0_rgba(255,255,255,0.08)]',
            'transition-opacity min-h-0 ',
            themeClasses('bg-white border', 'bg-neutral-800 text-white'),
            show ? 'opacity-100' : 'opacity-0',
            'shrink-0 flex flex-col items-stretch',
          )
        "
      >
        <template v-if="workspacesReqStatus.isSuccess">
          <h1
            :class="
              clsx(
                'shrink basis-0',
                'p-sm text-lg font-bold h-[60px] flex flex-row items-center',
                themeClasses('border-neutral-300', 'border-neutral-600'),
              )
            "
          >
            <SiLogo class="block h-[30px] w-[30px] ml-[12px] mr-[12px] flex-none" />
            CHOOSE A WORKSPACE
          </h1>
          <div class="h-[48px] shrink basis-0">
            <InstructiveVormInput
              :class="clsx('cursor-text')"
              :activeClasses="themeClasses('border-action-500', 'border-action-300')"
              :inactiveClasses="
                themeClasses('border-neutral-400 hover:border-black', 'border-neutral-600 hover:border-white')
              "
              @click="searchRef?.focus()"
            >
              <template #left>
                <Icon name="search" tone="neutral" size="sm" />
              </template>
              <template #default="slotProps">
                <VormInput
                  ref="searchRef"
                  v-model="searchString"
                  autocomplete="off"
                  :class="slotProps.class"
                  noStyles
                  placeholder="Filter workspace list"
                  @focus="
                    () => {
                      slotProps.focus();
                    }
                  "
                  @blur="
                    () => {
                      slotProps.blur();
                    }
                  "
                  @keydown.down.prevent="down"
                  @keydown.up.prevent="up"
                  @keydown.enter.prevent="enter"
                />
              </template>
            </InstructiveVormInput>
          </div>
          <ul class="grow basis-4/5 min-h-0 scrollable">
            <li
              v-for="(workspace, idx) in filteredWorkspaces"
              :key="workspace.id"
              :class="
                clsx(
                  'cursor-pointer',
                  idx % 2 === 1
                    ? themeClasses('bg-neutral-200', 'bg-neutral-800')
                    : themeClasses('bg-neutral-100', 'bg-neutral-700'),
                  themeClasses('hover:bg-neutral-300', 'hover:bg-neutral-600'),
                  workspace.id === selectedWorkspaceId && [
                    themeClasses('bg-neutral-300', 'bg-neutral-600'),
                    'workspace-selected-item',
                  ],
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
        <p v-else-if="workspacesReqStatus.isError">Error loading workspaces, please try again.</p>
        <DelayedLoader v-else-if="workspacesReqStatus.isPending" :size="'full'" />
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
import { computed, nextTick, onBeforeMount, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useRouter, useRoute } from "vue-router";
import clsx from "clsx";
import SiLogo from "@si/vue-lib/brand-assets/si-logo-symbol.svg?component";
import {
  NewButton,
  TruncateWithTooltip,
  themeClasses,
  useTheme,
  VormInput,
  Icon,
  userOverrideTheme,
} from "@si/vue-lib/design-system";
import { Fzf } from "fzf";
import { useWorkspacesStore } from "@/store/workspaces.store";
import AppLayout from "@/components/layout/AppLayout.vue";
import { AuthApiWorkspace } from "@/newhotness/types";
import InstructiveVormInput from "@/newhotness/layout_components/InstructiveVormInput.vue";
import DelayedLoader from "@/newhotness/layout_components/DelayedLoader.vue";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { keyEmitter, startKeyEmitter } from "@/newhotness/logic_composables/emitters";

const router = useRouter();
const route = useRoute();

const searchString = ref<string | null>("");
const searchRef = ref<InstanceType<typeof VormInput>>();
const selectedWorkspaceId = ref("");

const filteredWorkspaces = computed(() => {
  if (!searchString.value) return workspacesStore.allWorkspaces;
  else {
    const fzf = new Fzf(workspacesStore.allWorkspaces, {
      casing: "case-insensitive",
      selector: (c) => `${c.displayName} ${c.id}`,
    });
    return fzf.find(searchString.value).map((fz) => fz.item);
  }
});

const pickLast = () => {
  selectedWorkspaceId.value = filteredWorkspaces.value[filteredWorkspaces.value.length - 1]?.id || "";
  scrollToSelected();
};
const pickFirst = () => {
  selectedWorkspaceId.value = filteredWorkspaces.value[0]?.id || "";
  scrollToSelected();
};

const down = () => {
  let next = false;
  if (!selectedWorkspaceId.value) {
    pickFirst();
    return;
  }
  for (const w of filteredWorkspaces.value) {
    if (next) {
      selectedWorkspaceId.value = w.id;
      scrollToSelected();
      return;
    }
    if (w.id === selectedWorkspaceId.value) next = true;
  }
  if (next) pickFirst();
};

const up = () => {
  let last = "";
  if (!selectedWorkspaceId.value) {
    pickLast();
    return;
  }
  if (filteredWorkspaces.value[0]?.id === selectedWorkspaceId.value) {
    pickLast();
    return;
  }
  for (const w of filteredWorkspaces.value) {
    if (w.id === selectedWorkspaceId.value) {
      selectedWorkspaceId.value = last;
      scrollToSelected();
      return;
    }
    last = w.id;
  }
};

const enter = () => {
  if (!selectedWorkspaceId.value) return;
  const w = filteredWorkspaces.value.find((w) => w.id === selectedWorkspaceId.value);
  if (!w) return;
  goto(w);
};

const scrollToSelected = async () => {
  // First, wait one tick for the dom classes to update
  await nextTick();
  // Then, see if the element exists in the DOM
  const el = document.getElementsByClassName("workspace-selected-item")[0];
  if (el) {
    // If it does, scroll it to the center
    el.scrollIntoView({ block: "center" });
  }
};

startKeyEmitter(document);
const clearKeyEmitters = () => {
  keyEmitter.off("Enter");
  keyEmitter.off("ArrowUp");
  keyEmitter.off("ArrowDown");
};
onMounted(async () => {
  clearKeyEmitters();

  keyEmitter.on("Enter", () => {
    enter();
  });
  keyEmitter.on("ArrowUp", (e) => {
    e.preventDefault();
    up();
  });
  keyEmitter.on("ArrowDown", (e) => {
    e.preventDefault();
    down();
  });
});
onBeforeUnmount(() => {
  clearKeyEmitters();
});

const mountSearch = watch(searchRef, () => {
  if (searchRef.value) {
    searchRef.value?.focus();
    mountSearch();
  }
});

const featureFlagsStore = useFeatureFlagsStore();
const { theme } = useTheme();

function toggleTheme() {
  userOverrideTheme.value = theme.value === "dark" ? "light" : "dark";
}

// assume we're redirecting initially
const redirecting = ref(true);

const workspacesStore = useWorkspacesStore();

const workspacesReqStatus = workspacesStore.getRequestStatus("FETCH_USER_WORKSPACES");

const show = computed(() => workspacesReqStatus.value.completed && !redirecting.value);

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
