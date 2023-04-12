<!-- eslint-disable vue/no-v-html -->
<template>
  <div>
    <Confetti :active="activeStepSlug === 'next_steps'" start-top />

    <template v-if="!onboardingStore.githubAccessGranted && !PREVIEW_MODE">
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
              v-for="step in TUTORIAL_STEPS"
              :key="step.slug"
              :class="
                clsx(
                  'cursor-pointer flex items-center gap-xs leading-5',
                  !stepsEnabled[step.slug] && 'pointer-events-none opacity-50',
                )
              "
              @click="stepSelectHandler(step.slug)"
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
                  TUTORIAL_STEPS[activeStepSlug] &&
                  !TUTORIAL_STEPS[activeStepSlug].hideWorkspaceLink
                "
                compact
                class="mt-xs"
              />
            </Transition>
          </div>
        </div>
        <div
          class="grow border-l border-neutral-300 dark:border-neutral-700 pl-lg relative overflow-x-hidden"
        >
          <RichText>
            <Component
              :is="TUTORIAL_STEPS[activeStepSlug].component"
              v-if="TUTORIAL_STEPS[activeStepSlug]"
            />
          </RichText>
          <VButton2
            class="w-full mt-lg"
            icon-right="arrow--right"
            variant="solid"
            tone="action"
            @click="stepContinueHandler"
          >
            Continue
          </VButton2>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import * as _ from "lodash-es";
import clsx from "clsx";

import { ComponentOptions, computed, onBeforeMount, ref, watch } from "vue";
import {
  Icon,
  IconNames,
  Inline,
  RichText,
  VButton2,
} from "@si/vue-lib/design-system";

import { RouterLink, useRoute, useRouter } from "vue-router";
import { useHead } from "@vueuse/head";
import Confetti from "@/components/Confetti.vue";

import WorkspaceLinkWidget from "@/components/WorkspaceLinkWidget.vue";
import { useOnboardingStore } from "@/store/onboarding.store";
import { useWorkspacesStore } from "@/store/workspaces.store";
import TutorialSurvey from "./TutorialSurvey.vue";

// enable working on tutorial without being logged in
const PREVIEW_MODE = !!import.meta.env.VITE_PREVIEW_TUTORIAL;

const onboardingStore = useOnboardingStore();

useHead({ title: "Tutorial" });

const TUTORIAL_STEPS = {} as Record<
  string,
  {
    title: string;
    slug: string;
    completeIcon?: IconNames;
    incompleteIcon?: IconNames;
    hideWorkspaceLink?: boolean;
    fileName: string;
    component: ComponentOptions;
  }
>;
const docImports = import.meta.glob(`@/content/tutorial/*.md`, {
  eager: true,
});
for (const fileName in docImports) {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const importedDoc = docImports[fileName] as any;
  const slug = fileName.replace(/.md$/, "").replace(/.*\/\d+-/, "");
  TUTORIAL_STEPS[slug] = {
    slug,
    ...importedDoc.attributes,
    fileName,
    component: importedDoc.VueComponentWith({
      Icon,
      Inline,
      WorkspaceLinkWidget,
      RouterLink,
      TutorialSurvey,
    }),
  };
}

const stepsEnabled = computed(() => {
  // steps are enabled up to the first non-completed step
  let enabled = true;
  return _.mapValues(TUTORIAL_STEPS, (_step, stepName) => {
    if (!enabled) return false;
    else if (onboardingStore.stepsCompleted[stepName]) return true;
    else {
      enabled = false;
      return true;
    }
  });
});
const lastEnabledStepSlug = computed(() =>
  _.last(_.keys(_.pickBy(stepsEnabled.value))),
);

const activeStepSlug = ref("");
async function stepContinueHandler() {
  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  onboardingStore.COMPLETE_TUTORIAL_STEP(activeStepSlug.value);

  const currentStepIndex = _.indexOf(
    _.keys(TUTORIAL_STEPS),
    activeStepSlug.value,
  );
  const nextStepSlug = _.keys(TUTORIAL_STEPS)[currentStepIndex + 1];

  // continue button on next steps goes to dashboard
  if (!nextStepSlug) {
    // eslint-disable-next-line @typescript-eslint/no-floating-promises
    router.push({ name: "dashboard" });
  } else {
    activeStepSlug.value = nextStepSlug;
  }
}

function stepSelectHandler(slug: string) {
  activeStepSlug.value = slug;
}

// we are preemptively loading workspaces for the workspace login widget
const workspacesStore = useWorkspacesStore();
onBeforeMount(() => {
  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  workspacesStore.LOAD_WORKSPACES();
});

const router = useRouter();
const route = useRoute();
watch(activeStepSlug, () => {
  const kebabStepSlug = activeStepSlug.value.replace(/_/g, "-");
  // if going from /tutorial to /tutorial/intro, we use replace, otherwise push

  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  router[route.params.stepSlug ? "push" : "replace"]({
    ...router.currentRoute,
    params: { stepSlug: kebabStepSlug },
  });
  window.scrollTo(0, 0);
});

watch(
  route,
  () => {
    const kebabStepSlug = (route.params.stepSlug as string) || "";
    const snakeStepSlug = kebabStepSlug.replace(/-/g, "_");
    if (!snakeStepSlug || !TUTORIAL_STEPS[snakeStepSlug]) {
      // default to last enabled step that is not completed
      activeStepSlug.value = lastEnabledStepSlug.value || "intro";
    } else {
      activeStepSlug.value = snakeStepSlug;
    }
  },
  { immediate: true },
);
</script>
