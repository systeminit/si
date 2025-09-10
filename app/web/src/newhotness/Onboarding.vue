<template>
  <div
    data-testid="lobby"
    class="absolute w-screen h-screen bg-neutral-900 z-[1000] flex flex-col items-center text-white"
  >
    <div class="flex flex-row items-center justify-between w-full px-sm py-xs">
      <SiLogo class="block h-md w-md flex-none" />
      <NewButton
        v-if="isSkippable"
        aria-label="Skip Onboarding"
        class="text-neutral-400 hover:text-white font-normal"
        label="Skip"
        tone="empty"
        @click="closeOnboarding(true)"
      />
    </div>
    <div class="flex flex-col w-[856px]">
      <div
        :class="
          clsx(
            'rounded font-mono text-sm',
            'flex flex-col justify-end gap-xs mb-md',
          )
        "
      >
        <ScrollingOutputLine
          v-for="(data, index) in visibleSentences"
          :key="index"
          :message="data.sentence"
          :isActive="index === visibleItemsCount - 1"
          :isLastElement="index === visibleItemsCount - 1"
          fast
        />
      </div>
      <Transition
        enterActiveClass="duration-300 ease-out"
        enterFromClass="transform opacity-0"
        enterToClass="opacity-100"
        leaveActiveClass="delay-1000 duration-200 ease-in"
        leaveFromClass="opacity-100"
        leaveToClass="transform opacity-0"
        :onAfterEnter="() => userInputAreaRef?.open()"
      >
        <div v-if="showFormItems" class="flex flex-col gap-sm">
          <ErrorMessage
            v-if="initializeApiError"
            class="rounded text-md p-xs"
            icon="x-circle"
            variant="block"
          >
            Something went wrong. Please retry to validate your credential and
            region.
          </ErrorMessage>
          <!-- Step 1: User Input -->
          <OnboardingCollapsingArea
            ref="userInputAreaRef"
            @opened="aiTutorialAreaRef?.close()"
          >
            <template #header>
              <div class="flex flex-row items-center justify-between">
                <span
                  :class="
                    initializeRequestSentAndSuccessful && 'text-success-200'
                  "
                >
                  1. Enter an AWS credential and select a region
                </span>
                <Icon
                  :name="
                    initializeRequestSentAndSuccessful
                      ? 'check-circle'
                      : 'loader'
                  "
                  :class="
                    clsx(
                      !submittedOnboardRequest && 'hidden',
                      initializeRequestSentAndSuccessful && 'text-success-400',
                    )
                  "
                />
              </div>
            </template>
            <template #body>
              <!-- Credential -->
              <div class="flex flex-col border border-neutral-600 p-sm gap-sm">
                <div class="flex flex-row justify-between items-center">
                  <label for="aws-credential-name" class="basis-0 grow">
                    Name your credential*
                  </label>
                  <input
                    id="aws-credential-name"
                    v-model="credentialName"
                    :class="
                      clsx(
                        'h-lg p-xs text-sm border font-mono cursor-text basis-0 grow',
                        'focus:outline-none focus:ring-0 focus:z-10',
                        'text-shade-0 bg-black border-neutral-600 focus:border-action-300',
                      )
                    "
                  />
                </div>
                <!-- Secret Values -->
                <ErrorMessage
                  class="rounded-md text-md px-xs py-xs bg-action-900 mt-xs"
                  tone="action"
                  variant="block"
                  noIcon
                >
                  Pro tip: Paste the full Bash environment block into the first
                  field — we’ll auto-fill the rest.
                </ErrorMessage>

                <div class="flex flex-col">
                  <div
                    v-for="(field, title) in secretFormFields"
                    :key="title"
                    :class="'flex flex-row justify-between items-center text-sm'"
                  >
                    <label
                      class="basis-0 grow flex flex-row items-center gap-2xs"
                    >
                      {{ title }} {{ field.required && "*" }}
                    </label>
                    <div class="flex flex-row basis-0 grow">
                      <input
                        v-model="field.ref"
                        :type="field.type"
                        :class="
                          clsx(
                            'h-lg p-xs text-sm border font-mono cursor-text grow',
                            'focus:outline-none focus:ring-0 focus:z-10',
                            'text-shade-0 bg-black border-neutral-600 focus:border-action-300',
                          )
                        "
                        placeholder="***"
                        data-lpignore="true"
                        data-1p-ignore
                        data-bwignore
                        data-form-type="other"
                        @paste="(ev: ClipboardEvent) => tryMatchOnPaste(ev)"
                      />
                      <!-- TODO(Wendy+Victor) - let's get this working, see SecretInput for an example -->
                      <NewButton
                        icon="eye"
                        class="hidden"
                        @click="toggleVisibility(field)"
                      />
                    </div>
                  </div>
                </div>

                <div class="text-neutral-400 text-xs text-end">
                  All data in this section will be encrypted
                </div>
              </div>
              <!-- Region -->
              <div class="border border-neutral-600 flex flex-col p-sm gap-sm">
                <div>Pick a region*</div>
                <div class="flex flex-row items-center gap-sm">
                  <div
                    v-for="region in pickerRegions"
                    :key="region.value"
                    :class="
                      clsx(
                        'flex flex-row grow items-center border px-sm py-sm gap-sm hover:border-neutral-300 cursor-pointer',
                        region.value === awsRegion
                          ? 'border-neutral-300'
                          : 'border-neutral-600',
                      )
                    "
                    @click="awsRegion = region.value"
                  >
                    <Icon
                      :name="
                        region.value === awsRegion
                          ? 'check-circle'
                          : 'circle-empty'
                      "
                    />
                    <div class="flex flex-col justify-center align-middle">
                      <span>{{ region.title }}</span>
                      <span class="text-sm text-neutral-400">
                        {{ region.value }}
                      </span>
                    </div>
                  </div>
                </div>
                <div class="flex flex-row items-center">
                  <label for="aws-region" class="basis-0 grow">
                    Or select any region
                  </label>
                  <select
                    id="aws-region"
                    v-model="awsRegion"
                    :class="
                      clsx(
                        'h-lg basis-0 grow p-xs text-sm border font-mono cursor-pointer',
                        'focus:outline-none focus:ring-0 focus:z-10',
                        'text-shade-0 bg-black border-neutral-600 focus:border-action-300',
                      )
                    "
                  >
                    <option v-for="region in awsRegions" :key="region.value">
                      {{ region.value }}
                    </option>
                  </select>
                </div>
              </div>
            </template>
            <template #footer>
              <NewButton
                :label="initializeApiError ? 'Retry' : 'Next'"
                tone="action"
                :disabled="!formHasRequiredValues"
                @click="
                  aiTutorialAreaRef?.open();
                  submitOnboardRequest();
                "
              />
            </template>
          </OnboardingCollapsingArea>
          <!-- Step 2: Agent Tutorial + token -->
          <OnboardingCollapsingArea
            ref="aiTutorialAreaRef"
            :blockOpening="!submittedOnboardRequest"
            @opened="userInputAreaRef?.close()"
          >
            <template #header>
              <div class="flex flex-row items-center justify-between">
                <span :class="setupAiDone && 'text-success-200'">
                  2. Setup your AI Agent
                </span>
                <Icon
                  :name="'check-circle'"
                  :class="clsx('text-success-400', !setupAiDone && 'hidden')"
                />
              </div>
            </template>
            <template #body>
              <div class="flex flex-col gap-xs">
                <span>Clone the AI Agent</span>
                <CopyableTextBlock
                  text="git clone https://github.com/systeminit/si-ai-agent.git"
                />
              </div>
              <div class="flex flex-col gap-xs">
                <span>Run the setup script</span>
                <CopyableTextBlock text="./setup.sh" />
              </div>
              <div class="flex flex-col gap-xs">
                <span>
                  Copy this API token to use as part of the AI Agent setup
                </span>
                <ErrorMessage
                  class="rounded-md text-md px-xs py-xs bg-action-900 my-xs"
                  icon="alert-circle"
                  tone="action"
                  variant="block"
                >
                  We're only showing you the value of this token once. Please,
                  store it somewhere safe.
                </ErrorMessage>
                <CopyableTextBlock
                  :text="apiToken"
                  expandable
                  @copied="trackEvent('onboarding_ai_token_copied')"
                />
                <ErrorMessage
                  v-if="!isSkippable && !hasUsedAiAgent"
                  class="rounded-md text-md px-xs py-xs bg-newhotness-warningdark my-xs"
                  icon="loader"
                  tone="warning"
                  variant="block"
                >
                  Please run the AI agent before continuing.
                </ErrorMessage>
              </div>
            </template>
            <template #footer>
              <NewButton
                label="Previous"
                tone="neutral"
                @click="userInputAreaRef?.open()"
              />
              <NewButton
                label="Done"
                tone="action"
                :disabled="
                  !initializeRequestSentAndSuccessful ||
                  (!isSkippable && !hasUsedAiAgent)
                "
                @click="handleDoneClick()"
              />
            </template>
          </OnboardingCollapsingArea>
        </div>
      </Transition>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, onMounted, reactive, ref, watch } from "vue";
