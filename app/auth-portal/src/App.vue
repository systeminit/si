<template>
  <div>
    <template v-if="route.name === 'print-legal'">
      <RouterView />
    </template>
    <template v-else>
      <div class="m-auto text p-2xl">
        <div class="fixed left-0 bottom-0 p-sm">
          <VButton2
            :icon="rootTheme === 'dark' ? 'moon' : 'sun'"
            tone="shade"
            variant="transparent"
            size="md"
            @click="toggleTheme"
          />
        </div>

        <template v-if="!checkAuthReq.isRequested || checkAuthReq.isPending">
          <Icon name="loader" size="xl" />
          checking if you are logged in...
        </template>
        <template v-else>
          <nav>
            <img :src="SiLogoUrl" class="w-[40px] h-[40px]" />

            <template v-if="userIsLoggedIn">
              <p class="font-bold my-sm">Hi {{ authStore.bestUserLabel }}!</p>

              <div class="flex gap-md">
                <RouterLink :to="{ name: 'profile' }">Profile</RouterLink>
                <RouterLink :to="{ name: 'tutorial' }">Tutorial</RouterLink>
                <RouterLink :to="{ name: 'dashboard' }">Dashboard</RouterLink>
                <RouterLink :to="{ name: 'logout' }">Logout</RouterLink>
              </div>
              <!-- <VButton2
            tone="neutral"
            icon="x"
            variant="ghost"
            :href="`${API_URL}/auth/logout`"
            >Log out!</VButton2
          > -->
            </template>
            <template v-else>
              <VButton2 :href="`${API_URL}/auth/login`">Log in!</VButton2>
            </template>
          </nav>
          <div class="p-md border m-md">
            <RouterView />
          </div>
        </template>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { tw } from "@si/vue-lib";
import {
  Icon,
  useThemeContainer,
  userOverrideTheme,
  VButton2,
} from "@si/vue-lib/design-system";
import SiLogoUrl from "@si/vue-lib/brand-assets/si-logo.svg?url";
import SiLogoNoBorderUrl from "@si/vue-lib/brand-assets/si-logo-no-border.svg?url";
import { computed, onBeforeMount, watch } from "vue";
import { useHead } from "@vueuse/head";
import { RouterView, useRoute, useRouter } from "vue-router";
import { useAuthStore } from "./store/auth.store";

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

const API_URL = import.meta.env.VITE_AUTH_API_URL;

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
</script>

<style></style>
