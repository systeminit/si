<template>
  <div class="overflow-scroll">
    <template v-if="featureFlagsStore.ADMIN_PAGE">
      <div>
        <h2 class="pb-md font-bold">Administration Dashboard</h2>

        <div class="pb-md">
          <VButton
            iconRight="chevron--right"
            tone="action"
            variant="solid"
            @click="router.push({ name: 'rum-report' })"
          >
            View RUM Report
          </VButton>
        </div>

        <divider class="my-4" />

        <h3 class="pb-md font-bold">
          Create production workspace for an invited user
        </h3>

        <p>Create by User Email</p>
        <Stack>
          <ErrorMessage :requestStatus="createWorkspaceReqStatus" />
          <VormInput
            v-model="workspace.userEmail"
            :maxLength="500"
            label="Email"
            placeholder="The email of the user to invite"
            required
          />

          <VButton
            :requestStatus="createWorkspaceReqStatus"
            iconRight="chevron--right"
            loadingText="Creating..."
            tone="action"
            variant="solid"
            @click="createWorkspace()"
          >
            Create Workspace For User Email Address
          </VButton>
        </Stack>
        <p class="mt-4">
          Create by User Id - this must be an ID that is in our auth-api
          database
        </p>
        <Stack>
          <ErrorMessage :requestStatus="createWorkspaceByUserIdReqStatus" />
          <VormInput
            v-model="workspaceById.userId"
            :maxLength="500"
            label="User ID"
            placeholder="The user id of the user to invite"
            required
          />

          <VButton
            :requestStatus="createWorkspaceByUserIdReqStatus"
            iconRight="chevron--right"
            loadingText="Creating..."
            tone="action"
            variant="solid"
            @click="createWorkspaceById()"
          >
            Create Workspace for Know SystemInit UserID
          </VButton>
        </Stack>
      </div>
      <divider class="my-4" />
      <div>
        <h3 class="pb-md font-bold">User Accounts by Email</h3>
        <Stack>
          <VormInput
            v-model="usersByEmail"
            label="Email"
            placeholder="The email record to lookup"
            required
          />
          <VButton
            :requestStatus="userRecordsByEmailReqStatus"
            :disabled="_.isEmpty(usersByEmail)"
            iconRight="chevron--right"
            loadingText="Searching..."
            tone="action"
            variant="solid"
            @click="getUserRecordsForEmail()"
          >
            Search for User Records
          </VButton>
        </Stack>
        <template v-if="userRecordsByEmailReqStatus.isPending">
          <Icon name="loader" />
        </template>
        <template v-else-if="userRecordsByEmailReqStatus.isError">
          <ErrorMessage :requestStatus="userRecordsByEmailReqStatus" />
        </template>
        <template v-else-if="userRecordsByEmailReqStatus.isSuccess">
          <div class="relative">
            <div class="text-lg font-bold">Currently Quarantined Users:</div>
            <table
              class="w-full divide-y divide-neutral-400 dark:divide-neutral-600 border-b border-neutral-400 dark:border-neutral-600"
            >
              <thead>
                <tr
                  class="children:pb-xs children:px-md children:font-bold text-left text-xs uppercase"
                >
                  <th scope="col">User ID</th>
                  <th scope="col">Auth0 ID</th>
                  <th scope="col">Email</th>
                  <th scope="col">Is Quarantined?</th>
                  <th scope="col">Is Suspended?</th>
                </tr>
              </thead>
              <tbody
                class="divide-y divide-neutral-300 dark:divide-neutral-700"
              >
                <tr
                  v-for="userRecord in userRecords"
                  :key="userRecord.id"
                  class="children:px-md children:py-sm children:truncate text-sm font-medium text-neutral-800 dark:text-neutral-200"
                >
                  <td class="">
                    <div
                      class="xl:max-w-[800px] lg:max-w-[60vw] md:max-w-[50vw] sm:max-w-[40vw] max-w-[150px] truncate"
                    >
                      {{ userRecord.id }}
                    </div>
                  </td>
                  <td class="normal-case">
                    {{ formatAuth0Signup(userRecord.auth0Id) }}
                  </td>
                  <td class="normal-case">
                    {{ userRecord.email }}
                  </td>
                  <td class="normal-case">
                    {{ hasDate(userRecord.quarantinedAt) }}
                  </td>
                  <td class="normal-case">
                    {{ hasDate(userRecord.suspendedAt) }}
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </template>
      </div>
      <divider class="my-4" />
      <div>
        <h3 class="pb-md font-bold">Manage Account Quarantine</h3>

        <Stack>
          <ErrorMessage :requestStatus="setUserQuarantineReqStatus" />
          <VormInput
            v-model="quarantineUserId"
            :maxLength="26"
            label="User ID"
            placeholder="The user id of the account to be managed"
            required
          />

          <div class="flex gap-2">
            <VButton
              :disabled="_.isEmpty(quarantineUserId)"
              :requestStatus="setUserQuarantineReqStatus"
              class="grow"
              icon="lock"
              tone="destructive"
              variant="solid"
              @click="setUserQuarantine(true)"
            >
              Quarantine Account
            </VButton>
            <VButton
              :disabled="_.isEmpty(quarantineUserId)"
              :requestStatus="setUserQuarantineReqStatus"
              class="grow"
              icon="lock-open"
              tone="success"
              variant="solid"
              @click="setUserQuarantine(false)"
            >
              Unquarantine Account
            </VButton>
          </div>
        </Stack>
        <template v-if="quarantinedUsersReqStatus.isPending">
          <Icon name="loader" />
        </template>
        <template v-else-if="quarantinedUsersReqStatus.isError">
          <ErrorMessage :requestStatus="quarantinedUsersReqStatus" />
        </template>
        <template v-else-if="quarantinedUsersReqStatus.isSuccess">
          <div class="relative">
            <div class="text-lg font-bold">Currently Quarantined Users:</div>
            <table
              class="w-full divide-y divide-neutral-400 dark:divide-neutral-600 border-b border-neutral-400 dark:border-neutral-600"
            >
              <thead>
                <tr
                  class="children:pb-xs children:px-md children:font-bold text-left text-xs uppercase"
                >
                  <th scope="col">User ID</th>
                  <th scope="col">Email</th>
                  <th scope="col">Quarantined At</th>
                </tr>
              </thead>
              <tbody
                class="divide-y divide-neutral-300 dark:divide-neutral-700"
              >
                <tr
                  v-for="quarantinedUser in quarantinedUsers"
                  :key="quarantinedUser.userId"
                  class="children:px-md children:py-sm children:truncate text-sm font-medium text-neutral-800 dark:text-neutral-200"
                >
                  <td class="">
                    <div
                      class="xl:max-w-[800px] lg:max-w-[60vw] md:max-w-[50vw] sm:max-w-[40vw] max-w-[150px] truncate"
                    >
                      {{ quarantinedUser.userId }}
                    </div>
                  </td>
                  <td class="normal-case">
                    {{ quarantinedUser.email }}
                  </td>
                  <td class="normal-case">
                    {{ quarantinedUser.quarantinedAt.toString() }}
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </template>
      </div>
      <divider class="my-4" />
      <div>
        <h3 class="pb-md font-bold">Manage Workspace Quarantine</h3>

        <Stack>
          <ErrorMessage :requestStatus="setWorkspaceQuarantineReqStatus" />
          <VormInput
            v-model="quarantineWorkspaceId"
            :maxLength="26"
            label="Workspace ID"
            placeholder="The id of the workspace to be managed"
            required
          />

          <div class="flex gap-2">
            <VButton
              :disabled="_.isEmpty(quarantineWorkspaceId)"
              :requestStatus="setWorkspaceQuarantineReqStatus"
              class="grow"
              icon="lock"
              tone="destructive"
              variant="solid"
              @click="setWorkspaceQuarantine(true)"
            >
              Quarantine Workspace
            </VButton>
            <VButton
              :disabled="_.isEmpty(quarantineWorkspaceId)"
              :requestStatus="setWorkspaceQuarantineReqStatus"
              class="grow"
              icon="lock-open"
              tone="success"
              variant="solid"
              @click="setWorkspaceQuarantine(false)"
            >
              Unquarantine Workspace
            </VButton>
          </div>
        </Stack>
      </div>
      <divider class="my-4" />
      <div>
        <h3 class="pb-md font-bold">Manage User Suspensions</h3>

        <Stack>
          <ErrorMessage :requestStatus="setUserSuspensionReqStatus" />
          <VormInput
            v-model="suspensionUserId"
            :maxLength="26"
            label="User ID"
            placeholder="The user id of the account to be managed"
            required
          />

          <div class="flex gap-2">
            <VButton
              :disabled="_.isEmpty(suspensionUserId)"
              :requestStatus="setUserSuspensionReqStatus"
              class="grow"
              icon="lock"
              tone="destructive"
              variant="solid"
              @click="setUserSuspension(true)"
            >
              Suspend Account
            </VButton>
            <VButton
              :disabled="_.isEmpty(suspensionUserId)"
              :requestStatus="setUserSuspensionReqStatus"
              class="grow"
              icon="lock-open"
              tone="success"
              variant="solid"
              @click="setUserSuspension(false)"
            >
              Unsuspend Account
            </VButton>
          </div>
        </Stack>
        <template v-if="suspendedUsersReqStatus.isPending">
          <Icon name="loader" />
        </template>
        <template v-else-if="suspendedUsersReqStatus.isError">
          <ErrorMessage :requestStatus="suspendedUsersReqStatus" />
        </template>
        <template v-else-if="suspendedUsersReqStatus.isSuccess">
          <div class="relative">
            <div class="text-lg font-bold">Currently Suspended Users:</div>
            <table
              class="w-full divide-y divide-neutral-400 dark:divide-neutral-600 border-b border-neutral-400 dark:border-neutral-600"
            >
              <thead>
                <tr
                  class="children:pb-xs children:px-md children:font-bold text-left text-xs uppercase"
                >
                  <th scope="col">User ID</th>
                  <th scope="col">Email</th>
                  <th scope="col">Suspended At</th>
                </tr>
              </thead>
              <tbody
                class="divide-y divide-neutral-300 dark:divide-neutral-700"
              >
                <tr
                  v-for="suspendedUser in suspendedUsers"
                  :key="suspendedUser.userId"
                  class="children:px-md children:py-sm children:truncate text-sm font-medium text-neutral-800 dark:text-neutral-200"
                >
                  <td class="">
                    <div
                      class="xl:max-w-[800px] lg:max-w-[60vw] md:max-w-[50vw] sm:max-w-[40vw] max-w-[150px] truncate"
                    >
                      {{ suspendedUser.userId }}
                    </div>
                  </td>
                  <td class="normal-case">
                    {{ suspendedUser.email }}
                  </td>
                  <td class="normal-case">
                    {{ suspendedUser.suspendedAt.toString() }}
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </template>
      </div>
      <divider class="my-4" />
      <div>
        <h3 class="pb-md font-bold">Who owns this workspace?</h3>
        <Stack>
          <VormInput
            v-model="workspaceIdQuery"
            label="Workspace Id"
            placeholder="The workspace ID to lookup"
            required
          />
          <div class="flex gap-2">
            <VButton
              :disabled="_.isEmpty(workspaceIdQuery)"
              :requestStatus="getWorkspaceOwnerReqStatus"
              iconRight="chevron--right"
              loadingText="Querying..."
              tone="action"
              variant="solid"
              @click="getWorkspaceOwner()"
            >
              Get Workspace Owner
            </VButton>
          </div>
        </Stack>
        <template v-if="getWorkspaceOwnerReqStatus.isPending">
          <Icon name="loader" />
        </template>
        <template v-else-if="getWorkspaceOwnerReqStatus.isError">
          <ErrorMessage :requestStatus="getWorkspaceOwnerReqStatus" />
        </template>
        <template v-else-if="getWorkspaceOwnerReqStatus.isSuccess">
          <div class="relative">
            <div class="text-lg font-bold">Owner Details</div>
            <table
              v-if="workspaceOwner"
              class="w-full divide-y divide-neutral-400 dark:divide-neutral-600 border-b border-neutral-400 dark:border-neutral-600"
            >
              <thead>
                <tr
                  class="children:pb-xs children:px-md children:font-bold text-left text-xs uppercase"
                >
                  <th scope="col">First Name</th>
                  <th scope="col">Last Name</th>
                  <th scope="col">Email</th>
                  <th scope="col">Display Name</th>
                  <th scope="col">Instance Url</th>
                </tr>
              </thead>
              <tbody
                class="divide-y divide-neutral-300 dark:divide-neutral-700"
              >
                <tr
                  class="children:px-md children:py-sm children:truncate text-sm font-medium text-neutral-800 dark:text-neutral-200"
                >
                  <td class="">
                    <div
                      class="xl:max-w-[800px] lg:max-w-[60vw] md:max-w-[50vw] sm:max-w-[40vw] max-w-[150px] truncate"
                    >
                      {{ workspaceOwner.firstName || "" }}
                    </div>
                  </td>
                  <td class="normal-case">
                    {{ workspaceOwner.lastName || "" }}
                  </td>
                  <td class="normal-case">
                    {{ workspaceOwner.email }}
                  </td>
                  <td class="normal-case">
                    {{ workspaceOwner.displayName || "" }}
                  </td>
                  <td class="normal-case">
                    {{ workspaceOwner.instanceUrl }}
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </template>
      </div>
      <divider class="my-4" />
      <div>
        <h3 class="pb-md font-bold">Generate Weekly Signup Report</h3>
        <Stack>
          <VormInput
            v-model="reportStartDate"
            label="Start Date"
            placeholder="The start date for the user signups report e.g. 2024-08-14"
            required
          />
          <VormInput
            v-model="reportEndDate"
            label="End Date"
            placeholder="The end date for the user signups reporte.g. 2024-08-21"
            required
          />
          <div class="flex gap-2">
            <VButton
              :disabled="_.isEmpty(reportStartDate) || _.isEmpty(reportEndDate)"
              :requestStatus="setUserSignupsReqStatus"
              iconRight="chevron--right"
              loadingText="Generating Report..."
              tone="action"
              variant="solid"
              @click="getUserSignupsReport()"
            >
              Generate Report
            </VButton>
          </div>
        </Stack>
        <template v-if="setUserSignupsReqStatus.isPending">
          <Icon name="loader" />
        </template>
        <template v-else-if="setUserSignupsReqStatus.isError">
          <ErrorMessage :requestStatus="setUserSignupsReqStatus" />
        </template>
        <template v-else-if="setUserSignupsReqStatus.isSuccess">
          <div class="relative">
            <div class="text-lg font-bold">Signups:</div>
            <table
              class="w-full divide-y divide-neutral-400 dark:divide-neutral-600 border-b border-neutral-400 dark:border-neutral-600"
            >
              <thead>
                <tr
                  class="children:pb-xs children:px-md children:font-bold text-left text-xs uppercase"
                >
                  <th scope="col">First Name</th>
                  <th scope="col">Last Name</th>
                  <th scope="col">Email</th>
                  <th scope="col">SignUp Method</th>
                  <th scope="col">Discord</th>
                  <th scope="col">GitHub</th>
                  <th scope="col">Signup At</th>
                </tr>
              </thead>
              <tbody
                class="divide-y divide-neutral-300 dark:divide-neutral-700"
              >
                <tr
                  v-for="user in userSignups"
                  :key="user.email"
                  class="children:px-md children:py-sm children:truncate text-sm font-medium text-neutral-800 dark:text-neutral-200"
                >
                  <td class="">
                    <div
                      class="xl:max-w-[800px] lg:max-w-[60vw] md:max-w-[50vw] sm:max-w-[40vw] max-w-[150px] truncate"
                    >
                      {{ user.firstName }}
                    </div>
                  </td>
                  <td class="normal-case">
                    {{ user.lastName }}
                  </td>
                  <td class="normal-case">
                    {{ user.email }}
                  </td>
                  <td class="normal-case">{{ user.signupMethod }}</td>
                  <td class="normal-case">
                    {{ user.discordUsername }}
                  </td>
                  <td class="normal-case">
                    {{ user.githubUsername }}
                  </td>
                  <td class="normal-case">
                    {{ formatDate(user.signupAt) }}
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </template>
      </div>
    </template>
    <template v-else> Feature not Enabled for account </template>
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, onBeforeMount, onMounted, reactive, ref } from "vue";
import {
  Icon,
  VormInput,
  Stack,
  ErrorMessage,
  VButton,
  Divider,
} from "@si/vue-lib/design-system";
import { useRouter } from "vue-router";
import { User, useAuthStore } from "@/store/auth.store";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";