import clsx from "clsx";
import { sleep } from "@si/ts-lib/src/async-sleep";
import { ErrorMessage, Icon, NewButton } from "@si/vue-lib/design-system";
import SiLogo from "@si/vue-lib/brand-assets/si-logo-symbol.svg?component";
import * as _ from "lodash-es";
import OnboardingCollapsingArea from "@/newhotness/OnboardingCollapsingArea.vue";
import ScrollingOutputLine from "@/newhotness/ScrollingOutputLine.vue";
import { encryptMessage } from "@/utils/messageEncryption";
import { componentTypes, routes, useApi } from "@/newhotness/api_composables";
import { useContext } from "@/newhotness/logic_composables/context";
import CopyableTextBlock from "@/newhotness/CopyableTextBlock.vue";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { useAuthStore } from "@/store/auth.store";
import { trackEvent } from "@/utils/tracking";

const authStore = useAuthStore();
const featureFlagsStore = useFeatureFlagsStore();

const isSkippable = computed(
  () => !featureFlagsStore.INITIALIZER_ONBOARD_FORCE_AGENT,
);
const hasUsedAiAgent = computed(
  () => authStore.userWorkspaceFlags.executedAgent ?? false,
);

const ctx = useContext();

const userInputAreaRef = ref<InstanceType<typeof OnboardingCollapsingArea>>();
const aiTutorialAreaRef = ref<InstanceType<typeof OnboardingCollapsingArea>>();

