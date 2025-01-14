<template>
  <ScrollArea>
    <template #top>
      <ViewCard :view="props.view" />
    </template>

    <TabGroup
      ref="tabsRef"
      trackingSlug="asset_details"
      startSelectedTabSlug="approvers"
    >
      <TabGroupItem slug="approvers">
        <template #label>
          <Inline noWrap alignY="center">
            <span class="uppercase">Approvers</span>
            <PillCounter
              :count="approversCount"
              hideIfZero
              :paddingX="approversCount < 10 ? 'xs' : '2xs'"
            />
          </Inline>
        </template>

        <VormInput v-model="selectedApprover" :options="options" />
        <IconButton icon="plus" size="md" iconTone="action" @onClick="submit" />

        <hr class="border-neutral-600 my-xs" />
        <dl v-if="approversCount > 0">
          <template v-for="user in approversList" :key="user.id">
            <dt>{{ user.email }}</dt>
            <dd>
              <Icon name="trash" size="sm" tone="destructive" />
            </dd>
          </template>
        </dl>
        <div v-else>
          <EmptyStateIcon name="no-changes" />
          <p>No approvers configured. Add them above.</p>
        </div>
      </TabGroupItem>
    </TabGroup>
  </ScrollArea>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, ref } from "vue";
import {
  IconButton,
  PillCounter,
  Icon,
  Inline,
  TabGroup,
  TabGroupItem,
  ScrollArea,
} from "@si/vue-lib/design-system";
import VormInput, {
  InputOptions,
} from "@si/vue-lib/src/design-system/forms/VormInput.vue";
import { ViewDescription, ViewId } from "@/api/sdf/dal/views";
import EmptyStateIcon from "@/components/EmptyStateIcon.vue";
import ViewCard from "./ViewCard.vue";

const props = defineProps<{ viewId: ViewId; view: ViewDescription }>();

type MockUser = { id: string; email: string };

const approversList = ref<MockUser[]>([]);

const approversCount = computed(() => approversList.value.length);

const usersList = ref<MockUser[]>([]); // comes from authStore users in workspace, PR waiting to be merged

const selectedApprover = ref<InputOptions>({});

const options = computed<InputOptions>(() =>
  usersList.value.map((u) => {
    return { label: u.email, value: u.id };
  }),
);

const submit = () => {
  if (!selectedApprover.value) return;
  // TODO hit the API
};
</script>
