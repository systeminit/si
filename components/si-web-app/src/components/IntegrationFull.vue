<template>
  <div>
    <v-container>
      <v-row align="center" justify="center">
        <v-col xl="12">
          <v-card md8>
            <v-card-title>
              <v-btn :to="{ name: 'integrations' }" text active-class exact>
                <v-icon>arrow_back</v-icon>
              </v-btn>
              Integration / {{ getIntegrationInstanceById.name }}
              <v-spacer />
              <v-dialog v-model="deleteDialog">
                <template v-slot:activator="{ on }">
                  <v-btn text v-on="on">
                    <v-icon>delete</v-icon>
                  </v-btn>
                </template>
                <v-card>
                  <v-card-title>
                    Delete Integration
                  </v-card-title>
                  <v-divider />
                  <v-card-text>
                    Really delete {{ getIntegrationInstanceById.name }}? This
                    cannot be un-done.
                  </v-card-text>
                  <v-card-actions>
                    <v-spacer></v-spacer>
                    <v-btn text @click="deleteDialog = false">
                      Cancel
                    </v-btn>
                    <v-btn text @click="deleteIntegrationInstance()">
                      Delete
                    </v-btn>
                  </v-card-actions>
                </v-card>
              </v-dialog>
            </v-card-title>
            <v-divider />
            <v-card-text>
              <v-img
                height="100"
                contain
                :src="
                  require(`../assets/${
                    getIntegrationInstanceById.integration.image
                  }`)
                "
              ></v-img>

              {{ getIntegrationInstanceById.description }}

              <v-divider></v-divider>

              <h2>Enable/Disable on Workspaces</h2>
              <v-list>
                <IntegrationToggle
                  v-for="workspace in getWorkspacesWithIntegrationInstances"
                  :key="workspace.id"
                  :integrationInstanceId="integrationInstanceId"
                  :workspaceId="workspace.id"
                  :title="workspace.name"
                  :initialEnabled="isWorkspaceEnabled(workspace)"
                />
              </v-list>
            </v-card-text>
          </v-card>
        </v-col>
      </v-row>
    </v-container>
  </div>
</template>

<script lang="ts">
import Vue from "vue";

import IntegrationToggle from "@/components/IntegrationToggle.vue";

import getWorkspacesWithIntegrationInstances from "@/graphql/queries/getWorkspacesWithIntegrationInstances.graphql";
import getIntegrationInstances from "@/graphql/queries/getIntegrationInstances.graphql";
import getIntegrationInstanceById from "@/graphql/queries/getIntegrationInstanceById.graphql";
import deleteIntegrationInstance from "@/graphql/mutation/deleteIntegrationInstance.graphql";

import { Workspace } from "@/graphql/types";

export default Vue.extend({
  name: "IntegrationFull",
  apollo: {
    getIntegrationInstanceById: {
      query: getIntegrationInstanceById,
      variables() {
        return {
          id: this.integrationInstanceId,
        };
      },
    },
    getWorkspacesWithIntegrationInstances: {
      query: getWorkspacesWithIntegrationInstances,
      update: data => data.getWorkspaces,
    },
  },
  data() {
    return {
      getIntegrationInstanceById: {
        name: "Loading...",
        description: "Loading...",
      },
      getWorkspacesWithIntegrationInstances: [],
      message: false,
      deleteDialog: false,
    };
  },
  methods: {
    isWorkspaceEnabled(workspace: Workspace) {
      let found = workspace.integrationInstances.find(e => {
        return e.id == this.integrationInstanceId;
      });
      return found === undefined ? false : true;
    },
    deleteIntegrationInstance() {
      this.$apollo.mutate({
        mutation: deleteIntegrationInstance,
        variables: {
          id: this.integrationInstanceId,
        },
        update: (store, deleteData) => {
          let deleted_id =
            deleteData.data.deleteIntegrationInstance.integrationInstance.id;
          if (deleted_id) {
            // First, read the right data from the cache
            const data: any = store.readQuery({
              query: getIntegrationInstances,
            });
            // Then, remove our deleted entry from the cache
            data.getIntegrationInstances = data.getIntegrationInstances.filter(
              (i: any) => {
                return i.id != deleted_id;
              },
            );
            // Then, write the new list back to the cache
            store.writeQuery({
              query: getIntegrationInstances,
              data,
            });
            this.deleteDialog = false;
            this.$router.push({ name: "integrations" });
          }
        },
      });
    },
  },
  props: {
    integrationInstanceId: String,
  },
  components: {
    IntegrationToggle,
  },
});
</script>