const sentences = [
  {
    sentence: "Welcome to your System Initiative Workspace.",
  },
  {
    sentence: "Follow these 2 steps to get started:",
  },
];

const visibleItemsCount = ref<number>(0);
const visibleSentences = computed(() =>
  sentences.slice(0, visibleItemsCount.value),
);

/// STARTUP LOGIC
// Delay so we complete the fade in before starting to show log lines
const TRANSITION_DELAY_MS = 200;

const kickOffTerminalLogs = async () => {
  await sleep(TRANSITION_DELAY_MS);
  visibleItemsCount.value = 1;
};

onMounted(kickOffTerminalLogs);

// Every time we start showing a sentence, enqueue showing the next sentence after a delay
const LOADING_STEP_DELAY_MS = [1200, 1000];

watch([visibleItemsCount], async () => {
  if (visibleItemsCount.value > sentences.length) return;

  const delay = LOADING_STEP_DELAY_MS[visibleItemsCount.value - 1];

  if (delay === undefined) {
    // eslint-disable-next-line no-console
    console.error(
      `No delay found for onboarding step ${visibleItemsCount.value} of ${sentences.length}`,
    );
    return;
  }

  await sleep(delay);

  visibleItemsCount.value += 1;
});

/// FORM LOGIC
const showFormItems = computed(() => {
  return visibleItemsCount.value > sentences.length;
});

