<!-- eslint-disable vue/no-v-html -->
<template>
  <div>
    <h2>Update your profile!</h2>

    <template v-if="reloadUserReqStatus.isPending"> loading... </template>
    <template v-else-if="reloadUserReqStatus.isError">
      Error loading TOS - {{ reloadUserReqStatus.errorMessage }}
    </template>
    <template v-else-if="reloadUserReqStatus.isSuccess && draftUser">
      <p>{{ storeUser?.email }}</p>

      <Stack>
        <ErrorMessage :request-status="updateUserReqStatus" />
        <Inline>
          <VormInput v-model="draftUser.firstName" label="First Name" />
          <VormInput v-model="draftUser.lastName" label="Last Name" />
        </Inline>
        <VormInput v-model="draftUser.nickname" label="Nickname" required />
        <VormInput
          v-model="draftUser.email"
          label="Email"
          type="email"
          required
        />
        <VormInput
          v-model="draftUser.githubUsername"
          label="Github Username"
          placeholder="ex: devopsdude42"
          required
          :regex="GITHUB_USERNAME_REGEX"
          regex-message="Invalid github username"
        />
        <VormInput
          v-model="draftUser.discordUsername"
          label="Discord Tag"
          placeholder="ex: eggscellent#1234"
          required
          :regex="DISCORD_TAG_REGEX"
          regex-message="Invalid discord tag"
        >
          <template #instructions> where to find your handle </template>
        </VormInput>
        <VButton2
          :disabled="validationState.isError"
          :request-status="updateUserReqStatus"
          @click="saveHandler"
        >
          Save
        </VButton2>
      </Stack>
    </template>
  </div>
</template>

<script setup lang="ts">
import * as _ from "lodash-es";
import { useRouter } from "vue-router";
import { computed, onBeforeMount, ref, watch } from "vue";
import {
  ErrorMessage,
  Inline,
  Stack,
  useValidatedInputGroup,
  VButton2,
  VormInput,
} from "@si/vue-lib/design-system";
import { useAuthStore, User } from "@/store/auth.store";

const GITHUB_USERNAME_REGEX = /^[a-z\d](?:[a-z\d]|-(?=[a-z\d])){0,38}$/i;
const DISCORD_TAG_REGEX =
  /^((?!(discordtag|everyone|here)#)((?!@|#|:|```).{2,32})#\d{4})/;

const { validationState, validationMethods } = useValidatedInputGroup();
const authStore = useAuthStore();
const router = useRouter();

const reloadUserReqStatus = authStore.getRequestStatus("RELOAD_USER_DATA");
const updateUserReqStatus = authStore.getRequestStatus("UPDATE_USER");

const storeUser = computed(() => authStore.user);
const draftUser = ref<User>();

function resetDraftUser() {
  draftUser.value = _.cloneDeep(storeUser.value!);
}
watch(storeUser, resetDraftUser);

onBeforeMount(() => {
  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  authStore.RELOAD_USER_DATA();
});

async function saveHandler() {
  if (validationMethods.hasError()) return;
  await authStore.UPDATE_USER(draftUser.value!);
}
</script>
