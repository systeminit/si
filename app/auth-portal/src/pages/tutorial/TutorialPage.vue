<!-- eslint-disable vue/no-v-html -->
<template>
  <div>
    <template v-if="waitingForAccess">
      <p>
        Thank you! We're double checking everything and getting your access to
        the github repository. Be on the lookout for the invitation from GitHub!
        It may take us an hour or more to process things, depending on our
        availability. If you have any questions, or run into trouble, you can
        email us at
        <a href="mailto:preview@systeminit.com" target="_blank"
          >preview@systeminit.com</a
        >, or hit us up on <a href="https://discord.com/asdf">discord</a>.
      </p>
    </template>
    <template v-else>
      <h2 class="mb-lg">Tutorial!</h2>

      <Icon :name="devFrontendOnline ? 'check' : 'loader'" />
      <Icon :name="devBackendOnline ? 'check' : 'loader'" />

      <div class="flex gap-md">
        <div class="flex-none w-[220px]">
          <div class="sticky top-sm flex flex-col gap-sm">
            <div
              v-for="step in tutorialSteps"
              :key="step.slug"
              class="cursor-pointer flex items-center gap-xs"
              @click="activeStepSlug = step.slug"
            >
              <Icon name="check-circle" />
              {{ step.title }}
            </div>
          </div>
        </div>
        <div class="grow">
          <Component
            :is="tutorialSteps[activeStepSlug].component"
            v-if="tutorialSteps && activeStepSlug"
          />
        </div>
      </div>

      <!-- <template v-if="loadTosReqStatus.isPending"> loading... </template>
    <template v-else-if="loadTosReqStatus.isError">
      Error loading TOS - {{ loadTosReqStatus.errorMessage }}
    </template>
    <template v-else-if="loadTosReqStatus.isSuccess">
      <div class="legal-markdown" v-html="authStore.tosDetails?.html" />

      <VormInput v-model="userAgreed" type="checkbox"
        >I have read and agree to the terms above</VormInput
      >
      <VButton2
        variant="ghost"
        icon="arrow--right"
        :disabled="disableContinueButton"
        :request-status="agreeTosReqStatus"
        @click="agreeButtonHandler"
      >
        Agree & Continue
      </VButton2>
    </template> -->
    </template>
    <FrieNDAModal ref="friendaRef" />
  </div>
</template>

<script setup lang="ts">
import * as _ from "lodash-es";
import {
  ComponentOptions,
  computed,
  onBeforeMount,
  onBeforeUnmount,
  onMounted,
  ref,
} from "vue";
import { Icon } from "@si/vue-lib/design-system";
import Axios from "axios";
import { useAuthStore } from "@/store/auth.store";
import FrieNDAModal from "@/components/FrieNDAModal.vue";

const tutorialSteps = {} as Record<
  string,
  {
    title: string;
    slug: string;
    fileName: string;
    component: ComponentOptions;
  }
>;
onBeforeMount(async () => {
  const docImports = import.meta.glob(`@/content/tutorial/*.md`);
  for (const fileName in docImports) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const importedDoc = (await docImports[fileName]()) as any;
    tutorialSteps[importedDoc.attributes.slug] = {
      title: importedDoc.attributes.title,
      slug: importedDoc.attributes.slug,
      fileName,
      component: importedDoc.VueComponent,
    };
  }
});

const activeStepSlug = ref();

const authStore = useAuthStore();

const devFrontendOnline = ref(false);
const devBackendOnline = ref(false);
async function checkDevEnvOnline() {
  try {
    const _req = await Axios.get("http://localhost:8080/up.txt");
    devFrontendOnline.value = true;
  } catch (err) {
    devFrontendOnline.value = false;
  }

  try {
    const _req = await Axios.get("http://localhost:8080/api/demo");
    devBackendOnline.value = true;
  } catch (err) {
    devBackendOnline.value = false;
  }
}

let checkInterval: ReturnType<typeof setInterval>;
onMounted(async () => {
  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  await checkDevEnvOnline();
  checkInterval = setInterval(checkDevEnvOnline, 5000);
});
onBeforeUnmount(() => {
  clearInterval(checkInterval);
});

const waitingForAccess = computed(() => authStore.waitingForAccess);

const friendaRef = ref();
onMounted(() => {
  if (!waitingForAccess.value) {
    friendaRef.value?.open();
  }
});
</script>