// CREDENTIAL
const credentialName = ref("My AWS Credential");

const secretFormFields = reactive({
  "AWS access key ID": {
    ref: "",
    type: "password",
    required: true,
  },
  "AWS secret access key": {
    ref: "",
    type: "password",
    required: true,
  },
  "AWS session token": {
    ref: "",
    type: "password",
    required: false,
  },
});

const formHasRequiredValues = computed(
  () =>
    !Object.values(secretFormFields).some((f) => f.required && f.ref === "") &&
    credentialName.value !== "",
);

type SecretFormFields = typeof secretFormFields;
type SecretFormField = SecretFormFields[keyof SecretFormFields];

/// Match env var block to form fields
const tryMatchOnPaste = (ev: ClipboardEvent) => {
  const text = ev.clipboardData?.getData("text/plain");

  if (!text) return;

  const valuesFromInput = text.split("\n").reduce((acc, e) => {
    const [_, key, value] = e.match(/export\s+(.*)="(.*)"/) ?? [];
    if (!key || !value) return acc;
    acc[key] = value;
    return acc;
  }, {} as Record<string, string>);

  let matchedAValue = false;
  // Loop through form fields and try to match the value keys to the titles
  for (const [formKey, formValue] of Object.entries(secretFormFields)) {
    const formattedKey = formKey.replaceAll(" ", "_").toUpperCase();
    const matchedField = valuesFromInput[formattedKey];

    if (!matchedField) continue;
    matchedAValue = true;

    formValue.ref = matchedField;
  }

  // If we didn't match a value, just proceed with the paste
  if (!matchedAValue) return;

  ev.preventDefault();
};

const toggleVisibility = (field: SecretFormField) => {
  field.type = field.type === "password" ? "text" : "password";
};

// REGION
const awsRegion = ref("us-east-1");
const awsRegions = [
  { title: "US East (N. Virginia)", value: "us-east-1", onPicker: true },
  { title: "US West (Oregon)", value: "us-west-2", onPicker: true },
  { title: "US West (N. California)", value: "us-west-1", onPicker: true },
  { title: "Europe (Ireland)", value: "eu-west-1" },
  { title: "Europe (Frankfurt)", value: "eu-central-1" },
  { title: "Asia Pacific (Singapore)", value: "ap-southeast-1" },
  { title: "Asia Pacific (Tokyo)", value: "ap-northeast-1" },
  { title: "Asia Pacific (Sydney)", value: "ap-southeast-2" },
  { title: "US East (Ohio)", value: "us-east-2" },
  { title: "Europe (London)", value: "eu-west-2" },
  { title: "Africa (Cape Town)", value: "af-south-1" },
  { title: "Asia Pacific (Hong Kong)", value: "ap-east-1" },
  { title: "Asia Pacific (Taipei)", value: "ap-east-2" },
  { title: "Asia Pacific (Jakarta)", value: "ap-southeast-3" },
  { title: "Asia Pacific (Melbourne)", value: "ap-southeast-4" },
  { title: "Asia Pacific (Malaysia)", value: "ap-southeast-5" },
  { title: "Asia Pacific (Thailand)", value: "ap-southeast-7" },
  { title: "Asia Pacific (Mumbai)", value: "ap-south-1" },
  { title: "Asia Pacific (Hyderabad)", value: "ap-south-2" },
  { title: "Asia Pacific (Seoul)", value: "ap-northeast-2" },
  { title: "Asia Pacific (Osaka)", value: "ap-northeast-3" },
  { title: "Canada (Central)", value: "ca-central-1" },
  { title: "Canada West (Calgary)", value: "ca-west-1" },
  { title: "Europe (Zurich)", value: "eu-central-2" },
  { title: "Europe (Paris)", value: "eu-west-3" },
  { title: "Europe (Milan)", value: "eu-south-1" },
  { title: "Europe (Spain)", value: "eu-south-2" },
  { title: "Europe (Stockholm)", value: "eu-north-1" },
  { title: "Israel (Tel Aviv)", value: "il-central-1" },
  { title: "Middle East (Bahrain)", value: "me-south-1" },
  { title: "Middle East (UAE)", value: "me-central-1" },
  { title: "Mexico (Central)", value: "mx-central-1" },
  { title: "South America (Sao Paulo)", value: "sa-east-1" },
  { title: "AWS GovCloud (US-East)", value: "us-gov-east-1" },
  { title: "AWS GovCloud (US-West)", value: "us-gov-west-1" },
];
const pickerRegions = awsRegions.filter((r) => r.onPicker);

