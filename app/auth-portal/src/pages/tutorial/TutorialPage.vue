<!-- eslint-disable vue/no-v-html -->
<template>
  <div v-if="docsLoaded">
    <Confetti :active="activeStepSlug === 'next_steps'" start-top />
    <template v-if="!onboardingStore.stepsCompleted.github_access">
      <p>
        Thank you! We're double checking everything and getting your access to
        the github repository. Be on the lookout for the invitation from GitHub!
        It may take us an hour or more to process things, depending on our
        availability. If you have any questions, or run into trouble, you can
        email us at
        <a href="mailto:preview@systeminit.com" target="_blank"
          >preview@systeminit.com </a
        >, or hit us up on <a href="https://discord.com/asdf">discord</a>.
      </p>
    </template>
    <template v-else>
      <h2 class="mb-lg">Tutorial!</h2>

      <div class="flex gap-lg">
        <div class="flex-none w-[220px]">
          <div class="sticky top-sm flex flex-col gap-sm">
            <div
              v-for="step in tutorialSteps"
              :key="step.slug"
              :class="clsx('cursor-pointer flex items-center gap-xs')"
              @click="activeStepSlug = step.slug"
            >
              <Icon
                :name="
                  _.get(onboardingStore.stepsCompleted, step.slug)
                    ? 'check-circle'
                    : 'minus-circle'
                "
                size="lg"
                :class="
                  clsx(
                    _.get(onboardingStore.stepsCompleted, step.slug)
                      ? 'text-success-500'
                      : 'opacity-20',
                  )
                "
              />
              <a
                href="#"
                :class="
                  clsx(
                    'underline-link',
                    activeStepSlug === step.slug && '--active',
                  )
                "
                @click.prevent
              >
                {{ step.title }}
              </a>
            </div>
          </div>
        </div>
        <div class="grow">
          <RichText>
            <Component
              :is="tutorialSteps[activeStepSlug].component"
              v-if="tutorialSteps[activeStepSlug]"
            />
          </RichText>
          <VButton2
            class="w-full mt-lg"
            icon-right="arrow--right"
            variant="solid"
            tone="action"
            :disabled="!_.get(onboardingStore.stepsCompleted, activeStepSlug)"
            @click="stepContinueHandler"
          >
            Continue
          </VButton2>
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
import clsx from "clsx";
import { ComponentOptions, onBeforeMount, onMounted, ref } from "vue";
import { Icon, Inline, RichText, VButton2 } from "@si/vue-lib/design-system";
import Confetti from "@/components/Confetti.vue";

import FrieNDAModal from "@/components/FrieNDAModal.vue";
import { useOnboardingStore } from "@/store/onboarding.store";

const onboardingStore = useOnboardingStore();

const tutorialSteps = {} as Record<
  string,
  {
    title: string;
    slug: string;
    fileName: string;
    component: ComponentOptions;
  }
>;

const docsLoaded = ref(false);
onBeforeMount(async () => {
  const docImports = import.meta.glob(`@/content/tutorial/*.md`);
  for (const fileName in docImports) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const importedDoc = (await docImports[fileName]()) as any;
    const slug = fileName.replace(/.*\/\d\d-/, "").replace(".md", "");
    tutorialSteps[slug] = {
      title: importedDoc.attributes.title,
      slug,
      fileName,
      component: importedDoc.VueComponentWith({
        Icon,
        Inline,
      }),
    };
  }
  docsLoaded.value = true;
});

const activeStepSlug = ref("intro");
function stepContinueHandler() {
  const currentStepIndex = _.indexOf(
    _.keys(tutorialSteps),
    activeStepSlug.value,
  );
  const nextStepSlug = _.keys(tutorialSteps)[currentStepIndex + 1];
  activeStepSlug.value = nextStepSlug;
}

const friendaRef = ref();
onMounted(() => {
  if (
    onboardingStore.stepsCompleted.github_access &&
    !onboardingStore.stepsCompleted.frienda
  ) {
    friendaRef.value?.open();
  }
});
</script>
