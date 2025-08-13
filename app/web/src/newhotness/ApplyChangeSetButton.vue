<template>
  <section v-if="!ctx.onHead.value">
    <VButton
      ref="applyButtonRef"
      size="sm"
      label="Apply Change Set"
      :class="
        clsx(
          'ml-2xs mr-xs !text-sm !border !cursor-pointer !px-xs',
          themeClasses(
            '!text-neutral-100 !bg-[#1264BF] !border-[#318AED] hover:!bg-[#2583EC]',
            '!text-neutral-100 !bg-[#1264BF] !border-[#318AED] hover:!bg-[#2583EC]',
          ),
        )
      "
      loadingText="Applying Changes"
      :loading="applyInFlight"
      @click="openApplyChangeSetModal"
    >
      <template #iconRight>
        <PillCounter
          :count="proposedActions.length"
          :paddingX="proposedActions.length > 10 ? '2xs' : 'xs'"
          noColorStyles
          class="border border-action-200 ml-2xs py-2xs"
        />
      </template>
    </VButton>
    <ApplyChangeSetModal
      ref="applyChangeSetModalRef"
      votingKind="merge"
      :actions="proposedActions"
    />
  </section>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import * as _ from "lodash-es";
import { VButton, PillCounter, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { useQuery } from "@tanstack/vue-query";
import {
  BifrostActionViewList,
  EntityKind,
} from "@/workers/types/entity_kind_types";
import { bifrost, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import ApplyChangeSetModal from "./ApplyChangeSetModal.vue";
import { useApplyChangeSet } from "./logic_composables/change_set";
import { useContext } from "./logic_composables/context";

const ctx = useContext();

const applyChangeSetModalRef = ref<InstanceType<typeof ApplyChangeSetModal>>();

const openApplyChangeSetModal = () => {
  applyChangeSetModalRef.value?.open();
};

const { applyInFlight } = useApplyChangeSet(ctx);

const key = useMakeKey();
const args = useMakeArgs();

const actionsRaw = useQuery<BifrostActionViewList | null>({
  queryKey: key(EntityKind.ActionViewList),
  queryFn: async () =>
    await bifrost<BifrostActionViewList>(args(EntityKind.ActionViewList)),
  enabled: ctx.queriesEnabled,
});
const actions = computed(() => actionsRaw.data.value?.actions ?? []);
const proposedActions = computed(() =>
  actions.value.filter(
    (action) => action.originatingChangeSetId === ctx.changeSetId.value,
  ),
);
</script>
