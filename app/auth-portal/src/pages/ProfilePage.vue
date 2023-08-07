<!-- eslint-disable vue/no-v-html -->
<template>
  <div>
    <template v-if="loadUserReqStatus.isPending">
      <Icon name="loader" size="xl" />
    </template>
    <template v-else-if="loadUserReqStatus.isError">
      <ErrorMessage :requestStatus="loadUserReqStatus" />
    </template>
    <template v-else-if="draftUser">
      <div class="flex gap-xl">
        <div class="w-[35%] flex items-center pl-md">
          <Stack>
            <!-- this text only shows when filling out the step two questions -->
            <template
              v-if="isOnboarding && stepTwo && featureFlagsStore.OSS_RELEASE"
            >
              <RichText>
                <h2>Please Tell Us More About You</h2>
                <p>
                  These questions are completely optional but they will help us
                  to better understand who is using System Initiative.
                </p>
                <p>We won't share your info with anyone.</p>
              </RichText>
            </template>
            <!-- this text only shows the first time / user is in onboarding -->
            <template v-else-if="isOnboarding && featureFlagsStore.OSS_RELEASE">
              <RichText>
                <h2>Welcome To System Initiative!</h2>
                <p>Please enter your profile information.</p>
                <p>We won't share your info with anyone.</p>
              </RichText>
            </template>
            <!-- OLD TEXT this text only shows the first time a user is onboarding PRIOR TO OSS RELEASE -->
            <template
              v-else-if="isOnboarding && !featureFlagsStore.OSS_RELEASE"
            >
              <RichText>
                <h2>Welcome to the preview of System Initiative!</h2>
                <p>
                  In order to get you access to the software, we need to know a
                  little more about you: specifically, we need your GitHub
                  Username and your Discord ID. We will use your GitHub Username
                  to add you to our private repository, and your Discord ID to
                  ensure you have access to private channels to discuss System
                  Initiative with other folks in the preview.
                </p>
              </RichText>
            </template>
            <!-- this is the default text -->
            <template v-else>
              <RichText>
                <h2>Update Your Profile</h2>
                <p>Use this page to update your profile info.</p>
                <p>We won't share your info with anyone.</p>
              </RichText>
            </template>
          </Stack>
        </div>

        <form
          v-if="stepTwo && featureFlagsStore.OSS_RELEASE"
          class="grow my-md p-md"
        >
          <Stack spacing="lg">
            <Stack>
              <VormInput
                v-model="stepTwoData.company"
                label="Company"
                name="company"
                placeholder="Enter the name of your company here"
              />
              <VormInput
                v-model="stepTwoData.cloudProviders"
                label="Cloud Providers"
                prompt="Which of these cloud providers does your company use? Please select all that apply."
                name="cloud_providers"
                type="multi-checkbox"
                :options="cloudProviders"
              />
              <VormInput
                v-model="stepTwoData.devOpsTools"
                label="DevOps Tools"
                prompt="Which of these DevOps tools are you currently using? Please select all that apply."
                name="devops_tools"
                type="multi-checkbox"
                :options="devOpsTools"
              />
              <VormInput
                v-model="stepTwoData.openSource"
                label="Open Source Experience"
                prompt="Have you contributed to an open source project before?"
                name="open_source"
                type="radio"
                :options="[
                  { value: true, label: 'Yes' },
                  { value: false, label: 'No' },
                ]"
              />
              <div class="flex flex-row gap-sm">
                <VButton
                  class="flex-grow"
                  iconRight="chevron--right"
                  :disabled="validationState.isError"
                  :requestStatus="updateUserReqStatus"
                  tone="action"
                  variant="solid"
                  @click="tellUsMoreHandler"
                >
                  Submit
                </VButton>
                <VButton
                  class="flex-none"
                  iconRight="x"
                  :disabled="validationState.isError"
                  :requestStatus="updateUserReqStatus"
                  tone="destructive"
                  variant="ghost"
                  @click="completeProfile"
                >
                  Skip
                </VButton>
              </div>
            </Stack>
          </Stack>
        </form>
        <form v-else class="grow my-md p-md">
          <Stack spacing="lg">
            <Stack>
              <ErrorMessage :requestStatus="updateUserReqStatus" />
              <VormInput label="Profile Image" type="container">
                <div
                  v-if="draftUser.pictureUrl"
                  class="flex flex-row items-center gap-sm"
                >
                  <img
                    :src="draftUser.pictureUrl"
                    class="rounded-full w-xl h-xl"
                  />
                  <VButton
                    tone="destructive"
                    size="xs"
                    variant="ghost"
                    @click="clearPicture"
                    >Clear Picture
                  </VButton>
                </div>
                <div
                  v-else-if="storeUser?.pictureUrl"
                  class="h-xl items-center flex flex-row gap-sm"
                >
                  <div class="italic text-sm">No image set.</div>
                  <VButton
                    tone="action"
                    size="xs"
                    variant="ghost"
                    @click="restorePicture"
                    >Restore Picture
                  </VButton>
                </div>
              </VormInput>
              <Tiles columns="2" spacing="sm" columnsMobile="1">
                <VormInput
                  v-model="draftUser.firstName"
                  label="First Name"
                  autocomplete="given-name"
                  placeholder="Your first name"
                />
                <VormInput
                  v-model="draftUser.lastName"
                  label="Last Name"
                  autocomplete="last-name"
                  placeholder="Your last name"
                />
              </Tiles>
              <VormInput
                v-model="draftUser.nickname"
                label="Nickname"
                autocomplete="username"
                required
                placeholder="This name will be shown in the application"
              />
              <VormInput
                v-model="draftUser.email"
                label="Email"
                type="email"
                autocomplete="email"
                required
                placeholder="ex: yourname@somewhere.com"
              />
              <VormInput
                v-model="draftUser.githubUsername"
                label="Github Username"
                name="github_username"
                placeholder="ex: devopsdude42"
                :regex="GITHUB_USERNAME_REGEX"
                regexMessage="Invalid github username"
                :required="!featureFlagsStore.OSS_RELEASE"
              />
              <VormInput
                v-model="draftUser.discordUsername"
                label="Discord Username"
                name="discord_username"
                placeholder="ex: eggscellent OR eggscellent#1234"
                :regex="DISCORD_TAG_REGEX"
                regexMessage="Invalid discord tag"
                :required="!featureFlagsStore.OSS_RELEASE"
                class="pb-sm"
              >
                <template #instructions>
                  <div class="text-neutral-700 dark:text-neutral-200 italic">
                    Entering your Discord username will help us to better give
                    you technical support.
                  </div>
                </template>
              </VormInput>

              <VButton
                iconRight="chevron--right"
                :disabled="validationState.isError"
                :requestStatus="updateUserReqStatus"
                loadingText="Saving your profile..."
                successText="Updated your profile!"
                tone="action"
                variant="solid"
                @click="saveHandler"
              >
                Save
              </VButton>
            </Stack>
          </Stack>
        </form>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