const authStore = useAuthStore();
const router = useRouter();
const workspacesStore = useWorkspacesStore();
const featureFlagsStore = useFeatureFlagsStore();

const invitedUserByEmail = {
  userEmail: "",
};
const invitedUserById = {
  userId: "",
};
const workspace = reactive(_.cloneDeep(invitedUserByEmail));
const workspaceById = reactive(_.cloneDeep(invitedUserById));

const createWorkspaceReqStatus = workspacesStore.getRequestStatus(
  "SETUP_PRODUCTION_WORKSPACE",
);
const createWorkspace = async () => {
  await workspacesStore.SETUP_PRODUCTION_WORKSPACE(workspace.userEmail);
};

const createWorkspaceByUserIdReqStatus = workspacesStore.getRequestStatus(
  "SETUP_PRODUCTION_WORKSPACE_BY_USER_ID",
);
const createWorkspaceById = async () => {
  await workspacesStore.SETUP_PRODUCTION_WORKSPACE_BY_USER_ID(
    workspaceById.userId,
  );
};

// User quarantine state
const quarantineUserId = ref("");
const setUserQuarantineReqStatus = authStore.getRequestStatus(
  "SET_USER_QUARANTINE",
);
const setUserQuarantine = async (isQuarantined: boolean) => {
  await authStore.SET_USER_QUARANTINE(quarantineUserId.value, isQuarantined);
};

