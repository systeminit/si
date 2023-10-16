<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <ResizablePanel rememberSizeKey="func-picker" side="left" :minSize="300">
    <div class="flex flex-col h-full">
      <div class="relative flex-grow">
        <CustomizeTabs tabContentSlug="secrets">
          <template v-if="secretsStore.definitions.length > 0">
            <VormInput
              v-model="selectedDef"
              class="mx-sm mt-sm"
              label="Secret Definition"
              type="dropdown"
              :options="
                _.map(secretsStore.definitions, (d) => ({ value: d, label: d }))
              "
            />
            <AddSecretForm
              v-if="selectedDef"
              :definitionId="selectedDef"
              class="h-auto"
            />
            <p v-else>Please select a secret definition above</p>
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
                      by {{ secret.createdInfo?.actor?.label || "UNDEF" }}
                    </i>
                    <!-- Disabled until delete gets reimplemented on the backend -->
                    <!--VButton
                      :disabled="secretsStore.secretIsTransitioning[secret.id]"
                      class="ml-2"
                      size="xs"
                      tone="neutral"
                      icon="x-circle"
                      @click="secretsStore.DELETE_SECRET(secret.id)"
                    /-->
                  </li>
                </ul>
              </li>
            </ul>
          </template>
          <template v-else>
            <p>
              You need to create secret defining schema before using this page
            </p>
          </template>
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
      <SidebarSubpanelTitle>Secret Details</SidebarSubpanelTitle>
      WIP
    </div>
  </ResizablePanel>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import { ResizablePanel, VormInput } from "@si/vue-lib/design-system";
import * as _ from "lodash-es";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import SidebarSubpanelTitle from "@/components/SidebarSubpanelTitle.vue";
import { useSecretsStore } from "@/store/secrets.store";
import AddSecretForm from "@/components/AddSecretForm.vue";
import CustomizeTabs from "../CustomizeTabs.vue";

const secretsStore = useSecretsStore();
const featureFlagsStore = useFeatureFlagsStore();

const selectedDef = ref<string>();

const FF_SECRETS = computed(() => featureFlagsStore.SECRETS);

const secretDefinitions = computed(() => secretsStore.definitions);
</script>
