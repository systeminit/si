<template>
  <div>
    <template v-if="BROWSER_IS_MOBILE">
      <div class="fixed inset-0 flex items-center justify-center p-md">
        <div
          class="text-2xl text-center dark:text-neutral-300 text-neutral-700 font-bold"
        >
          You are accessing this page from a mobile device.
          <br /><br />System Initiative can only be used on a desktop computer.
        </div>
      </div>
    </template>
    <template v-else-if="route.name === 'print-legal'">
      <RouterView />
    </template>
    <template v-else>
      <header class="flex p-md items-center">
        <div class="mr-md shrink-0">
          <img :src="SiLogoUrl" class="w-[40px] h-[40px]" />
        </div>

        <template v-if="userIsLoggedIn">
          <Stack spacing="md">
            <!-- <p class="capsize text-lg font-bold">
              Hi {{ authStore.bestUserLabel }}!
            </p> -->

            <nav class="flex gap-md font-bold">
              <!-- <RouterLink :to="{ name: 'profile' }">Profile</RouterLink> -->
              <RouterLink :to="{ name: 'tutorial' }" class="underline-link"
                >Tutorial</RouterLink
              >
              <RouterLink :to="{ name: 'dashboard' }" class="underline-link"
                >Dashboard</RouterLink
              >
              <!-- <RouterLink :to="{ name: 'logout' }">Logout</RouterLink> -->
            </nav>
          </Stack>

          <!-- dark/light mode toggle, floating in bottom left -->
          <div class="fixed left-0 bottom-0 p-sm">
            <VButton2
              :icon="rootTheme === 'dark' ? 'moon' : 'sun'"
              tone="shade"
              variant="transparent"
              size="md"
              @click="toggleTheme"
            />
          </div>

          <a
            href="#"
            class="ml-auto flex items-center gap-sm children:pointer-events-none"
            @click.prevent="profileMenuRef?.open"
          >
            <div>Hi Theo!</div>
            <div class="hover:opacity-90 cursor-pointer flex">
              <img
                v-if="user?.pictureUrl"
                :src="user.pictureUrl"
                class="w-[30px] h-[30px] block rounded-full"
                referrerpolicy="no-referrer"
              />
              <Icon v-else name="user-circle" size="lg" />
            </div>
          </a>
          <!-- <VButton2
        tone="neutral"
        icon="x"
        variant="ghost"
        :href="`${API_URL}/auth/logout`"
        >Log out!</VButton2
      > -->
        </template>
      </header>
      <DropdownMenu ref="profileMenuRef" force-align-right>
        <DropdownMenuItem icon="user-circle" link-to-named-route="profile"
          >Profile</DropdownMenuItem
        >
        <DropdownMenuItem icon="logout" link-to-named-route="logout"
          >Log out</DropdownMenuItem
        >
      </DropdownMenu>

      <div class="">
        <template v-if="!checkAuthReq.isRequested || checkAuthReq.isPending">
          <Icon name="loader" size="xl" />
          checking if you are logged in...
        </template>
        <template v-else>
          <div class="m-auto max-w-[1200px]">
            <div
              class="m-lg mb-xl p-md dark:bg-neutral-800 bg-neutral-200 rounded-md"
            >
              <RouterView />
            </div>
          </div>
        </template>
      </div>
    </template>

    <div class="text-sm opacity-30 p-xs text-center">
      &copy; 2023 System Initiative Inc
    </div>
  </div>
</template>

<script setup lang="ts">
import { tw } from "@si/vue-lib";
import {
  Icon,
  useThemeContainer,
  userOverrideTheme,
  VButton2,
  Stack,
  DropdownMenu,
  DropdownMenuItem,
} from "@si/vue-lib/design-system";
import SiLogoUrl from "@si/vue-lib/brand-assets/si-logo.svg?url";
import SiLogoNoBorderUrl from "@si/vue-lib/brand-assets/si-logo-no-border.svg?url";
import { computed, onBeforeMount, ref, watch } from "vue";
import { useHead } from "@vueuse/head";
import { RouterView, useRoute, useRouter } from "vue-router";
import { useAuthStore } from "./store/auth.store";
import { BROWSER_IS_MOBILE } from "./lib/browser";

// provides the root theme value to all children, and returns that root theme to use below
const { theme: rootTheme } = useThemeContainer();

useHead(
  computed(() => ({
    bodyAttrs: {
      // add some base classes we need these type classes set for capsize plugin to work throughout
      // and add dark mode style/class
      class: tw`font-sans text-base leading-none ${rootTheme.value}`,
    },
    htmlAttrs: {
      style: `color-scheme: ${rootTheme.value};`,
    },
    link: [{ rel: "icon", href: SiLogoNoBorderUrl }],

    // set up title template and a default
    titleTemplate: "%s | System Init",
    title: "DevOps without papercuts",
  })),
);

const authStore = useAuthStore();
const checkAuthReq = authStore.getRequestStatus("CHECK_AUTH");

const userIsLoggedIn = computed(() => authStore.userIsLoggedIn);
const user = computed(() => authStore.user);

onBeforeMount(async () => {
  if (import.meta.env.SSR) return;

  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  authStore.CHECK_AUTH();
});

// some logic around pushing the user to the right page to go through onboarding
// could make sense to live in the router, but easier to interact with the auth loading state here
const router = useRouter();
const route = useRoute();
watch([checkAuthReq, route], () => {
  // if we're still checking auth, do nothing
  if (!checkAuthReq.value.isRequested || checkAuthReq.value.isPending) return;

  const currentRouteName = route.name as string;

  if (["print-legal"].includes(currentRouteName)) {
    return;
  }

  // if user is not logged in, kick back to login screen
  if (!userIsLoggedIn.value || !user.value) {
    if (!["login", "404"].includes(currentRouteName)) {
      return router.push({ name: "login" });
    }
    return;
  }

  // check user has agreed to TOS
  if (user.value.needsTosUpdate) {
    if (currentRouteName !== "review-legal") {
      // eslint-disable-next-line @typescript-eslint/no-floating-promises
      router.push({ name: "review-legal" });
    }
    return;
  }
  // check user has reviewed/completed their profile
  if (authStore.needsProfileUpdate) {
    if (currentRouteName !== "profile") {
      // eslint-disable-next-line @typescript-eslint/no-floating-promises
      return router.push({ name: "profile" });
    }
    return;
  }
});

function toggleTheme() {
  // TODO: could match our normal behaviour and allow setting to system/dark/light
  userOverrideTheme.value = rootTheme.value === "dark" ? "light" : "dark";
}

const profileMenuRef = ref<InstanceType<typeof DropdownMenu>>();
</script>
