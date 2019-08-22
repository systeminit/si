<template>
  <div>
    <v-container>
      <v-row align="center" justify="center">
        <v-col xl="12">
          <v-card md8>
            <v-card-title>
              <v-btn :to="{ name: 'workspaces' }" text active-class exact>
                <v-icon>arrow_back</v-icon>
              </v-btn>
              Workspace / {{ getWorkspaceById.name }}
              <v-spacer />
              <v-dialog v-model="deleteDialog">
                <template v-slot:activator="{ on }">
                  <v-btn text v-on="on">
                    <v-icon>delete</v-icon>
                  </v-btn>
                </template>
                <v-card>
                  <v-card-title>
                    Delete Workspace
                  </v-card-title>
                  <v-divider />
                  <v-card-text>
                    Really delete {{ getWorkspaceById.name }}? This cannot be
                    un-done.
                  </v-card-text>
                  <v-card-actions>
                    <v-spacer></v-spacer>
                    <v-btn text @click="deleteDialog = false">
                      Cancel
                    </v-btn>
                    <v-btn text @click="deleteWorkspace()">
                      Delete
                    </v-btn>
                  </v-card-actions>
                </v-card>
              </v-dialog>
            </v-card-title>
            <v-divider />
            <v-card-text>
              {{ getWorkspaceById.description }}

              <ComponentTable :workspaceId="workspace_id"></ComponentTable>

              <v-divider />
              <div v-if="getIntegrationInstances.length != 0">
                <h3>Enable/Disable Integrations</h3>
                <v-list>
                  <IntegrationToggle
                    v-for="integrationInstance in getIntegrationInstances"
                    :key="integrationInstance.id"
                    :integrationInstanceId="integrationInstance.id"
                    :workspaceId="getWorkspaceById.id"
                    :title="integrationInstance.name"
                    :integrationName="integrationInstance.integration.name"
                    :initialEnabled="
                      isIntegrationInstanceEnabled(integrationInstance)
                    "
                  />
                </v-list>
              </div>
              <div v-else>
                There are no integrations yet; perhaps add one?
              </div>
            </v-card-text>
          </v-card>
        </v-col>
      </v-row>
    </v-container>
  </div>
</template>

<script lang="ts">
import Vue from "vue";

import ComponentTable from "@/components/ComponentTable.vue";
import IntegrationToggle from "@/components/IntegrationToggle.vue";

import getIntegrationInstances from "@/graphql/queries/getIntegrationInstances.graphql";
import getWorkspaceById from "@/graphql/queries/getWorkspaceById.graphql";
import getWorkspaces from "@/graphql/queries/getWorkspaces.graphql";
import deleteWorkspace from "@/graphql/mutation/deleteWorkspace.graphql";

import { IntegrationInstance } from "@/graphql/types";

export default Vue.extend({
  name: "WorkspaceFull",
  apollo: {
    getWorkspaceById: {
      query: getWorkspaceById,
      variables() {
        return {
          id: this.workspace_id,
        };
      },
    },
    getIntegrationInstances: {
      query: getIntegrationInstances,
    },
  },
  data() {
    return {
      getWorkspaceById: {
        name: "Loading",
        description: "Loading",
      },
      deleteDialog: false,
    };
  },
  methods: {
    isIntegrationInstanceEnabled(integrationInstance: IntegrationInstance) {
      console.log(integrationInstance);
      let found = integrationInstance.workspaces.find(w => {
        return w.id == this.workspace_id;
      });
      return found === undefined ? false : true;
    },
    deleteWorkspace() {
      this.$apollo.mutate({
        mutation: deleteWorkspace,
        variables: {
          id: this.workspace_id,
        },
        update: (store, deleteData) => {
          let deleted_id = deleteData.data.deleteWorkspace.workspace.id;

          if (deleted_id) {
            // First, read the right data from the cache
            const data: any = store.readQuery({
              query: getWorkspaces,
            });
            // Then, remove our deleted entry from the cache
            data.getWorkspaces = data.getWorkspaces.filter((w: any) => {
              return w.id != deleted_id;
            });
            // Then, write the new list back to the cache
            store.writeQuery({
              query: getWorkspaces,
              data,
            });
            this.deleteDialog = false;
            this.$router.push({ name: "workspaces" });
          }
        },
      });
    },
  },
  props: {
    workspace_id: String,
  },
  components: {
    IntegrationToggle,
    ComponentTable,
  },
});
</script>
