<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <ResizablePanel rememberSizeKey="func-picker" side="left" :minSize="300">
    <div class="flex flex-col h-full">
      <div class="relative flex-grow">
        <CustomizeTabs tabContentSlug="secrets">
          <AddSecretForm definitionId="Mocks" class="h-auto" />
          <ul class="m-xs">
            <li v-for="(definition, index) in secretDefinitions" :key="index">
              <span>{{ definition }}</span>
              <ul class="ml-md flex flex-col gap-1">
                <li
                  v-for="secret in secretsStore.secretsByDefinitionId[
                    definition
                  ]"
                  :key="secret.id"
                  class="text-sm"
                >
                  <b>{{ secret.name }}</b>
                  <i class="text-xs text-neutral-500">
                    by {{ secret.createdInfo.actor.label }}
                  </i>
                  <VButton
                    :disabled="secretsStore.secretIsTransitioning[secret.id]"
                    class="ml-2"
                    size="xs"
                    tone="neutral"
                    icon="x-circle"
                    @click="secretsStore.DELETE_SECRET(secret.id)"
                  />
                </li>
              </ul>
            </li>
          </ul>
        </CustomizeTabs>
      </div>
    </div>
  </ResizablePanel>
  <div
    class="grow overflow-hidden bg-shade-0 dark:bg-neutral-800 dark:text-shade-0 font-semi-bold flex flex-col relative"
  >
    <div class="inset-0 p-sm absolute overflow-auto">SECRETS ENTRIES LIST</div>
  </div>
  <ResizablePanel rememberSizeKey="func-details" side="right" :minSize="200">
    <div v-if="FF_SECRETS" class="flex flex-col h-full items-center">
      <ApplyChangeSetButton class="w-10/12 mx-auto my-4" />
      <SidebarSubpanelTitle>Secret Details</SidebarSubpanelTitle>
      WIP
    </div>
  </ResizablePanel>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import { ResizablePanel, VButton } from "@si/vue-lib/design-system";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import SidebarSubpanelTitle from "@/components/SidebarSubpanelTitle.vue";
import ApplyChangeSetButton from "@/components/ApplyChangeSetButton.vue";
import { useSecretsStore } from "@/store/secrets.store";
import AddSecretForm from "@/components/AddSecretForm.vue";
import CustomizeTabs from "../CustomizeTabs.vue";

const secretsStore = useSecretsStore();
const featureFlagsStore = useFeatureFlagsStore();

const FF_SECRETS = computed(() => featureFlagsStore.SECRETS);

const secretDefinitions = computed(() => secretsStore.definitions);
</script>
