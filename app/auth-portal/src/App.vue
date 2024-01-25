<template>
  <div class="text-black dark:text-white">
    <DeployNotification />

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
    <template
      v-else-if="
        !checkAuthReq.isRequested ||
        checkAuthReq.isPending ||
        !hasCheckedOnboardingStatus
      "
    >
      <div
        class="fixed inset-0 flex flex-col items-center justify-center p-md gap-sm"
      >
        <SiLogo class="w-[60px] h-[60px] animate-pulse" />
        <div
          :class="
            clsx(
              'w-[140px] h-[2px] bg-neutral-300 dark:bg-neutral-800 relative',
            )
          "
        >
          <div
            :class="
              clsx(
                'h-[2px] bg-action-500 transition-all duration-500',
                runAuthProgressBar ? 'w-[90%]' : 'w-[5%]',
              )
            "
          ></div>
        </div>
      </div>
    </template>
    <template v-else>
      <div class="flex flex-col min-h-screen">
        <header class="flex p-md items-center">
          <RouterLink
            id="header-logo"
            :to="{ name: 'home' }"
            class="mr-md shrink-0 relative"
          >
            <div id="header-logo-inner">
              <SiLogo class="w-[40px] h-[40px]" />
            </div>
          </RouterLink>

          <template v-if="userIsLoggedIn">
            <nav class="flex gap-md font-bold items-center">
              <template
                v-if="
                  !(
                    authStore.needsProfileUpdate ||
                    authStore.user?.needsTosUpdate ||
                    !authStore.user?.onboardingDetails?.reviewedProfile
                  )
                "
              >
                <RouterLink :to="{ name: 'tutorial' }" class="underline-link">
                  Tutorial
                </RouterLink>
                <RouterLink :to="{ name: 'dashboard' }" class="underline-link">
                  Dashboard
                </RouterLink>
              </template>
            </nav>

            <nav class="flex gap-sm mr-xs items-center ml-auto">
              <a
                href="https://github.com/systeminit/si"
                target="_blank"
                class="hover:dark:text-action-300 hover:text-action-700"
              >
                <Icon name="logo-github" />
              </a>
              <a
                href="https://discord.gg/system-init"
                target="_blank"
                class="hover:dark:text-action-300 hover:text-action-700"
              >
                <Icon name="logo-discord" />
              </a>
              <span class="opacity-50">|</span>
            </nav>

            <VButton
              class="flex items-center gap-sm children:pointer-events-none"
              variant="transparent"
              tone="shade"
              @mousedown.prevent
              @click.prevent="profileMenuRef?.open($event) || _.noop"
            >
              <div class="mr-xs">Hi {{ authStore.bestUserLabel }}!</div>
              <template #iconRight>
                <img
                  v-if="user?.pictureUrl"
                  :src="user?.pictureUrl"
                  class="w-[32px] h-[32px] block rounded-full"
                  referrerpolicy="no-referrer"
                />
                <Icon v-else name="user-circle" />
              </template>
            </VButton>
          </template>
        </header>
        <DropdownMenu ref="profileMenuRef" forceAlignRight>
          <DropdownMenuItem
            v-if="route.name !== 'review-legal'"
            icon="user-circle"
            linkToNamedRoute="profile"
            >Profile
          </DropdownMenuItem>
          <DropdownMenuItem icon="logout" linkToNamedRoute="logout"
            >Log out
          </DropdownMenuItem>
        </DropdownMenu>

        <!-- dark/light mode toggle, floating in bottom left -->
        <div class="fixed left-0 bottom-0 p-sm">
          <VButton
            :icon="rootTheme === 'dark' ? 'moon' : 'sun'"
            tone="shade"
            variant="transparent"
            rounded
            size="md"
            @click="toggleTheme"
          />
        </div>

        <div class="">
          <div class="m-auto max-w-[1200px] min-w-[520px]">
            <div
              class="m-lg p-lg dark:bg-neutral-800 bg-neutral-200 rounded-md"
            >
              <!-- email verification warning w/ buttons to help resolve -->
              <ErrorMessage v-if="user && !user?.emailVerified" class="mb-lg">
                <Inline spacing="md" alignY="center">
                  <p>Please verify your email address</p>

                  <VButton
                    tone="shade"
                    variant="transparent"
                    size="sm"
                    :requestStatus="refreshAuth0Req"
                    @click="authStore.REFRESH_AUTH0_PROFILE"
                    >Already verified?</VButton
                  >
                  <!-- normally we'd use the ErrorMessage component, but we're already using it as the wrapper here for a sort of alert -->
                  <p v-if="refreshAuth0Req.isError">
                    ERROR: {{ refreshAuth0Req.errorMessage }}
                  </p>

                  <VButton
                    v-if="!resendEmailVerificationReq.isSuccess"
                    tone="shade"
                    variant="transparent"
                    size="sm"
                    :requestStatus="resendEmailVerificationReq"
                    @click="authStore.RESEND_EMAIL_VERIFICATION"
                    >Resend Email</VButton
                  >
                  <p v-if="resendEmailVerificationReq.isError">
                    ERROR: {{ resendEmailVerificationReq.errorMessage }}
                  </p>
                </Inline>
              </ErrorMessage>

              <RouterView />
            </div>
          </div>
        </div>

        <footer
          class="mt-auto flex text-sm p-sm gap-sm justify-end text-neutral-800 dark:text-neutral-200 min-w-[350px]"
        >
          <a
            class="hover:underline hover:dark:text-action-300 hover:text-action-700"
            href="mailto:help@systeminit.com"
            target="_blank"
            >Help</a
          >
          <span class="opacity-50">|</span>
          <RouterLink
            class="hover:underline hover:dark:text-action-300 hover:text-action-700"
            :to="{ name: 'legal' }"
            >Legal
          </RouterLink>
          <span class="opacity-50">|</span>
          <div class="text-center whitespace-nowrap">
            &copy; System Initiative, Inc.
          </div>
        </footer>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import * as _ from "lodash-es";