const initializeApi = useApi();
const initializeApiError = ref<string | null>(null);
const submittedOnboardRequest = ref(false);
const keyApi = useApi();

const initializeRequestSentAndSuccessful = computed(() => {
  return (
    submittedOnboardRequest.value &&
    !initializeApi.inFlight.value &&
    !initializeApiError.value
  );
});

const submitOnboardRequest = async () => {
  // Encrypt secret
  const callApi = keyApi.endpoint<componentTypes.PublicKey>(
    routes.GetPublicKey,
    { id: "00000000000000000000000000" }, // TODO Remove component id from this endpoint's path, it's not needed
  );
  const resp = await callApi.get();
  const publicKey = resp.data;

  // Format cred values for encryption
  const credValue = (
    [
      ["SessionToken", secretFormFields["AWS session token"].ref],
      ["AccessKeyId", secretFormFields["AWS access key ID"].ref],
      ["SecretAccessKey", secretFormFields["AWS secret access key"].ref],
    ].filter(([_, value]) => value !== "") as [string, string][]
  ) // Remove empty values
    .reduce<{ [key: string]: string }>((acc, [key, value]) => {
      // make the pairs array into an object
      acc[key] = value;
      return acc;
    }, {});

  const crypted = await encryptMessage(credValue, publicKey);

  submittedOnboardRequest.value = true;
  const call = initializeApi.endpoint(routes.ChangeSetInitializeAndApply);
  const { errorMessage } = await call.post({
    awsRegion: awsRegion.value,
    credential: {
      name: credentialName.value,
      crypted,
      keyPairPk: publicKey.pk,
      version: componentTypes.SecretVersion.V1,
      algorithm: componentTypes.SecretAlgorithm.Sealedbox,
    },
  });

  if (errorMessage) {
    initializeApiError.value = errorMessage;
  }
};

// AI CONFIGURATION
const setupAiDone = ref(false);
const generateTokenApi = useApi();
const apiToken = ref();
const generateToken = async () => {
  const workspacePk = ctx.workspacePk.value;
  const userPk = ctx.user?.pk;

  const apiTokenSessionStorageKey = `si-api-token-${workspacePk}-${userPk}`;

  const storedToken = sessionStorage.getItem(apiTokenSessionStorageKey);
  if (storedToken) {
    apiToken.value = storedToken;
    return;
  }

  const callApi = generateTokenApi.endpoint<{ token: string }>(
    routes.GenerateApiToken,
  );
  const {
    req: { data },
  } = await callApi.post({
    name: "Onboarding Key",
    expiration: "1y",
  });

  const token = data.token;

  if (!token) {
    // TODO deal with errors on API Token generation
    // eslint-disable-next-line no-console
    console.error("No token generated");
    return;
  }

  sessionStorage.setItem(apiTokenSessionStorageKey, token);
  apiToken.value = token;
};

onMounted(generateToken);

const dismissOnboardingApi = useApi();
const closeOnboarding = async (fast = false) => {
  const userPk = ctx.user?.pk;
  if (!userPk) return;

  const call = dismissOnboardingApi.endpoint(routes.DismissOnboarding, {
    userPk,
  });

  if (fast) {
    call.post({});
  } else {
    await call.post({});
  }

  emit("completed");
};

const handleDoneClick = () => {
  aiTutorialAreaRef.value?.close();
  setupAiDone.value = true;
};

watch(setupAiDone, closeOnboarding);

const emit = defineEmits<{
  (e: "completed"): void;
}>();
</script>
