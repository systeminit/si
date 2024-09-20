<!-- eslint-disable vue/no-v-html -->
<template>
  <div>
    <!-- <div class="legal-markdown" v-html="authStore.tosDetails?.html" /> -->

    <RichText class="mb-xl">
      <template v-if="viewOnlyMode">
        <h1>System Initiative Legal Documents</h1>
        <p><i>Last updated 2023-03-30</i></p>
      </template>
      <template v-else>
        <h1>Review our legal docs:</h1>
        <p>
          In order to use System Initiative, we need you to review and agree to
          our terms:
        </p>
      </template>
    </RichText>

    <div class="flex gap-lg">
      <div class="flex-none w-[220px]">
        <div class="sticky top-md flex flex-col gap-md">
          <div
            v-for="doc in LEGAL_DOCS_CONTENT[currentVersion]"
            :key="doc.fileName"
            :class="
              clsx(
                'cursor-pointer flex items-center gap-xs',
                doc.slug === activeDocSlug && '',
              )
            "
            @click="scrollToDoc(doc.slug)"
          >
            <a
              :class="
                clsx(
                  'underline-link w-auto',
                  doc.slug === activeDocSlug && '--active',
                )
              "
              href="#"
              @click.prevent
            >
              {{ doc.title }}
            </a>
          </div>
          <div
            v-if="!viewOnlyMode"
            class="cursor-pointer"
            @click="scrollToDoc('agree')"
          >
            <a
              :class="
                clsx(
                  'underline-link w-auto',
                  activeDocSlug === 'agree' && '--active',
                )
              "
              href="#"
              @click.prevent
            >
              Agree and Continue
            </a>
          </div>
        </div>
      </div>
      <div
        class="grow border-l border-neutral-300 dark:border-neutral-700 pl-lg"
      >
        <div
          v-for="(doc, key, index) in LEGAL_DOCS_CONTENT[currentVersion]"
          :key="key"
          :class="
            !viewOnlyMode ||
            index !== Object.keys(LEGAL_DOCS_CONTENT).length - 1
              ? 'mb-xl'
              : ''
          "
          :data-doc-slug="doc.slug"
        >
          <RichText class="text-sm">
            <Component :is="doc.component" />
          </RichText>
          <div class="mt-md">
            <VButton
              :linkTo="{
                name: 'print-legal',
                params: { docVersion: currentVersion, docSlug: doc.slug },
              }"
              icon="download"
              size="sm"
              target="_blank"
              tone="shade"
              variant="soft"
              >Print / Download
            </VButton>
          </div>
        </div>

        <Stack v-if="!viewOnlyMode" data-doc-slug="agree">
          <VormInput v-model="userAgreed" type="checkbox"
            >I have read and agree to the terms above
          </VormInput>
          <VButton
            :disabled="disableContinueButton"
            :requestStatus="agreeTosReqStatus"
            icon="arrow--right"
            variant="solid"
            @click="agreeButtonHandler"
          >
            Agree & Continue
          </VButton>
        </Stack>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { useRoute, useRouter } from "vue-router";
import {
  computed,
  onBeforeUnmount,
  onMounted,
  reactive,
  ref,
  watch,
} from "vue";
import { RichText, Stack, VButton, VormInput } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { useHead } from "@vueuse/head";
import { TosVersion } from "@si/ts-lib/src/terms-of-service";
import { useAuthStore } from "@/store/auth.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { LEGAL_DOCS_CONTENT } from "./load-docs";

const authStore = useAuthStore();
const featureFlagStore = useFeatureFlagsStore();
const router = useRouter();
const route = useRoute();

// this page handles 2 modes, public view-only and review/agreement
const viewOnlyMode = route.name === "legal";

const agreeTosReqStatus = authStore.getRequestStatus("AGREE_TOS");

const currentVersion = computed(() =>
  featureFlagStore.SAAS_RELEASE ? TosVersion.v20240919 : TosVersion.v20230330,
);

const userAgreed = ref(false);

useHead({ title: "Legal" });

const disableContinueButton = computed(() => {
  if (!userAgreed.value) return true;
  if (agreeTosReqStatus.value.isPending) return true;
  return false;
});

async function loadTosDetails() {
  if (import.meta.env.SSR) return;
  if (viewOnlyMode) return;
  if (authStore.user?.needsTosUpdate === false) {
    return router.push({ name: "login-success" });
  }
}

watch(() => authStore.user?.needsTosUpdate, loadTosDetails, {
  immediate: true,
});

async function agreeButtonHandler() {
  const isFirstAgreement = !authStore.user?.agreedTosVersion;
  const agreeReq = await authStore.AGREE_TOS(currentVersion.value);
  if (agreeReq.result.success) {
    await router.push({ name: isFirstAgreement ? "profile" : "login-success" });
  }
}

const scrollingToSlug = ref<string | undefined>(undefined);
function scrollToDoc(slug: string) {
  const el = document.querySelector(`[data-doc-slug="${slug}"]`);
  el?.scrollIntoView({ behavior: "smooth" });
  activeDocSlug.value = slug;
  scrollingToSlug.value = slug;
}

// track all intersecting sections, and current one should be the last in the list
const intersectingDocs = reactive<Record<string, boolean>>({});
const activeDocSlug = ref("tos");
const observer = new IntersectionObserver(
  (entries) => {
    const entry = entries[0];
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    const slug = entry.target.getAttribute("data-doc-slug")!;
    if (entry.isIntersecting && slug !== "agree") {
      intersectingDocs[slug] = true;
    } else {
      intersectingDocs[slug] = false;
    }

    activeDocSlug.value =
      _.last(_.keys(_.pickBy(intersectingDocs))) || activeDocSlug.value;
  },
  { threshold: [0] },
);
watch(activeDocSlug, () => {
  /* eslint-disable @typescript-eslint/no-floating-promises */
  router.replace({ ...route, hash: `#${activeDocSlug.value}` });
});

const enableObserver = () => {
  const sectionEls = document.querySelectorAll("[data-doc-slug]");
  sectionEls.forEach((el) => {
    observer.observe(el);
  });
};

onMounted(() => {
  enableObserver();

  window.addEventListener("scrollend", () => {
    if (scrollingToSlug.value) {
      activeDocSlug.value = scrollingToSlug.value;
      scrollingToSlug.value = undefined;
    }
  });

  // if url refers to a specific doc, we'll scroll to it right away
  if (route.hash) {
    scrollToDoc(route.hash.replace("#", ""));
  }
});
onBeforeUnmount(() => {
  observer.disconnect();
});
</script>
