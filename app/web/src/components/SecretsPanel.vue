<template>
  <ScrollArea
    :class="
      clsx(
        'flex flex-col w-full h-full',
        addingSecretId && 'justify-items-stretch',
      )
    "
  >
    <template #top>
      <SiSearch
        ref="searchRef"
        placeholder="search secrets"
        :filters="searchFiltersWithCounts"
        @search="onSearch"
      />
    </template>

    <template v-if="secretsStore.secretsByLastCreated.length > 0">
      <TreeNode
        v-for="definition in filteredSecrets"
        :ref="
        (treeNode) => {
          if (secretDefinitionRefs) {
            // TODO - fix type here
            secretDefinitionRefs[definition.id] = treeNode as any;
          }
        }
      "
        :key="definition.id"
        alwaysShowArrow
        enableGroupToggle
        :defaultOpen="
          (secretsStore.secretsByDefinitionId[definition.id]?.length || 0) > 0
        "
        classes="bg-neutral-100 dark:bg-neutral-700"
        noIndentationOrLeftBorder
        enableDefaultHoverClasses
      >
        <template #label>
          <div class="flex-grow text-sm font-bold truncate leading-loose">
            {{ definition.id }}
          </div>
        </template>
        <template #openLabel>
          <div class="flex-grow text-sm font-bold break-words overflow-hidden">
            {{ definition.id }}
          </div>
        </template>
        <template #icons>
          <div class="flex flex-row flex-none items-center gap-2xs pl-xs">
            <PillCounter
              :count="definition.secrets.length"
              showHoverInsideTreeNode
            />
            <IconButton
              icon="plus"
              tone="action"
              @click.stop="openAddSecretForm(definition.id)"
            />
          </div>
        </template>
        <template #default>
          <div
            v-if="definition.secrets.length === 0"
            class="p-sm text-center text-neutral-400"
          >
            <template
              v-if="
                secretsStore.secretsByDefinitionId[definition.id]?.length === 0
              "
            >
              No secrets of this definition found.
            </template>
            <template v-else>
              No secrets of this definition match your search.
            </template>
          </div>
          <SecretCard
            v-for="secret in definition.secrets"
            v-else
            :key="secret.id"
            :class="secret.id"
            :secret="secret"
            :selected="editingSecret === secret"
            @select="toggleDrawer(definition.id, secret)"
            @edit="toggleDrawer(definition.id, secret, true)"
            @deleted="closeDrawerIfDeleted(secret)"
          />
        </template>
      </TreeNode>
    </template>
    <RequestStatusMessage
      v-else-if="!secretsLoadingRequestStatus.isSuccess"
      :requestStatus="secretsLoadingRequestStatus"
      loadingMessage="Loading Secrets..."
    />
    <div
      v-else
      class="w-full text-center p-sm text-neutral-500 dark:text-neutral-400 italic"
    >
      No secret definitions found.
    </div>
    <RightPanelDrawer :open="!!addingSecretId">
      <TabGroup
        v-if="addingSecretId && editingSecret"
        ref="drawerTabGroupRef"
        startSelectedTabSlug="form"
        @closeButtonTabClicked="closeAddSecretForm"
      >
        <TabGroupCloseButton />
        <TabGroupItem :label="addingSecretId" slug="form">
          <div class="flex flex-col">
            <div
              :class="
                clsx(
                  'flex-none flex flex-col p-xs border-y h-8',
                  themeClasses('border-neutral-200', 'border-neutral-700'),
                )
              "
            >
              <div
                v-if="!editingSecret.isUsable"
                :class="
                  clsx(
                    'w-full text-xs font-bold',
                    themeClasses(
                      'text-destructive-600',
                      'text-destructive-500',
                    ),
                  )
                "
              >
                Created in another workspace, replace this secret to be able to
                use it.
              </div>
              <div
                v-else-if="editingSecret.updatedInfo"
                :class="
                  clsx(
                    'w-full text-xs flex flex-row items-center gap-xs',
                    themeClasses('text-neutral-500', 'text-neutral-400'),
                  )
                "
              >
                <div class="flex-none">
                  <span class="font-bold">Updated: </span>
                  <Timestamp
                    :date="new Date(editingSecret.updatedInfo.timestamp)"
                    size="long"
                  />
                </div>
                <div class="flex-none font-bold">|</div>
                <TruncateWithTooltip>
                  <span class="font-bold">By: </span>
                  {{ editingSecret.updatedInfo.actor.label }}
                </TruncateWithTooltip>
              </div>
              <div
                v-else
                :class="
                  clsx(
                    'w-full text-xs flex flex-row items-center gap-xs',
                    themeClasses('text-neutral-500', 'text-neutral-400'),
                  )
                "
              >
                <div class="flex-none">
                  <span class="font-bold">Created: </span>
                  <Timestamp
                    :date="new Date(editingSecret.createdInfo.timestamp)"
                    size="long"
                  />
                </div>
                <div class="flex-none font-bold">|</div>
                <TruncateWithTooltip>
                  <span class="font-bold">By: </span>
                  {{ editingSecret.createdInfo.actor.label }}
                </TruncateWithTooltip>
              </div>
            </div>
            <div class="flex-grow">
              <AddSecretForm
                :key="editingSecret?.name"
                :definitionId="addingSecretId"
                :editingSecret="editingSecret"
                :replacing="replacingSecret"
                @save="finishSavingSecret"
                @cancel="closeAddSecretForm"
              />
            </div>
          </div>
        </TabGroupItem>
        <TabGroupItem
          v-if="editingSecret"
          label="Connected Components"
          slug="connected"
        >
          <ScrollArea
            v-if="editingSecret.connectedComponents.length > 0"
            class="w-full h-full"
          >
            <template #top>
              <div
                :class="
                  clsx(
                    'text-sm italic p-xs border-b truncate',
                    themeClasses(
                      'border-neutral-200 text-neutral-600',
                      'border-neutral-700 text-neutral-400',
                    ),
                  )
                "
              >
                <span class="font-bold">{{
                  editingSecret.connectedComponents.length
                }}</span>
                component{{
                  editingSecret.connectedComponents.length > 1 ? "s" : ""
                }}
                currently using
                <span class="font-bold">"{{ editingSecret.name }}"</span>

                <SiSearch
                  ref="searchComponentsRef"
                  placeholder="search components"
                  @search="onComponentSearch"
                />
              </div>
            </template>
            <div class="flex flex-col gap-xs p-xs">
              <ComponentCard
                v-for="id in filteredComponents"
                :key="id"
                :componentId="id"
              />
            </div>
          </ScrollArea>
          <EmptyStateCard
            v-else
            iconName="no-components"
            primaryText="No components are currently using this secret."
            secondaryText="This secret can be used in a component or deleted."
          />
        </TabGroupItem>
      </TabGroup>
      <div v-else-if="addingSecretId" class="flex flex-col">
        <div class="h-8 flex flex-row items-center gap-xs">
          <Icon
            name="x"
            size="lg"
            :class="
              clsx(
                'flex-none cursor-pointer',
                themeClasses('hover:text-action-500', 'hover:text-action-300'),
              )
            "
            @click="closeAddSecretForm"
          />
          <div class="grow text-xs truncate uppercase font-bold text-center">
            New {{ addingSecretId }}
          </div>
        </div>
        <div
          :class="
            clsx(
              'flex-none p-xs border-y h-8 text-xs italic',
              themeClasses(
                'border-neutral-200 text-neutral-500',
                'border-neutral-700 text-neutral-400',
              ),
            )
          "
        >
          Fill out the form below to add your secret to System Initiative.
        </div>
        <div class="flex-grow">
          <AddSecretForm
            :definitionId="addingSecretId"
            @save="finishSavingSecret"
            @cancel="closeAddSecretForm"
          />
        </div>
      </div>
    </RightPanelDrawer>
  </ScrollArea>