/* eslint-disable @typescript-eslint/no-non-null-assertion */

import * as _ from "lodash-es";
import { useRouter } from "vue-router";
import { computed, onBeforeMount, reactive, ref, watch } from "vue";
import {
  ErrorMessage,
  Icon,
  Tiles,
  Stack,
  useValidatedInputGroup,
  VButton,
  VormInput,
  RichText,
} from "@si/vue-lib/design-system";
import { useHead } from "@vueuse/head";
import { useAuthStore, User } from "@/store/auth.store";
import { tracker } from "@/lib/posthog";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";

const featureFlagsStore = useFeatureFlagsStore();

const GITHUB_USERNAME_REGEX = /^[a-z\d](?:[a-z\d]|-(?=[a-z\d])){0,38}$/i;
const DISCORD_TAG_REGEX =
  /^(?!(discord|here|everyone))(((?!.*\.\.)(([\w.]{2,32})))|[^@#:]{2,32}#[\d]{4})$/i;

const { validationState, validationMethods } = useValidatedInputGroup();
const authStore = useAuthStore();
const router = useRouter();

const loadUserReqStatus = authStore.getRequestStatus("LOAD_USER");
const checkAuthReqStatus = authStore.getRequestStatus("CHECK_AUTH");
const updateUserReqStatus = authStore.getRequestStatus("UPDATE_USER");

const storeUser = computed(() => authStore.user);
const draftUser = ref<User>();
const isOnboarding = ref<boolean>();
const stepTwo = ref(false);
const stepTwoData = reactive({
  company: undefined,
  cloudProviders: [],
  devOpsTools: [],
  openSource: undefined,
});

const cloudProviders = [
  { value: "aws", label: "AWS" },
  { value: "azure", label: "Azure" },
  { value: "digital_ocean", label: "Digital Ocean" },
  { value: "gcp", label: "GCP" },
];

const devOpsTools = [
  { value: "cdk", label: "CDK" },
  { value: "docker", label: "Docker" },
  { value: "kubernetes", label: "Kubernetes" },
  { value: "pulumi", label: "Pulumi" },
  { value: "terraform", label: "Terraform" },
];

useHead({ title: "Profile" });

function resetDraftUser() {
  draftUser.value = _.cloneDeep(storeUser.value!);
}

watch(storeUser, resetDraftUser, { immediate: true });

function checkUserOnboarding() {
  if (storeUser.value && isOnboarding.value === undefined) {
    isOnboarding.value = !storeUser.value.onboardingDetails?.reviewedProfile;
  }
}

watch(storeUser, checkUserOnboarding, { immediate: true });

onBeforeMount(() => {
  // normally when landing on this page, we should probably make sure we have the latest profile info
  // but we already load user info with CHECK_AUTH so can skip if it was just loaded
  // (this will likely go away if we start fetching more profile info than what gets fetched while checking auth)
  if (+checkAuthReqStatus.value.lastSuccessAt! > +new Date() - 10000) {
    return;
  }

  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  authStore.LOAD_USER();
});

async function saveHandler() {
  if (validationMethods.hasError()) return;

  // if this is first time, we will take them off profile page after save
  const updateReq = await authStore.UPDATE_USER(draftUser.value!);
  if (updateReq.result.success && isOnboarding.value) {
    tracker.trackEvent("initial_profile_set", {
      email: draftUser.value?.email,
      githubUsername: draftUser.value?.githubUsername,
      discordUsername: draftUser.value?.discordUsername,
      firstName: draftUser.value?.firstName,
      lastName: draftUser.value?.lastName,
    });
    if (featureFlagsStore.OSS_RELEASE) {
      stepTwo.value = true;
    } else {
      await completeProfile();
    }
  }
}

async function tellUsMoreHandler() {
  if (validationMethods.hasError()) return;

  if (draftUser.value && !draftUser.value.onboardingDetails) {
    draftUser.value.onboardingDetails = {};
  }

  if (draftUser.value?.onboardingDetails) {
    if (stepTwoData.company) {
      draftUser.value.onboardingDetails.company = stepTwoData.company;
    }
    if (stepTwoData.cloudProviders.length > 0) {
      draftUser.value.onboardingDetails.cloudProviders =
        stepTwoData.cloudProviders;
    }
    if (stepTwoData.devOpsTools.length > 0) {
      draftUser.value.onboardingDetails.devOpsTools = stepTwoData.devOpsTools;
    }
    if (stepTwoData.openSource !== undefined) {
      draftUser.value.onboardingDetails.openSource = stepTwoData.openSource;
    }

    // TODO(Wendy) - Can't figure out why Prisma isn't updating the onboardingDetails properly :(
    const draftUserClone = _.cloneDeep(draftUser.value);
    const tellUsMoreReq = await authStore.UPDATE_USER(draftUser.value!);

    if (tellUsMoreReq.result.success) {
      tracker.trackEvent("tell_us_more_answers", {
        company: draftUserClone.onboardingDetails?.company || "",
        cloudProviders: draftUserClone.onboardingDetails?.cloudProviders || "",
        devOpsTools: draftUserClone.onboardingDetails?.devOpsTools || "",
        openSource: draftUserClone.onboardingDetails?.openSource || "",
      });

      await completeProfile();
    }
  }
}

async function completeProfile() {
  const completeProfileReq = await authStore.COMPLETE_PROFILE(draftUser.value!);
  if (completeProfileReq.result.success) {
    // eslint-disable-next-line @typescript-eslint/no-floating-promises
    router.push({ name: "login-success" });
  }
}

const clearPicture = () => {
  if (draftUser.value) {
    draftUser.value.pictureUrl = null;
  }
};
const restorePicture = () => {
  if (draftUser.value && storeUser.value) {
    draftUser.value.pictureUrl = storeUser.value.pictureUrl;
  }
};
</script>