import { tw } from "@si/vue-lib";
import {
  Icon,
  useThemeContainer,
  userOverrideTheme,
  VButton,
  DropdownMenu,
  DropdownMenuItem,
  ErrorMessage,
  Inline,
} from "@si/vue-lib/design-system";
import "floating-vue/dist/style.css";

import SiLogo from "@si/vue-lib/brand-assets/si-logo-symbol.svg?component";
import SiLogoUrlLight from "@si/vue-lib/brand-assets/si-logo-symbol-white-bg.svg?url";
import SiLogoUrlDark from "@si/vue-lib/brand-assets/si-logo-symbol-black-bg.svg?url";
import { computed, onBeforeMount, onMounted, ref, watch } from "vue";
import { useHead } from "@vueuse/head";
import { RouterView, useRoute, useRouter } from "vue-router";
import clsx from "clsx";
import storage from "local-storage-fallback";
import { useAuthStore } from "./store/auth.store";
import { BROWSER_IS_MOBILE } from "./lib/browser";
import DeployNotification from "./components/DeployNotification.vue";

// provides the root theme value to all children, and returns that root theme to use below
const { theme: rootTheme } = useThemeContainer();

useHead(
  computed(() => ({
    bodyAttrs: {
      // add some base classes we need these type classes set for capsize plugin to work throughout
      // and add dark mode style/class
      class: tw`font-sans text-base leading-none text-black dark:text-white`,
    },
    htmlAttrs: {
      style: () => `color-scheme: ${rootTheme.value};`,
      class: () => rootTheme.value,
    },
    link: [
      {
        rel: "icon",
        href: rootTheme.value === "light" ? SiLogoUrlLight : SiLogoUrlDark,
      },
    ],

    // set up title template and a default
    titleTemplate: "SI | %s",
  })),
);

onMounted(() => {
  // useHead not properly clearing existing dark/light class from pre-render...?
  // should be able to remove...
  document.documentElement.classList.remove("dark", "light");
});

const authStore = useAuthStore();
const checkAuthReq = authStore.getRequestStatus("CHECK_AUTH");

const refreshAuth0Req = authStore.getRequestStatus("REFRESH_AUTH0_PROFILE");
const resendEmailVerificationReq = authStore.getRequestStatus(
  "RESEND_EMAIL_VERIFICATION",
);