// Workspace quarantine state
const quarantineWorkspaceId = ref("");
const setWorkspaceQuarantineReqStatus = workspacesStore.getRequestStatus(
  "SET_WORKSPACE_QUARANTINE",
);
const setWorkspaceQuarantine = async (isQuarantined: boolean) => {
  await workspacesStore.SET_WORKSPACE_QUARANTINE(
    quarantineWorkspaceId.value,
    isQuarantined,
  );
};

// User suspension state
const suspensionUserId = ref("");
const setUserSuspensionReqStatus = authStore.getRequestStatus(
  "SET_USER_SUSPENSION",
);
const setUserSuspension = async (isSuspended: boolean) => {
  await authStore.SET_USER_SUSPENSION(suspensionUserId.value, isSuspended);
};

const quarantinedUsersReqStatus = authStore.getRequestStatus(
  "GET_QUARANTINED_USERS",
);
const quarantinedUsers = computed(() => authStore.quarantinedUsers);

const suspendedUsersReqStatus = authStore.getRequestStatus(
  "GET_SUSPENDED_USERS",
);
const suspendedUsers = computed(() => authStore.suspendedUsers);

onMounted(async () => {
  if (!authStore.userIsLoggedIn) return;

  if (
    !authStore.user?.email?.includes("@systeminit.com") &&
    !featureFlagsStore.ADMIN_PAGE
  ) {
    await router.push({
      name: "workspaces",
    });
  }
});