</template>

<script lang="ts" setup>
import {
  PillCounter,
  ScrollArea,
  themeClasses,
  TreeNode,
  TabGroup,
  TabGroupItem,
  TabGroupCloseButton,
  RequestStatusMessage,
  Icon,
  Timestamp,
} from "@si/vue-lib/design-system";
import { computed, ref } from "vue";
import clsx from "clsx";
import { storeToRefs } from "pinia";
import {
  Secret,
  SecretId,
  SecretsOrderedArray,
  useSecretsStore,
} from "@/store/secrets.store";
import { useComponentsStore } from "@/store/components.store";
import AddSecretForm from "./AddSecretForm.vue";
import IconButton from "./IconButton.vue";
import SiSearch, { Filter } from "./SiSearch.vue";
import RightPanelDrawer from "./RightPanelDrawer.vue";
import ComponentCard from "./ComponentCard.vue";
import SecretCard from "./SecretCard.vue";
import EmptyStateCard from "./EmptyStateCard.vue";
import TruncateWithTooltip from "./TruncateWithTooltip.vue";

const secretsStore = useSecretsStore();
const { secretsByLastCreated } = storeToRefs(secretsStore);
const secretsLoadingRequestStatus =
  secretsStore.getRequestStatus("LOAD_SECRETS");

