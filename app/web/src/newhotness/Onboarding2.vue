<template>
  <div
    data-testid="lobby"
    :class="
      clsx(
        'absolute w-screen h-screen z-[1000] scrollable',
        themeClasses('bg-white text-black', 'bg-neutral-900 text-white,'),
      )
    "
  >
    <div class="flex flex-col items-center w-full h-full min-w-[900px]">
      <!--  HEADER  -->
      <div
        class="flex flex-row flex-none items-center justify-between w-full px-sm py-xs"
      >
        <SiLogo class="block h-md w-md flex-none" />
        <NewButton
          aria-label="schedule-demo-header"
          class="group/schedule font-normal"
          tone="empty"
          :href="scheduleLink"
          target="_blank"
        >
          <span
            :class="
              clsx(
                themeClasses(
                  'text-neutral-600 group-hover/schedule:text-black',
                  'text-neutral-400 group-hover/schedule:text-white',
                ),
              )
            "
          >
            Not ready? Schedule a meeting and we'll demo it for you.
          </span>
        </NewButton>
      </div>
      <!--  Form + Text in the middle  -->
      <div
        class="flex flex-row items-center grow w-full max-w-[1500px] px-lg gap-lg"
      >
        <div class="flex-1 basis-1/2 min-w-0">
          <!-- Step 1: User Input -->
          <OnboardingStepBlock v-if="currentStep === OnboardingStep.INITIALIZE">
            <template #header>
              <div class="flex flex-row items-center justify-between">
                <span
                  :class="
                    initializeRequestSentAndSuccessful && 'text-success-200'
                  "
                >
                  Enter an AWS Credential
                </span>
                <div>1/3</div>
              </div>
            </template>
            <template #body>
              <!-- Credential -->
              <div
                :class="
                  clsx(
                    'flex flex-col border p-sm gap-sm',
                    themeClasses('border-neutral-400', 'border-neutral-600'),
                  )
                "
              >
                <div class="flex flex-row justify-between items-center">
                  <label for="aws-credential-name" class="basis-0 grow">
                    Name your credential*
                  </label>
                  <input
                    id="aws-credential-name"
                    v-model="credentialName"
                    data-lpignore="true"
                    data-1p-ignore
                    data-bwignore
                    data-form-type="other"
                    :class="
                      clsx(
                        'h-lg p-xs text-sm border font-mono cursor-text basis-0 grow',
                        'focus:outline-none focus:ring-0 focus:z-10',
                        themeClasses(
                          'text-black bg-white border-neutral-400 focus:border-action-500',
                          'text-white bg-black border-neutral-600 focus:border-action-300',
                        ),
                      )
                    "
                  />
                </div>
                <!-- Secret Values -->
                <ErrorMessage
                  :class="
                    clsx(
                      'rounded-md text-md px-xs py-xs mt-xs',
                      themeClasses('bg-action-200', 'bg-action-900'),
                    )
                  "
                  tone="action"
                  variant="block"
                  noIcon
                >
                  <div class="flex flex-row items-center justify-between">
                    <div>
                      Paste the full Bash environment block into the first field
                      — we’ll auto-fill the rest.
                    </div>
                    <Icon
                      v-tooltip="
                        someFieldsVisible
                          ? 'Hide All Values'
                          : 'Show All Values'
                      "
                      :name="someFieldsVisible ? 'hide' : 'eye'"
                      size="xs"
                      class="cursor-pointer z-20"
                      @click="toggleAll"
                    />
                  </div>
                </ErrorMessage>

                <div class="flex flex-col">
                  <div
                    v-for="(field, title) in secretFormFields"
                    :key="title"
                    :class="'flex flex-row justify-between items-center text-sm mb-[-1px]'"
                  >
                    <label
                      class="basis-0 grow flex flex-row items-center gap-2xs"
                    >
                      {{ title }}<span v-if="field.required">*</span>
                    </label>
                    <div class="flex flex-row basis-0 grow relative">
                      <input
                        v-model="field.ref"
                        :type="field.type"
                        :class="
                          clsx(
                            'h-lg p-xs pr-7 text-sm border font-mono cursor-text grow',
                            'focus:outline-none focus:ring-0 focus:z-10',
                            themeClasses(
                              'text-black bg-white border-neutral-400 focus:border-action-500',
                              'text-white bg-black border-neutral-600 focus:border-action-300',
                            ),
                          )
                        "
                        :placeholder="
                          field.type === 'password'
                            ? '*****'
                            : 'Value will be visible'
                        "
                        data-lpignore="true"
                        data-1p-ignore
                        data-bwignore
                        data-form-type="other"
                        @paste="(ev: ClipboardEvent) => tryMatchOnPaste(ev)"
                      />
                      <Icon
                        v-tooltip="
                          field.type === 'password'
                            ? 'Show Value'
                            : 'Hide Value'
                        "
                        :name="field.type === 'password' ? 'eye' : 'hide'"
                        size="xs"
                        class="absolute right-xs top-[10px] cursor-pointer z-20"
                        @click="toggleVisibility(field)"
                      />
                    </div>
                  </div>
                </div>
              </div>
              <!-- Region -->
              <div
                :class="
                  clsx(
                    'border flex flex-col p-sm gap-sm',
                    themeClasses('border-neutral-400', 'border-neutral-600'),
                  )
                "
              >
                <div>Pick a region*</div>
                <div
                  class="flex desktop:flex-row flex-col desktop:items-center items-stretch desktop:gap-sm gap-xs"
                >
                  <div
                    v-for="region in pickerRegions"
                    :key="region.value"
                    :class="
                      clsx(
                        'flex flex-row grow items-center border desktop:p-sm p-xs gap-sm cursor-pointer',
                        themeClasses(
                          'hover:border-neutral-700',
                          'hover:border-neutral-300',
                        ),
                        region.value === awsRegion
                          ? themeClasses(
                              'border-neutral-700',
                              'border-neutral-300',
                            )
                          : themeClasses(
                              'border-neutral-300',
                              'border-neutral-600',
                            ),
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
                      <span class="text-sm">{{ region.title }}</span>
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
                        themeClasses(
                          'text-black bg-white border-neutral-400 focus:border-action-500',
                          'text-white bg-black border-neutral-600 focus:border-action-300',
                        ),
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
            <template #footerRight>
              <NewButton
                :label="initializeApiError ? 'Retry' : 'Next'"
                :tooltip="
                  !formHasRequiredValues
                    ? 'You must enter your credential to continue'
                    : undefined
                "
                tone="action"
                :disabled="!formHasRequiredValues"
                :loading="submitOnboardingInProgress"
                loadingText="Saving"
                @click="submitOnboardRequest"
              />
            </template>
          </OnboardingStepBlock>
          <!-- Step 2: Agent Tutorial + token -->
          <OnboardingStepBlock
            v-else-if="currentStep === OnboardingStep.SETUP_AI"
          >
            <template #header>
              <div class="flex flex-row items-center justify-between">
                <span>Connect your AI Agent</span>
                <div>2/3</div>
              </div>
            </template>
            <template #body>
              <div class="flex flex-col gap-xs">
                <span>Clone the AI Agent repository</span>
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
                  :class="
                    clsx(
                      'rounded-md text-md px-xs py-xs my-xs',
                      themeClasses('bg-action-200', 'bg-action-900'),
                    )
                  "
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
                  @copied="onCopyAgentToken"
                />
                <ErrorMessage
                  v-if="agentTokenCopied"
                  tone="neutral"
                  icon="loader"
                >
                  Waiting for the AI agent to start. You’ll be able to proceed
                  as soon as setup is finished.
                </ErrorMessage>
                <ErrorMessage v-else tone="neutral" icon="sparkles">
                  Click the API token to copy it.
                </ErrorMessage>
              </div>
            </template>
            <template #footerRight>
              <!-- <NewButton
                label="Previous"
                tone="neutral"
                @click="decrementOnboardingStep"
              /> -->
              <NewButton
                label="Next"
                tone="action"
                :icon="
                  stepTwoNextDisabled && agentTokenCopied ? 'loader' : undefined
                "
                :disabled="stepTwoNextDisabled"
                :tooltip="
                  stepTwoNextDisabled
                    ? 'You must setup your AI agent to continue'
                    : undefined
                "
                @click="incrementOnboardingStep"
              />
            </template>
          </OnboardingStepBlock>
          <!-- Step 3: Try your first prompt -->
          <OnboardingStepBlock v-else>
            <template #header>
              <div class="flex flex-row items-center justify-between">
                <span>Try your first prompt</span>
                <div
                  :class="themeClasses('text-success-600', 'text-success-300')"
                >
                  3/3
                </div>
              </div>
            </template>
            <template #body>
              <div>
                Now that you have connected your AI agent, you can use any of
                these prompts, or one of your own, to see System Initiative in
                action:
              </div>
              <CopyableTextBlock
                v-for="(prompt, index) in prompts"
                :key="index"
                :text="prompt"
                prompt
              />
            </template>
            <template #footerRight>
              <NewButton
                label="Get Started"
                tone="action"
                @click="closeOnboarding"
              />
            </template>
          </OnboardingStepBlock>
        </div>
        <div
          class="flex-1 basis-1/4 min-w-0 flex flex-col gap-lg ml-xl font-medium"
        >
          <div class="text-xl">
            <span
              :class="
                clsx(themeClasses('text-neutral-600', 'text-neutral-400'))
              "
              >Your workspace</span
            >
            will be ready with these simple steps
          </div>
          <div class="flex flex-col">
            <div
              v-for="(step, index) in steps"
              :key="index"
              class="grid steps gap-0"
            >
              <div
                :class="
                  clsx(
                    'number self-center flex flex-row items-center justify-center rounded-full w-8 h-8',
                    finishedStep(index)
                      ? themeClasses('bg-neutral-300', 'bg-neutral-600')
                      : themeClasses(
                          'bg-success-300',
                          'bg-success-900 text-success-200',
                        ),
                  )
                "
              >
                <div v-if="finishedStep(index)" class="w-full text-center">
                  {{ index + 1 }}
                </div>
                <Icon v-else name="check" size="sm" />
              </div>
              <div class="numberline w-full h-full flex flex-col items-center">
                <div
                  v-if="index < steps.length - 1"
                  :class="
                    clsx(
                      'border-r h-full',
                      finishedStep(index)
                        ? themeClasses(
                            'border-neutral-300',
                            'border-neutral-600',
                          )
                        : themeClasses(
                            'border-success-300',
                            'border-success-900',
                          ),
                    )
                  "
                />
              </div>
              <TruncateWithTooltip
                class="primary self-center pl-sm max-h-8 text-sm leading-none pb-3xs mt-3xs"
                :lineClamp="2"
              >
                {{ step.primaryText }}
              </TruncateWithTooltip>
              <div
                :class="
                  clsx(
                    'secondary pl-sm pt-xs pb-lg text-sm',
                    themeClasses('text-neutral-600', 'text-neutral-400'),
                  )
                "
              >
                {{ step.secondaryText }}
              </div>
            </div>
          </div>
        </div>
      </div>
      <!--  Bottom Links  -->
      <div
        class="flex flex-row flex-none w-full items-center justify-start px-lg py-sm gap-sm"
      >
        <template v-if="currentStep === OnboardingStep.INITIALIZE">
          <div
            :class="
              clsx(
                'hover:underline cursor-pointer',
                themeClasses(
                  'text-neutral-700 hover:text-black',
                  'text-neutral-300 hover:text-white',
                ),
              )
            "
            @click="() => credNecessaryModal?.open()"
          >
            Why is a credential necessary?
          </div>
          <p class="text-neutral-600">|</p>
          <div
            :class="
              clsx(
                'hover:underline cursor-pointer',
                themeClasses(
                  'text-neutral-700 hover:text-black',
                  'text-neutral-300 hover:text-white',
                ),
              )
            "
            @click="() => noCredModal?.open()"
          >
            I don't use AWS, what do I do?
          </div>
        </template>
        <template v-else-if="currentStep === OnboardingStep.SETUP_AI">
          <!-- TODO(Wendy) - any footer links we want here? -->
        </template>
        <template v-else>
          <!-- TODO(Wendy) - any footer links we want here? -->
        </template>
      </div>
    </div>

    <Modal
      ref="credNecessaryModal"
      title="Why is a credential necessary?"
      onboardingModal
      size="xl"
      type="done"
    >
      <span>
        System Initiative is an AI Native Infrastructure Automation Platform
        that provides a real-time digital twin of your infrastructure. To do
        that, we need access to your AWS account to securely manage your
        infrastructure. Nothing changes without your approval.
      </span>
      <br />
      <span>
        Your secrets are safe. They are encrypted in transit and at rest using
        industry best-practice cryptography.
      </span>
    </Modal>
    <Modal
      ref="noCredModal"
      title="I don't use AWS, what do I do?"
      onboardingModal
      size="xl"
      type="done"
    >
      <div>
        <a
          :href="mailToLink"
          target="_blank"
          :class="
            clsx(
              'underline',
              themeClasses('hover:text-action-500', 'hover:text-action-300'),
            )
          "
          >Reach out</a
        >
        <span>
          and let us know what providers you work with that we should add
          support for.
        </span>
      </div>
    </Modal>
  </div>
</template>

<script lang="ts" setup>
import { computed, onMounted, reactive, ref } from "vue";
import clsx from "clsx";
import {
  ErrorMessage,
  Icon,
  Modal,
  NewButton,
  themeClasses,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import SiLogo from "@si/vue-lib/brand-assets/si-logo-symbol.svg?component";
import * as _ from "lodash-es";
import { encryptMessage } from "@/utils/messageEncryption";
import { componentTypes, routes, useApi } from "@/newhotness/api_composables";
import { useContext } from "@/newhotness/logic_composables/context";
import { trackEvent } from "@/utils/tracking";
import OnboardingStepBlock from "@/newhotness/OnboardingStepBlock.vue";
import { useAuthStore } from "@/store/auth.store";
import CopyableTextBlock from "./CopyableTextBlock.vue";
import { prompts } from "./WelcomeBanner.vue";

const scheduleLink =
  "https://calendly.com/d/cns7-v2b-jkz/system-initiative-demo";

const mailToLink = computed(() => {
  const to = "help@systeminit.com";
  const subject = encodeURIComponent("Provider Support");
  const body = encodeURIComponent(
    "Please tell us what providers you are interested in support for:",
  );
  return `mailto:${to}?subject=${subject}&body=${body}`;
});

const steps = [
  {
    primaryText: "Enter an AWS credential and select a region",
    secondaryText:
      "With a credential, you'll see System Initiative at full power. No mock data, you’ll get the real thing.",
  },
  {
    primaryText: "Connect our AI Agent",
    secondaryText:
      "Use our AI Agent to solve your problems quickly and easily. Automate your infrastructure, explain how it works, propose changes, optimize costs, perform security reviews, etc.",
  },
  {
    primaryText: "Try your first prompt",
    secondaryText:
      "Use the sample prompts to perform automation tasks in minutes.",
  },
];

const ctx = useContext();

const authStore = useAuthStore();

const hasUsedAiAgent = computed(
  () => authStore.userWorkspaceFlags.executedAgent ?? false,
);

/// STARTUP LOGIC
enum OnboardingStep {
  INITIALIZE,
  SETUP_AI,
  DONE,
}
const currentStep = ref<OnboardingStep>(OnboardingStep.INITIALIZE);
const incrementOnboardingStep = () => {
  currentStep.value = Math.min(
    OnboardingStep.DONE,
    currentStep.value + 1,
  ) as OnboardingStep;
};
// const decrementOnboardingStep = () => {
//   currentStep.value = Math.max(
//     OnboardingStep.INITIALIZE,
//     currentStep.value - 1,
//   ) as OnboardingStep;
// };

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
const someFieldsVisible = computed(() =>
  Object.values(secretFormFields).some((field) => field.type !== "password"),
);
const toggleAll = () => {
  if (someFieldsVisible.value) {
    Object.values(secretFormFields).forEach((field) => {
      field.type = "password";
    });
  } else {
    Object.values(secretFormFields).forEach((field) => {
      field.type = "text";
    });
  }
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
// TODO Figure out where to put this
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

const submitOnboardingInProgress = ref(false);

const submitOnboardRequest = async () => {
  // Disable button
  submitOnboardingInProgress.value = true;

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

  submitOnboardingInProgress.value = false;

  if (errorMessage) {
    initializeApiError.value = errorMessage;
  } else {
    incrementOnboardingStep();
  }
};

// AI CONFIGURATION
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

const agentTokenCopied = ref(false);
const onCopyAgentToken = () => {
  agentTokenCopied.value = true;
  trackEvent("onboarding_ai_token_copied");
};

const finishedStep = (step: number) => currentStep.value < step + 1;

const stepTwoNextDisabled = computed(
  () => !initializeRequestSentAndSuccessful.value || !hasUsedAiAgent.value,
);

const credNecessaryModal = ref<InstanceType<typeof Modal>>();
const noCredModal = ref<InstanceType<typeof Modal>>();

const emit = defineEmits<{
  (e: "completed"): void;
}>();
</script>

<style lang="css" scoped>
.steps {
  grid-template-columns: 32px 1fr;
  grid-template-rows: auto auto;
  grid-template-areas:
    "number primary"
    "numberline secondary";
}

.number {
  grid-area: number;
}

.numberline {
  grid-area: numberline;
}

.primary {
  grid-area: primary;
}

.secondary {
  grid-area: secondary;
}
</style>
