<template>
  <div
    :class="
      clsx(
        'group/pendingcard',
        'border rounded flex flex-row gap-xs items-center p-2xs cursor-pointer',
        themeClasses(
          'border-neutral-200 hover:border-action-500 hover:text-action-500 text-shade-100',
          'border-neutral-700 hover:border-action-300 hover:text-action-300 text-shade-0',
        ),
      )
    "
    @click="goToChangeSet(changeSet.id)"
  >
    <div class="group-hover/pendingcard:underline flex-1 min-w-0">
      <div class="font-bold line-clamp-2">
        {{ changeSet.name }}
      </div>
      <div
        :class="
          clsx(
            'text-xs italic',
            themeClasses(
              'text-neutral-500 group-hover/pendingcard:text-action-500',
              'text-neutral-400 group-hover/pendingcard:text-action-300',
            ),
          )
        "
      >
        <Timestamp
          :date="changeSet.mergeRequestedAt"
          showTimeIfToday
          size="extended"
        />

        by {{ changeSet.mergeRequestedByUser }}
      </div>
    </div>
    <div class="flex gap-xs flex-none">
      <VButton
        size="xs"
        label="Reject"
        variant="ghost"
        tone="destructive"
        @click.stop="rejectChangeSet(changeSet.id)"
      />
      <VButton
        size="xs"
        tone="success"
        class="grow"
        label="Approve"
        @click.stop="approveChangeSet(changeSet.id)"
      />
    </div>
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import * as _ from "lodash-es";
import { themeClasses, VButton, Timestamp } from "@si/vue-lib/design-system";
import { useRoute, useRouter } from "vue-router";
import { ChangeSet, ChangeSetId } from "@/api/sdf/dal/change_set";
import { navigateToExistingChangeSet } from "../logic_composables/change_set";
import { useContext } from "../logic_composables/context";
import { useApi, routes, apiContextForChangeSet } from "../api_composables";

defineProps<{
  changeSet: ChangeSet;
}>();

const route = useRoute();
const router = useRouter();

const goToChangeSet = (id: ChangeSetId) => {
  navigateToExistingChangeSet(id, route, router);
  emit("closeModal");
};

const ctx = useContext();

const rejectChangeSet = (id: ChangeSetId) => {
  const apiCtx = apiContextForChangeSet(ctx, id);
  const api = useApi(apiCtx);

  const call = api.endpoint(routes.ChangeSetApprove);
  call.post({ status: "Rejected" });
};

const approveChangeSet = (id: ChangeSetId) => {
  const apiCtx = apiContextForChangeSet(ctx, id);
  const api = useApi(apiCtx);

  const call = api.endpoint(routes.ChangeSetApprove);
  call.post({ status: "Approved" });
};

const emit = defineEmits<{
  (e: "closeModal"): void;
}>();
</script>
