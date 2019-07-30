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
            </v-card-text>
          </v-card>
        </v-col>
      </v-row>
    </v-container>
  </div>
</template>

<script lang="ts">
import Vue from "vue";

import getWorkspaceById from "@/graphql/queries/getWorkspaceById.graphql";
import getWorkspaces from "@/graphql/queries/getWorkspaces.graphql";
import deleteWorkspace from "@/graphql/mutation/deleteWorkspace.graphql";

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
});
</script>
