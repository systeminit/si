<template>
  <div>
    <v-container>
      <v-layout align-center justify-center>
        <v-flex xl12>
          <v-card md8>
            <v-card-title dark class="secondary">
              <v-btn :to="{ name: 'workspaces' }" dark flat>
                <v-icon>arrow_back</v-icon>
              </v-btn>
              <span class="headline white--text">
                Workspace {{ getWorkspaceById.name }}
              </span>
              <v-spacer />
              <v-btn dark flat @click="deleteWorkspace()">
                <v-icon>delete</v-icon>
              </v-btn>
            </v-card-title>
            <v-divider />
            <v-card-text>
              {{ getWorkspaceById.description }}
            </v-card-text>
          </v-card>
        </v-flex>
      </v-layout>
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
