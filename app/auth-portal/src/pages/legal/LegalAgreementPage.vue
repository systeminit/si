<!-- eslint-disable vue/no-v-html -->
<template>
  <div>
    <h2 class="mb-lg">Review the TOS!</h2>

    <!-- <div class="legal-markdown" v-html="authStore.tosDetails?.html" /> -->

    <template v-if="!docsLoaded"> Loading... </template>
    <template v-else>
      <p class="mb-lg">
        In order to use System Initiative, we need you to review and agree to
        our terms:
      </p>

      <div class="flex gap-md">
        <div class="flex-none w-[220px]">
          <div class="sticky top-md flex flex-col gap-md">
            <div
              v-for="doc in docs"
              :key="doc.fileName"
              :class="
                clsx(
                  'cursor-pointer flex items-center gap-xs',
                  doc.slug === activeDocSlug && '',
                )
              "
              @click="scrollToDoc(doc.slug)"
            >
              <!-- <Icon name="check-circle" /> -->
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
              <!-- <Icon name="download" size="sm" /> -->
            </div>
          </div>
        </div>
        <div class="grow">
          <div
            v-for="doc in docs"
            :key="doc.fileName"
            class="mb-xl"
            :data-doc-slug="doc.slug"
          >
            <RichText>
              <h2>{{ doc.title }}</h2>
              <Component :is="doc.component" />
            </RichText>
            <div class="">
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

          <Stack>
            <VormInput v-model="userAgreed" type="checkbox"
              >I have read and agree to the terms above</VormInput
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
    </template>
  </div>
</template>

<script setup lang="ts">
import * as _ from "lodash-es";
import { useRouter } from "vue-router";
import {
  ComponentOptions,
  computed,
  nextTick,
  onBeforeMount,
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
import { useAuthStore } from "@/store/auth.store";

const authStore = useAuthStore();
const router = useRouter();

const agreeTosReqStatus = authStore.getRequestStatus("AGREE_TOS");

const CURRENT_VERSION = "2023-03-30";

const docsLoaded = ref(false);
const docs = {} as Record<
  string,
  {
    title: string;
    slug: string;
    fileName: string;
    component: ComponentOptions;
  }
>;
onBeforeMount(async () => {
  const docImports = import.meta.glob(`@/content/legal/2023-03-30/*.md`);
  for (const fileName in docImports) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const importedDoc = (await docImports[fileName]()) as any;
    const slug = fileName.replace(/.md$/, "").replace(/.*\/\d+-/, "");
    docs[slug] = {
      title: importedDoc.attributes.title,
      slug,
      fileName,
      component: importedDoc.VueComponent,
    };
  }
  docsLoaded.value = true;
});

const userAgreed = ref(false);

const disableContinueButton = computed(() => {
  if (!userAgreed.value) return true;
  if (agreeTosReqStatus.value.isPending) return true;
  return false;
});

async function loadTosDetails() {
  if (import.meta.env.SSR) return;
  if (authStore.user?.needsTosUpdate === false) {
    return router.push({ name: "login-success" });
  }
}

watch(() => authStore.user?.needsTosUpdate, loadTosDetails, {
  immediate: true,
});

async function agreeButtonHandler() {
  const agreeReq = await authStore.AGREE_TOS(CURRENT_VERSION);
  if (agreeReq.result.success) {
    await router.push({ name: "login-success" });
  }
}

// const activeDocSlug = ref("tos");

function scrollToDoc(slug: string) {
  const el = document.querySelector(`[data-doc-slug="${slug}"]`);
  el?.scrollIntoView({ behavior: "smooth" });
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
  { threshold: [0] },
);

watch(docsLoaded, () => {
  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  nextTick(activateObserver);
});

function activateObserver() {
  const sectionEls = document.querySelectorAll("[data-doc-slug]");
  sectionEls.forEach((el) => {
    observer.observe(el);
  });
}

onMounted(() => {
  // window.addEventListener("scroll", scrollHandler);
});
onBeforeUnmount(() => {
  observer.disconnect();
});
</script>
