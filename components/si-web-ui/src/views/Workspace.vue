<template>
  <StandardLayout>
    <v-container class="justify-start d-flex" style="flex-direction: row;">
      <v-card class="flex-grow-1">
        <v-toolbar>
          <v-toolbar-title>Entities in Workspace</v-toolbar-title>

          <template v-slot:extension>
            <v-tabs v-model="tab" show-arrows centered>
              <v-tab
                v-for="siComponent in siComponentRegistry.list()"
                :key="siComponent.typeName"
              >
                {{ siComponent.name }}
              </v-tab>
            </v-tabs>
          </template>
        </v-toolbar>
        <v-tabs-items v-model="tab">
          <v-tab-item
            v-for="siComponent in siComponentRegistry.list()"
            :key="siComponent.typeName"
          >
            <EntityList
              :entityType="siComponent.typeName"
              :organizationId="organizationId"
              :workspaceId="workspaceId"
            >
            </EntityList>
          </v-tab-item>
        </v-tabs-items>
      </v-card>
    </v-container>
  </StandardLayout>
</template>

<script lang="ts">
import StandardLayout from "@/components/StandardLayout.vue";
import EntityList from "@/components/EntityList.vue";

import { siComponentRegistry, SiComponentRegistry } from "@/registry";

export default {
  name: "workspace",
  props: {
    organizationId: String,
    workspaceId: String,
  },
  data(): { tab: null | string; siComponentRegistry: SiComponentRegistry } {
    return {
      tab: null,
      siComponentRegistry,
    };
  },
  components: {
    StandardLayout,
    EntityList,
  },
};
</script>
