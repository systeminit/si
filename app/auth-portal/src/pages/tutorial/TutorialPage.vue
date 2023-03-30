<!-- eslint-disable vue/no-v-html -->
<template>
  <div v-if="docsLoaded">
    <Confetti :active="activeStepSlug === 'next_steps'" start-top />
    <template v-if="!onboardingStore.stepsCompleted.github_access">
      <RichText>
        <h3>We're getting you access</h3>
        <p>
          Thank you! We're double checking everything and getting your access to
          the github repository. Be on the lookout for the invitation from
          GitHub! It may take us an hour or more to process things, depending on
          our availability. If you have any questions, or run into trouble, you
          can email us at
          <a href="mailto:preview@systeminit.com" target="_blank"
            >preview@systeminit.com</a
          >, or hit us up on <a href="https://discord.com/asdf">discord</a>.
        </p>
      </RichText>
    </template>
    <template v-else>
      <div class="flex gap-lg">
        <div class="flex-none w-[250px]">
          <div class="sticky top-md flex flex-col gap-sm">
            <div
              v-for="step in tutorialSteps"
              :key="step.slug"
              :class="clsx('cursor-pointer flex items-center gap-xs leading-5')"
              @click="activeStepSlug = step.slug"
            >
              <Icon
                :name="
                  _.get(onboardingStore.stepsCompleted, step.slug)
                    ? step.completeIcon || 'check-circle'
                    : step.incompleteIcon || 'minus-circle'
                "
                size="lg"
                :class="
                  clsx(
                    '-ml-[2px]',
                    _.get(onboardingStore.stepsCompleted, step.slug)
                      ? 'text-success-500'
                      : activeStepSlug !== step.slug
                      ? 'text-neutral-400'
                      : '',
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

            <Transition
              class="duration-500"
              enter-from-class="transform opacity-0"
              enter-to-class="opacity-100"
              leave-to-class="opacity-0"
            >
              <WorkspaceLinkWidget
                v-if="
                  activeStepSlug !== 'intro' && activeStepSlug !== 'dev_setup'
                "
                compact
                class="mt-xs"
              />
            </Transition>
          </div>
        </div>
        <div
          class="grow border-l border-neutral-300 dark:border-neutral-700 pl-lg"
        >
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
import {
  Icon,
  IconNames,
  Inline,
  RichText,
  VButton2,
} from "@si/vue-lib/design-system";
import { RouterLink } from "vue-router";
import Confetti from "@/components/Confetti.vue";

import WorkspaceLinkWidget from "@/components/WorkspaceLinkWidget.vue";
import FrieNDAModal from "@/components/FrieNDAModal.vue";
import { useOnboardingStore } from "@/store/onboarding.store";
import { useWorkspacesStore } from "@/store/workspaces.store";

const onboardingStore = useOnboardingStore();

const tutorialSteps = {} as Record<
  string,
  {
    title: string;
    slug: string;
    completeIcon?: IconNames;
    incompleteIcon?: IconNames;
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
      completeIcon: importedDoc.attributes.completeIcon,
      incompleteIcon: importedDoc.attributes.incompleteIcon,
      fileName,
      component: importedDoc.VueComponentWith({
        Icon,
        Inline,
        "workspace-link-widget": WorkspaceLinkWidget,
        "router-link": RouterLink,
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

const workspacesStore = useWorkspacesStore();
onBeforeMount(() => {
  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  workspacesStore.LOAD_WORKSPACES();
});
</script>
