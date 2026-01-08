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
            <!-- this text only shows the first time / user is in onboarding -->
            <template v-if="isOnboarding">
              <RichText>
                <h2>Welcome To System Initiative!</h2>
                <p>Please enter your profile information.</p>
              </RichText>
            </template>
            <!-- this is the default text -->
            <template v-else>
              <RichText>
                <h2>Update Your Profile</h2>
                <p>Use this page to update your profile info.</p>
              </RichText>
            </template>
          </Stack>
        </div>

        <form class="grow my-md px-md">
          <Stack>
            <ErrorMessage :requestStatus="updateUserReqStatus" />
            <VormInput
              v-if="draftUser.pictureUrl || storeUser?.pictureUrl"
              label="Profile Image"
              type="container"
            >
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
                :maxLength="MAX_LENGTH_STANDARD"
                :regex="NAME_REGEX"
                regexMessage="First name contains invalid characters or URLs"
              />
              <VormInput
                v-model="draftUser.lastName"
                label="Last Name"
                autocomplete="last-name"
                placeholder="Your last name"
                :maxLength="MAX_LENGTH_STANDARD"
                :regex="NAME_REGEX"
                regexMessage="Last name contains invalid characters or URLs"
              />
            </Tiles>
            <VormInput
              v-model="draftUser.nickname"
              label="Nickname"
              autocomplete="username"
              required
              placeholder="This name will be shown in the application"
              :maxLength="MAX_LENGTH_STANDARD"
              :regex="NICKNAME_REGEX"
              regexMessage="Nickname contains invalid characters or URLs"
            />
            <VormInput
              v-model="draftUser.email"
              label="Email"
              type="email"
              autocomplete="email"
              required
              disabled
              placeholder="ex: yourname@somewhere.com"
            />
            <VormInput
              v-model="draftUser.discordUsername"
              label="Discord Username"
              name="discord_username"
              placeholder="ex: eggscellent OR eggscellent#1234"
              :maxLength="MAX_LENGTH_STANDARD"
              :regex="DISCORD_TAG_REGEX"
              regexMessage="Invalid discord tag"
              class="pb-xs"
            >
              <template #instructions>
                <div class="text-neutral-700 dark:text-neutral-200 italic">
                  Entering your username will help us to give you technical
                  support
                  <a href="" class="underline text-action-500">on our Discord</a
                  >.
                </div>
              </template>
            </VormInput>
            <VormInput
              v-model="draftUser.githubUsername"
              label="Github Username"
              name="github_username"
              placeholder="ex: devopsdude42"
              :maxLength="MAX_LENGTH_STANDARD"
              :regex="GITHUB_USERNAME_REGEX"
              regexMessage="Invalid github username"
            />

            <VButton
              iconRight="chevron--right"
              :disabled="validationState.isError || failedEmailVerify"
              :requestStatus="updateUserReqStatus"
              loadingText="Saving your profile..."
              successText="Updated your profile!"
              tone="action"
              variant="solid"
              :label="saveButtonLabel"
              @click="saveHandler"
            />
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
import { computed, onBeforeMount, ref, watch } from "vue";
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
import { useWorkspacesStore } from "@/store/workspaces.store";
import {
  NAME_REGEX,
  NICKNAME_REGEX,
  GITHUB_USERNAME_REGEX,
  DISCORD_TAG_REGEX,
  MAX_LENGTH_STANDARD,
} from "@/lib/validations";

const { validationState, validationMethods } = useValidatedInputGroup();
const authStore = useAuthStore();
const workspacesStore = useWorkspacesStore();
// Reactively load the workspaces so that the workspace redirect happens for first time users
workspacesStore.refreshWorkspaces();
const router = useRouter();

const loadUserReqStatus = authStore.getRequestStatus("LOAD_USER");
const checkAuthReqStatus = authStore.getRequestStatus("CHECK_AUTH");
const updateUserReqStatus = authStore.getRequestStatus("UPDATE_USER");

const storeUser = computed(() => authStore.user);
const draftUser = ref<User>();
const isOnboarding = ref<boolean>();

const refreshAuth0ReqStatus = authStore.getRequestStatus(
  "REFRESH_AUTH0_PROFILE",
);

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

watch([refreshAuth0ReqStatus, storeUser], () => {
  if (storeUser.value?.emailVerified) {
    failedEmailVerify.value = false;
    saveButtonLabel.value = "Save";
  }
});

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

const saveHandler = async () => {
  if (validationMethods.hasError()) return;

  // Users whose email has not been verified should not be able to continue past here
  if (!authStore.user?.emailVerified) {
    await verifyEmail();
    if (!authStore.user?.emailVerified) {
      youMustVerifyYourEmailAddress();
      return;
    }
  }

  // Okay now we know that their email has been verified
  // if this is first time, do this stuff
  const updateReq = await authStore.UPDATE_USER(draftUser.value!);
  if (updateReq.result.success && isOnboarding.value) {
    if (
      storeUser.value &&
      storeUser.value.emailVerified &&
      !storeUser.value.auth0Id.startsWith("auth0")
    ) {
      // We only want to send this event when a user has signed up and
      // we captured a verified email for them. We will only ever have a
      // user with an auth0Id of auth0 if it's not using SSO. So we will
      // force that user through a manual verification and capture the user
      // at that stage
      // This means we won't ever be sending badly formed data to our CRM
      // or billing
      // This is also the place we would trigger the creation of a Billing user
      tracker.trackEvent("initial_profile_set", {
        email: draftUser.value?.email,
        githubUsername: draftUser.value?.githubUsername,
        discordUsername: draftUser.value?.discordUsername,
        firstName: draftUser.value?.firstName,
        lastName: draftUser.value?.lastName,
      });

      await authStore.BILLING_INTEGRATION();
    }

    const completeProfileReq = await authStore.COMPLETE_PROFILE({});

    if (completeProfileReq.result.success) {
      // eslint-disable-next-line @typescript-eslint/no-floating-promises
      router.push({ name: "workspaces" });
    }
  } else {
    // eslint-disable-next-line @typescript-eslint/no-floating-promises
    router.push({ name: "workspaces" });
  }
};
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

const saveButtonLabel = ref("Save");
const failedEmailVerify = ref(false);
const youMustVerifyYourEmailAddress = () => {
  saveButtonLabel.value = "You must verify your email address to continue";
  failedEmailVerify.value = true;
};

const verifyEmail = async () => {
  // if this is first time, we will take them off profile page after save
  const verificationReq = await authStore.REFRESH_AUTH0_PROFILE();
  if (verificationReq.result.success) {
    failedEmailVerify.value = false;
    if (storeUser.value && storeUser.value.emailVerified) {
      // We only want to send this event when a user has signed up and
      // we captured a verified email for them
      // This means we won't ever be sending badly formed data to our CRM
      // or billing
      // This is also the place we would trigger the creation of a Billing user
      tracker.trackEvent("user_email_manually_verified", {
        email: storeUser.value?.email,
        githubUsername: storeUser.value?.githubUsername,
        discordUsername: storeUser.value?.discordUsername,
        firstName: storeUser.value?.firstName,
        lastName: storeUser.value?.lastName,
      });

      await authStore.BILLING_INTEGRATION();
    }
  }
};
</script>
