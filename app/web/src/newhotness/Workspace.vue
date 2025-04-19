<template>
  <div id="app-layout" class="h-screen flex flex-col">
    <!-- nav itself is fixed at 60 px-->
    <nav
      :class="
        clsx(
          'navbar bg-neutral-900 text-white relative shadow-[0_4px_4px_0_rgba(0,0,0,0.15)]',
          'z-90 h-[60px] overflow-hidden shrink-0 flex flex-row justify-between select-none',
          windowWidth > 740 && 'gap-sm',
        )
      "
    >
      <!-- Left side -->
      <NavbarPanelLeft />

      <!-- Center -->
      <div
        class="flex flex-row flex-none items-center h-full justify-center place-items-center mx-auto overflow-hidden"
      >
        <NavbarButton
          tooltipText="Compose"
          icon="diagram"
          :selected="route.name?.toString().startsWith('new-hotness')"
          :linkTo="compositionLink"
        />

        <NavbarButton
          tooltipText="Customize"
          icon="beaker"
          :selected="route.matched.some((r) => r.name === 'workspace-lab')"
          :linkTo="{
            path: `/w/${workspacePk}/:${changeSetId}/l`,
          }"
        />

        <NavbarButton
          tooltipText="Audit"
          icon="eye"
          :selected="route.matched.some((r) => r.name === 'workspace-audit')"
          :linkTo="{
            path: `/w/${workspacePk}/:${changeSetId}/a`,
          }"
        />
      </div>

      <!-- Right -->
      <NavbarPanelRight />
    </nav>

    <!-- grow the main body to fit all the space in between the nav and footer
     by default flexbox containers grow at minimum to the size of their content (e.g. autp)
     min-h-0 prevents the main container from being *larger* than the max it can grow, no matter its contents
     also, i chose 12 here and not a standard unit because 12 puts the left border of contents exactly on the SI logo in the top left -->
    <main class="grow p-[12px] min-h-0">
      <!-- more v-ifs for "am i looking at viewId? secretId? or the list of views or list of secrets?"-->
      <template v-if="componentId">
        <ComponentDetail :componentId="componentId" />
      </template>
      <template v-else>
        <Explore />
      </template>
    </main>

    <!-- footer is fixed at "10", 2.5rem, 40px-->
    <div
      :class="
        clsx(
          'flex justify-end items-center',
          'bg-neutral-900 text-white relative border-t flex-shrink-0 border-shade-100 shadow-[0_4px_4px_0_rgba(0,0,0,0.15)] z-90 h-10',
          'select-none',
        )
      "
    >
      <div class="text-sm pl-xs mr-lg w-full">
        System&nbsp;Initiative
        <RealtimeStatusPageState />

        <!-- im not really gonna float it, but im tired, FIXME -->
        <Breadcrumbs style="float: right" />
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { useRoute, useRouter } from "vue-router";
import clsx from "clsx";
import {
  computed,
  onBeforeUnmount,
  onMounted,
  ref,
  provide,
  toRef,
  watch,
} from "vue";
import { useQueryClient } from "@tanstack/vue-query";
import NavbarPanelLeft from "@/components/layout/navbar/NavbarPanelLeft.vue";
import NavbarPanelRight from "@/components/layout/navbar/NavbarPanelRight.vue";
import NavbarButton from "@/components/layout/navbar/NavbarButton.vue";
import RealtimeStatusPageState from "@/components/RealtimeStatusPageState.vue";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import * as heimdall from "@/store/realtime/heimdall";
import { useAuthStore } from "@/store/auth.store";
import Explore from "./Explore.vue";
import ComponentDetail from "./Component.vue";
import { WSCS } from "./types";
import Breadcrumbs from "./layout_components/Breadcrumbs.vue";
import { startKeyEmitter } from "./logic_composables/key_emitter";

const props = defineProps<{
  workspacePk: string;
  changeSetId: string;
  componentId?: string;
  viewId?: string;
  secretId?: string;
}>();

const authStore = useAuthStore();
const featureFlagsStore = useFeatureFlagsStore();

const wscs: WSCS = {
  workspacePk: toRef(props.workspacePk),
  changeSetId: toRef(props.changeSetId),
};

startKeyEmitter(document);

provide("WSCS", wscs);

const compositionLink = computed(() => {
  // eslint-disable-next-line no-nested-ternary
  const name = props.componentId
    ? "new-hotness-component"
    : props.viewId
    ? "new-hotness-view"
    : "new-hotness";
  return {
    name,
    params: props,
  };
});

const route = useRoute();
const router = useRouter();

const queryClient = useQueryClient();

