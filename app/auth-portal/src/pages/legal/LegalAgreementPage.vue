<!-- eslint-disable vue/no-v-html -->
<template>
  <div>
    <!-- <div class="legal-markdown" v-html="authStore.tosDetails?.html" /> -->

    <RichText class="mb-xl">
      <template v-if="viewOnlyMode">
        <h1>System Initiative Legal Docs</h1>
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
              v-for="doc in LEGAL_DOCS_CONTENT"
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
        </div>
      </div>
      <div
          class="grow border-l border-neutral-300 dark:border-neutral-700 pl-lg"
      >
        <div
            v-for="doc in LEGAL_DOCS_CONTENT"
            :key="doc.fileName"
            class="mb-xl"
            :data-doc-slug="doc.slug"
        >
          <RichText class="text-sm">
            <Component :is="doc.component"/>
          </RichText>
          <div class="mt-md">
            <VButton2
                icon="download"
                variant="soft"
                tone="shade"
                size="sm"
                :link-to="{
                name: 'print-legal',
                params: { docVersion: CURRENT_VERSION, docSlug: doc.slug },
              }"
                target="_blank"
            >Print / Download
            </VButton2>
          </div>
        </div>

        <Stack v-if="!viewOnlyMode">
          <VormInput v-model="userAgreed" type="checkbox"
          >I have read and agree to the terms above
          </VormInput
          >
          <VButton2
              variant="solid"
              icon="arrow--right"
              :disabled="disableContinueButton"
              :request-status="agreeTosReqStatus"
              @click="agreeButtonHandler"
          >
            Agree & Continue
          </VButton2>
        </Stack>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import * as _ from "lodash-es";
import {useRoute, useRouter} from "vue-router";
import {
  computed,
  onBeforeUnmount,
  onMounted,
  reactive,
  ref,
  watch,
} from "vue";
import {
  RichText,
  Stack,
  VButton2,
  VormInput,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import {useAuthStore} from "@/store/auth.store";
import {LEGAL_DOCS_CONTENT} from "./load-docs";
import {useHead} from "@vueuse/head";

const authStore = useAuthStore();
const router = useRouter();
const route = useRoute();

// this page handles 2 modes, public view-only and review/agreement
const viewOnlyMode = route.name === "legal";

const agreeTosReqStatus = authStore.getRequestStatus("AGREE_TOS");

const CURRENT_VERSION = "2023-03-30";

const userAgreed = ref(false);

useHead({title: "Legal"});

const disableContinueButton = computed(() => {
  if (!userAgreed.value) return true;
  if (agreeTosReqStatus.value.isPending) return true;
  return false;
});

async function loadTosDetails() {
  if (import.meta.env.SSR) return;
  if (viewOnlyMode) return;
  if (authStore.user?.needsTosUpdate === false) {
    return router.push({name: "login-success"});
  }
}

watch(() => authStore.user?.needsTosUpdate, loadTosDetails, {
  immediate: true,
});

async function agreeButtonHandler() {
  const agreeReq = await authStore.AGREE_TOS(CURRENT_VERSION);
  if (agreeReq.result.success) {
    await router.push({name: "login-success"});
  }
}

function scrollToDoc(slug: string) {
  const el = document.querySelector(`[data-doc-slug="${slug}"]`);
  el?.scrollIntoView({behavior: "smooth"});
}

// track all intersecting secitons, and current one should be the last in the list
const intersectingDocs = reactive<Record<string, boolean>>({});
const activeDocSlug = ref("tos");
const observer = new IntersectionObserver(
    (entries) => {
      const entry = entries[0];
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const slug = entry.target.getAttribute("data-doc-slug")!;
      if (entry.isIntersecting) {
        intersectingDocs[slug] = true;
      } else {
        intersectingDocs[slug] = false;
      }

      activeDocSlug.value = _.last(_.keys(_.pickBy(intersectingDocs))) || "tos";
    },
    {threshold: [0]},
);
watch(activeDocSlug, () => {
  /* eslint-disable @typescript-eslint/no-floating-promises */
  router.replace({...route, params: {docSlug: activeDocSlug.value}});
});

onMounted(() => {
  const sectionEls = document.querySelectorAll("[data-doc-slug]");
  sectionEls.forEach((el) => {
    observer.observe(el);
  });

  // if url refers to a specific doc, we'll scroll to it right away
  if (route.params.docSlug) {
    scrollToDoc(route.params.docSlug as string);
  }
});
onBeforeUnmount(() => {
  observer.disconnect();
});
</script>