const addingSecretId = ref<SecretId>();
const editingSecret = ref<Secret>();
const replacingSecret = ref(false);
const secretDefinitionRefs = ref<
  Record<SecretId, InstanceType<typeof TreeNode>>
>({});
const drawerTabGroupRef = ref<InstanceType<typeof TabGroup>>();

const componentStore = useComponentsStore();

const openAddSecretForm = (
  secretId: SecretId,
  edit?: Secret,
  replace = false,
) => {
  editingSecret.value = edit;
  addingSecretId.value = secretId;
  replacingSecret.value = replace;
  if (replace && drawerTabGroupRef.value) {
    drawerTabGroupRef.value.selectTab("form");
  }
};

const closeAddSecretForm = () => {
  editingSecret.value = undefined;
  addingSecretId.value = undefined;
  replacingSecret.value = false;
};

const finishSavingSecret = () => {
  if (secretDefinitionRefs.value && addingSecretId.value) {
    // When the new secret is created, make sure the definition TreeNode is open!
    secretDefinitionRefs.value[addingSecretId.value]?.toggleIsOpen(true);
  }
  closeAddSecretForm();
};

const toggleDrawer = (secretId: SecretId, edit: Secret, replace = false) => {
  if (editingSecret.value === edit && !replace) {
    closeAddSecretForm();
  } else {
    openAddSecretForm(secretId, edit, replace);
  }
};

const closeDrawerIfDeleted = (deleted: Secret) => {
  if (editingSecret.value === deleted) closeAddSecretForm();
};

const searchRef = ref<InstanceType<typeof SiSearch>>();
const searchString = ref("");

const searchComponentsRef = ref<InstanceType<typeof SiSearch>>();
const searchComponentString = ref("");

const onSearch = (search: string) => {
  searchString.value = search.trim().toLocaleLowerCase();
};

const onComponentSearch = (search: string) => {
  searchComponentString.value = search.trim().toLocaleLowerCase();
};

const filteredComponents = computed(() => {
  if (searchComponentString.value.length === 0)
    return editingSecret.value?.connectedComponents;

  return editingSecret.value?.connectedComponents.filter((componentId) => {
    const component = componentStore.componentsById[componentId];
    if (
      component?.displayName
        .toLocaleLowerCase()
        .includes(searchComponentString.value)
    )
      return true;
    if (
      component?.schemaName
        .toLocaleLowerCase()
        .includes(searchComponentString.value)
    )
      return true;
    return false;
  });
});

const searchFiltersWithCounts = computed(() => {
  const searchFilters: Array<Filter> = [];

  secretsByLastCreated.value.forEach((definition) => {
    if (definition.secrets.length > 0) {
      searchFilters.push({
        name: definition.id,
        count: definition.secrets.length,
      });
    }
  });

  return searchFilters;
});

const filteredSecrets = computed(() => {
  const filtered = [] as SecretsOrderedArray;

  if (
    searchFiltersWithCounts.value &&
    searchRef.value &&
    searchRef.value.filteringActive
  ) {
    // Restrict the secrets to only the selected definitions
    searchRef.value.activeFilters.forEach((enabled, index) => {
      const definition = secretsByLastCreated.value[index];
      if (enabled && definition) {
        filtered.push(definition);
      }
    });
  } else {
    filtered.push(...secretsByLastCreated.value);
  }

  // Now filter the remaining secrets by the search string
  const s = searchString.value;
  const searched = [] as SecretsOrderedArray;

  filtered.forEach((definition) => {
    if (definition.id.includes(s)) searched.push(definition);
    else {
      const filteredDef = {
        ...definition,
        secrets: [] as Array<Secret>,
      };
      definition.secrets.forEach((secret) => {
        if (secret.name.includes(s)) filteredDef.secrets.push(secret);
      });
      searched.push(filteredDef);
    }
  });

  return searched;
});
</script>