const userIsLoggedIn = computed(() => authStore.userIsLoggedIn);
const user = computed(() => authStore.user);

onBeforeMount(async () => {
  if (import.meta.env.SSR) return;

  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  authStore.CHECK_AUTH();
});

const runAuthProgressBar = ref(false);
onMounted(() => {
  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  setTimeout(() => {
    runAuthProgressBar.value = true;
  }, 10);
});

const hasCheckedOnboardingStatus = ref(false);

// some logic around pushing the user to the right page to go through onboarding
// could make sense to live in the router, but easier to interact with the auth loading state here
const router = useRouter();
const route = useRoute();
watch([checkAuthReq, route], () => {
  // if we're still checking auth, do nothing
  if (!checkAuthReq.value.isRequested || checkAuthReq.value.isPending) return;

  // loading state is shown above until this flips
  // so stop the RouterView from loading/showing a page that it shouldnt yet
  setTimeout(() => {
    hasCheckedOnboardingStatus.value = true;
  });

  const currentRouteName = route.name as string;

  function saveLoginSuccessRedirect() {
    const fullPath = route.fullPath;
    if (fullPath !== "/") {
      storage.setItem("SI-LOGIN-REDIRECT", fullPath);
    }
  }

  if (["print-legal", "logout", "logout-success"].includes(currentRouteName)) {
    return;
  }

  // if user is not logged in, kick back to login screen
  if (!userIsLoggedIn.value || !user.value) {
    if (
      ![
        "login",
        "signup",
        "404",
        "legal",
        // allow viewing tutorial without login if VITE_PREVIEW_TUTORIAL set in env
        ...(import.meta.env.VITE_PREVIEW_TUTORIAL ? ["tutorial"] : []),
      ].includes(currentRouteName)
    ) {
      saveLoginSuccessRedirect();
      return router.push({ name: "login" });
    }
    return;
  }

  // check user has agreed to TOS
  if (user.value.needsTosUpdate) {
    if (currentRouteName !== "review-legal") {
      saveLoginSuccessRedirect();
      // eslint-disable-next-line @typescript-eslint/no-floating-promises
      router.push({ name: "review-legal" });
    }
    return;
  }

  // check user has reviewed/completed their profile
  if (
    authStore.needsProfileUpdate ||
    !authStore.user?.onboardingDetails?.reviewedProfile
  ) {
    if (currentRouteName !== "profile" && currentRouteName !== "legal") {
      saveLoginSuccessRedirect();
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

<style lang="less">
#app {
  min-height: 100vh;
}

#header-logo {
  cursor: pointer;

  svg {
    position: relative;
    z-index: 2;
  }

  &:hover {
    #header-logo-inner {
      animation: spin3d 0.8s linear infinite;

      &:before {
        content: "";
        display: block;
        position: absolute;
        width: 100%;
        height: 100%;
        left: 0;
        top: 0;
        z-index: 1;
        border-radius: 6px;
        background: linear-gradient(-45deg, #ee7752, #e73c7e, #23a6d5, #23d5ab);
        background-size: 400% 400%;
        animation: gradient 2s ease infinite;
        animation-direction: alternate;
      }
    }
  }
}

@keyframes spin3d {
  0% {
    transform: rotateX(0deg) rotateY(0deg);
  }
  50% {
    transform: rotateX(20deg) rotateY(90deg);
  }
  51% {
    transform: rotateX(-20deg) rotateY(90deg);
  }
  100% {
    transform: rotateX(0deg) rotateY(0deg);
  }

  // 100% {
  //   transform: rotateX(0deg) rotateY(-90deg);
  // }
}

@keyframes gradient {
  0% {
    background-position: 0% 0%;
  }
  // 50% {
  //   background-position: 100% 50%;
  // }
  100% {
    background-position: 0% 100%;
  }
}

.v-popper__arrow-container {
  display: none;
}

.v-popper__inner {
  border-radius: 0px !important;
  border-color: #5a5a5a !important;
  max-width: 80vw;
  overflow-wrap: break-word;
}

.bg-caution-lines {
  background: repeating-linear-gradient(
    -45deg,
    #000,
    #000 10px,
    #333 10px,
    #333 20px
  );
}
</style>
