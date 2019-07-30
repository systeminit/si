<template>
  <v-card>
    <v-card-title>
      Create a Workspace
    </v-card-title>
    <v-card-text>
      <v-text-field
        required
        outlined
        v-model="name"
        label="Name"
      ></v-text-field>
      <v-text-field
        required
        outlined
        v-model="description"
        label="Description"
      ></v-text-field>
    </v-card-text>
    <v-card-actions>
      <v-spacer />
      <v-btn color="primary" @click="createWorkspace()">
        Create Workspace
      </v-btn>
    </v-card-actions>
  </v-card>
</template>

<script lang="ts">
import Vue from "vue";
import sortBy from "lodash/sortBy";

import createWorkspace from "@/graphql/mutation/createWorkspace.graphql";
import getWorkspaces from "@/graphql/queries/getWorkspaces.graphql";

export default Vue.extend({
  name: "WorkspaceCreate",
  methods: {
    createWorkspace() {
      this.$apollo.mutate({
        mutation: createWorkspace,
        variables: {
          name: this.name,
          description: this.description,
        },
        update: (store, createData) => {
          const workspace = createData.data.createWorkspace.workspace;
          const data: any = store.readQuery({
            query: getWorkspaces,
          });
          data.getWorkspaces.push(workspace);
          let nw = sortBy(data.getWorkspaces, ["name", "id"]);
          data.getWorkspaces = nw;
          store.writeQuery({
            query: getWorkspaces,
            data,
          });
          this.name = "";
          this.description = "";
          this.$router.push({
            name: "workspace",
            params: { id: workspace.id },
          });
        },
      });
    },
  },
  data() {
    return {
      name: "",
      description: "",
      errorMessage: "",
    };
  },
});
</script>