function formatDate(date: string | Date): string {
  return new Date(date).toLocaleDateString();
}

const reportStartDate = ref("");
const reportEndDate = ref("");

const setUserSignupsReqStatus = authStore.getRequestStatus(
  "GET_USER_SIGNUP_REPORT",
);
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const userSignups = ref<any[]>([]);
async function getUserSignupsReport() {
  if (!reportStartDate.value || !reportEndDate.value) return;

  const formattedStartDate = new Date(reportStartDate.value);
  const formattedEndDate = new Date(reportEndDate.value);

  await authStore.GET_USER_SIGNUP_REPORT(formattedStartDate, formattedEndDate);
  userSignups.value = authStore.userSignups;
}

const workspaceIdQuery = ref("");
const workspaceOwner = computed(() => workspacesStore.workspaceForOwner);
async function getWorkspaceOwner() {
  if (!workspaceIdQuery.value) return;

  await workspacesStore.GET_WORKSPACE_OWNER(workspaceIdQuery.value);
}

const getWorkspaceOwnerReqStatus = workspacesStore.getRequestStatus(
  "GET_WORKSPACE_OWNER",
);

const usersByEmail = ref("");
const userRecords = ref<User[]>([]);
const userRecordsByEmailReqStatus = authStore.getRequestStatus(
  "GET_USERS_FOR_EMAIL",
  usersByEmail.value,
);
async function getUserRecordsForEmail() {
  if (!usersByEmail.value) return;

  const users = await authStore.GET_USERS_FOR_EMAIL(usersByEmail.value);
  userRecords.value = users.result.data || [];
}

function formatAuth0Signup(signup: string) {
  const signupMethod = signup.split("|", 1);
  return signupMethod[0];
}

function hasDate(date: string | undefined) {
  if (!date || date === "") return false;

  return true;
}

onBeforeMount(async () => {
  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  authStore.GET_QUARANTINED_USERS();
  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  authStore.GET_SUSPENDED_USERS();
});
</script>