onMounted(async () => {
  // NOTE(nick,wendy): if you do not have the flag enabled, you will be re-directed. This will be
  // true for all of the new hotness routes, provided that they are all children of the parent
  // route that uses this component. This is wrapped in a "setTimeout" to ensure that the feature
  // flag loads in time.
  setTimeout(() => {
    if (!featureFlagsStore.NEW_HOTNESS) {
      router.push({ name: "workspace-index" });
    }
  }, 500);

  // Activate the norse stack, which is explicitly NOT flagged for the job-specific UI.
  if (!authStore.selectedOrDefaultAuthToken)
    throw new Error("no auth token selected");
  await heimdall.init(authStore.selectedOrDefaultAuthToken, queryClient);
  watch(
    connectionShouldBeEnabled,
    async () => {
      if (connectionShouldBeEnabled.value) {
        // NOTE(nick,wendy): this says "reconnect", but it must run once on startup.
        await heimdall.bifrostReconnect();
      } else {
        await heimdall.bifrostClose();
      }
    },
    { immediate: true },
  );
  heimdall.niflheim(props.workspacePk, props.changeSetId, true);
});

const connectionShouldBeEnabled = computed(() => {
  try {
    const authStore = useAuthStore();
    return (
      authStore.userIsLoggedInAndInitialized && authStore.selectedWorkspaceToken
    );
  } catch (_err) {
    return false;
  }
});

const windowWidth = ref(window.innerWidth);

const windowResizeHandler = () => {
  windowWidth.value = window.innerWidth;
};

onMounted(() => {
  windowResizeHandler();
  window.addEventListener("resize", windowResizeHandler);
});
onBeforeUnmount(() => {
  window.removeEventListener("resize", windowResizeHandler);
});
</script>

<style lang="less">
* {
  box-sizing: border-box;
  /* other global styles */
}

/* use on any div in a grid that you want to have scrollable content */
.scrollable {
  overflow-y: auto;
  scrollbar-width: thin;
}
body.dark .scrollable {
  scrollbar-color: rgb(33, 33, 33) rgba(0, 0, 0, 0);
}
body.light .scrollable {
  scrollbar-color: rgb(212, 212, 212) rgb(255, 255, 255);
}

.grid > .scrollable {
  transition: max-height 1s;

  > .sticky {
    // TODO, find the right color
    // elevate this into `hasbg` in the HTML
    // and we use variables here to pick the right one
    background-color: rgb(66, 66, 66);
  }
}

/* any grid that has scrollable elements needs min-height 0 (just like main, above) otherwise the contents of the scrollable can blow out the container (putting it here globally means a human doesnt need to remember to do it every time) */
.grid:has(> .scrollable) {
  min-height: 0;
}

.tilegrid {
  display: grid;
  grid-gap: 1rem;

  > .tile {
    border: 1px solid white;
    &.pinned {
      grid-column: 1 / -1;
    }
  }
}

/* This rules determines
 * - the min card width
 * - fit as many on a row
 * - and since its a grid, every card will have the same height (as tall as the tallest card)
 */
@supports (width: min(250px, 100%)) {
  .tilegrid {
    grid-template-columns: repeat(auto-fit, minmax(min(250px, 100%), 1fr));
  }
}

// this is a LESS mixin, right now we've got 2 different logical elements with the same grid
// let's re-use the definition only once, until we don't want these elements to follow the same grid
.MixinGridIconsFlankingHeaderAndSubheader {
  display: grid;
  grid-row-gap: 2px;
  grid-column-gap: 0.5rem;
  // 32px is the icon size
  grid-template-columns: 32px 1fr 32px;
  grid-template-rows: 16px 16px;
  grid-template-areas:
    "logo h2 spinner"
    "logo h3 spinner";

  // the icons are divs
  > div:first-child {
    grid-area: "logo";
  }
  > div:last-child {
    grid-area: spinner;
  }
  > h2 {
    grid-area: h2;
    font-weight: bold;
  }
  > h3 {
    grid-area: h3;
    font-size: 0.8rem;
  }
}

.actions.list {
  .item {
    // actions list items also follow the grid
    .MixinGridIconsFlankingHeaderAndSubheader();
  }
}

// component cards look like this, everywhere in the app
.tile.component {
  display: flex;
  flex-direction: column;

  > * {
    // only direct children
    flex-grow: 1;
    padding: 0.5rem;
  }

  > header {
    // this header follows the grid from the mixin
    .MixinGridIconsFlankingHeaderAndSubheader();
  }

  > footer {
    // always place a gap between buttons
    button + button {
      margin-left: 0.5rem;
    }
  }
}

// other cards may look differently, can be defined globally, or piecemeal
</style>
