<template>
  <div>
    <v-container>
      <v-row align="center" justify="center">
        <v-col xl="12">
          <v-card md8 flat>
            <v-toolbar secondary flat>
              <v-toolbar-title>Workspaces</v-toolbar-title>
              <template v-slot:extension>
                <v-tabs v-model="tabs" align-with-title>
                  <v-tab>
                    <v-icon>list</v-icon>
                  </v-tab>
                  <v-tab>
                    <v-icon>add</v-icon>
                  </v-tab>
                </v-tabs>
              </template>
            </v-toolbar>
            <v-divider />
            <v-tabs-items v-model="tabs">
              <v-tab-item key="0">
                <v-card-text v-if="getWorkspaces.length > 0">
                  <v-container>
                    <v-row align="center">
                      <v-col
                        v-for="workspace in getWorkspaces"
                        :key="workspace.id"
                        md="6"
                      >
                        <WorkspaceCard
                          :id="workspace.id"
                          :name="workspace.name"
                          :description="workspace.description"
                        />
                      </v-col>
                    </v-row>
                  </v-container>
                </v-card-text>
                <v-card-text v-else>
                  <v-container>
                    <v-row align="center" justify="center">
                      <v-col md="6">
                        <v-card>
                          <v-card-title>
                            <h2>You don't have access to any workspaces!</h2>
                          </v-card-title>
                          <v-card-text>
                            You haven't created any workspaces, and nobody else
                            has added you to any yet.
                          </v-card-text>
                          <v-card-actions>
                            <v-spacer />
                            <v-btn @click="tabs = 1" color="primary" text>
                              Create a new Workspace
                            </v-btn>
                          </v-card-actions>
                        </v-card>
                      </v-col>
                    </v-row>
                  </v-container>
                </v-card-text>
              </v-tab-item>
              <v-tab-item key="1">
                <v-card-text>
                  <WorkspaceCreate />
                </v-card-text>
              </v-tab-item>
            </v-tabs-items>
          </v-card>
        </v-col>
      </v-row>
    </v-container>
  </div>
</template>

<script lang="ts">
import Vue from "vue";

import WorkspaceCard from "@/components/WorkspaceCard.vue";
import WorkspaceCreate from "@/components/WorkspaceCreate.vue";
import getWorkspaces from "@/graphql/queries/getWorkspaces.graphql";

export default Vue.extend({
  name: "WorkspaceTop",
  apollo: {
    getWorkspaces: {
      query: getWorkspaces,
      fetchPolicy: "cache-and-network",
    },
  },
  data() {
    return {
      getWorkspaces: [],
      tabs: 0,
    };
  },
  components: {
    WorkspaceCard,
    WorkspaceCreate,
  },
});
</script>
