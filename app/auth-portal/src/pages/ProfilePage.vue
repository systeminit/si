<!-- eslint-disable vue/no-v-html -->
<template>
  <div>
    <template v-if="loadUserReqStatus.isPending">
      <Icon name="loader" size="xl" />
    </template>
    <template v-else-if="loadUserReqStatus.isError">
      <ErrorMessage :request-status="loadUserReqStatus" />
    </template>
    <template v-else-if="draftUser">
      <div class="flex gap-xl">
        <div class="w-[35%] flex items-center pl-md">
          <Stack>
            <!-- this text only shows the first time / user is in onboarding -->
            <template v-if="isOnboarding">
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
                <h2>Update your profile</h2>
                <p>Use this page to update your profile info.</p>
                <p>We won't share your info with anyone.</p>
              </RichText>
            </template>
          </Stack>
        </div>

        <form class="grow my-md p-md">
          <Stack spacing="lg">
            <Stack>
              <ErrorMessage :request-status="updateUserReqStatus" />
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
                <div v-else class="h-xl items-center flex flex-row gap-sm">
                  <div class="italic text-sm">No image set.</div>
                  <VButton
                    v-if="storeUser?.pictureUrl"
                    tone="action"
                    size="xs"
                    variant="ghost"
                    @click="restorePicture"
                    >Restore Picture
                  </VButton>
                </div>

                <!--
  v-model="draftUser.pictureUrl"
              placeholder="Leave blank to have no profile picture"
-->
              </VormInput>
              <Tiles columns="2" spacing="sm" columns-mobile="1">
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
                required
                :regex="GITHUB_USERNAME_REGEX"
                regex-message="Invalid github username"
              />
              <VormInput
                v-model="draftUser.discordUsername"
                label="Discord Username"
                name="discord_username"
                placeholder="ex: eggscellent#1234"
                required
                :regex="DISCORD_TAG_REGEX"
                regex-message="Invalid discord tag"
              />

              <VButton
                icon-right="chevron--right"
                :disabled="validationState.isError"
                :request-status="updateUserReqStatus"
                loading-text="Saving your profile..."
                success-text="Updated your profile!"
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

const GITHUB_USERNAME_REGEX = /^[a-z\d](?:[a-z\d]|-(?=[a-z\d])){0,38}$/i;
const DISCORD_TAG_REGEX =
  /^((?!(discordUsername|everyone|here)#)((?!@|#|:|```).{2,32})#?[0-9]?)/;

const { validationState, validationMethods } = useValidatedInputGroup();
const authStore = useAuthStore();
const router = useRouter();

const loadUserReqStatus = authStore.getRequestStatus("LOAD_USER");
const checkAuthReqStatus = authStore.getRequestStatus("CHECK_AUTH");
const updateUserReqStatus = authStore.getRequestStatus("UPDATE_USER");

const storeUser = computed(() => authStore.user);
const draftUser = ref<User>();

useHead({ title: "Profile" });

const isOnboarding = ref(authStore.needsProfileUpdate);

function resetDraftUser() {
  draftUser.value = _.cloneDeep(storeUser.value!);
}

watch(storeUser, resetDraftUser, { immediate: true });

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
  const goToNextStepOnSave = isOnboarding.value;
  const updateReq = await authStore.UPDATE_USER(draftUser.value!);
  if (updateReq.result.success && goToNextStepOnSave) {
    tracker.trackEvent("initial_profile_set", {
      email: draftUser.value?.email,
      githubUsername: draftUser.value?.githubUsername,
      discordUsername: draftUser.value?.discordUsername,
      firstName: draftUser.value?.firstName,
      lastName: draftUser.value?.lastName,
    });
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
