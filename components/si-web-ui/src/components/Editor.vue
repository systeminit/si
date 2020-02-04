<template>
  <div>
    <v-container class="justify-start d-flex" style="flex-direction: row;">
      <v-row no-gutters class="justify-start">
        <v-col cols="2" class="flex-grow-0 flex-shrink-0">
          <v-card class="mr-2" height="100%">
            <v-card-title>Workspace</v-card-title>
            <v-card-text>
              <v-treeview
                :items="workspaceList"
                activatable
                hoverable
                color="secondary"
                dense
                :active.sync="active"
                transition
              >
              </v-treeview>
            </v-card-text>
          </v-card>
        </v-col>
        <v-col cols="8" class="flex-grow-1 flex-shrink-0">
          <v-row no-gutters>
            <v-toolbar>
              <v-toolbar-title>SSH Key</v-toolbar-title>
              <v-spacer></v-spacer>
              <v-btn icon>
                <v-icon>mdi-magnify</v-icon>
              </v-btn>
              <v-btn icon>
                <v-icon>mdi-plus</v-icon>
              </v-btn>
            </v-toolbar>
          </v-row>
          <v-row no-gutters class="mt-2 flex-grow-1">
            <v-col cols="12">
              <v-card :loading="loading">
                <v-card-text>
                  <codemirror
                    :value="code"
                    :options="cmOptions"
                    @input="onCmCodeChange"
                  >
                  </codemirror>
                </v-card-text>
                <v-card-actions>
                  <v-spacer></v-spacer>
                  <v-btn @click="createEntity">
                    Create
                  </v-btn>
                </v-card-actions>
              </v-card>
            </v-col>
          </v-row>
          <v-row no-gutters class="mt-2 flex-grow-1">
            <v-col cols="12">
              <v-card>
                <v-card-title>
                  Output
                </v-card-title>
                <v-card-text>
                  <codemirror
                    :value="outputAsYAML"
                    :options="cmOutputOptions"
                  ></codemirror>
                </v-card-text>
              </v-card>
            </v-col>
          </v-row>
        </v-col>
        <v-col cols="2">
          <v-card class="ml-2" height="100%">
            <v-card-title>Hints</v-card-title>
            <v-card-text>
              <div v-if="!selected"></div>
              <div v-else>
                {{ selected.name }}
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
import { codemirror } from "vue-codemirror";
import "codemirror/lib/codemirror.css";
import "codemirror/theme/gruvbox-dark.css";
import "codemirror/keymap/vim.js";
import "codemirror/keymap/emacs.js";
import "codemirror/keymap/sublime.js";
import "codemirror/mode/yaml/yaml.js";
import YAML from "yaml";
import NameGenerator from "project-name-generator";

import { auth } from "@/auth";
import workspaceList from "@/graphql/queries/workspaceList.gql";
import pickComponent from "@/graphql/queries/pickComponent.gql";
import createEntityMutation from "@/graphql/mutations/createEntity.gql";
import streamEntityEvents from "@/graphql/subscription/streamEntityEvents.gql";
import { SshKeyKeyType } from "../graphql-types";

export default Vue.extend({
  name: "Editor",
  props: {
    entityName: String,
  },
  data() {
    const entityName = NameGenerator.generate({ words: 4, number: true });
    return {
      code: `name: ${entityName.dashed}
displayName: ${entityName.spaced}
description: SSH Key ${entityName.spaced}
constraints:
    # Your constraints here
      `,
      codeOutput: "# Nothing yet!",
      active: [],
      createEntityData: {},
      checkComponent: {
        component: {},
        implicitConstraints: [],
      },
      streamEntityEvent: {},
      workspaceList: [
        {
          id: 1,
          name: "Entities",
          children: [
            {
              id: 1,
              name: "SSH Key",
              children: [{ id: 2, name: "2048 bit RSA" }],
            },
          ],
        },
        {
          id: 2,
          name: "Components",
          children: [
            {
              id: 1,
              name: "SSH Key",
              children: [{ id: 2, name: "2048 bit RSA" }],
            },
          ],
        },
      ],
      cmOptions: {
        tabSize: 4,
        theme: "gruvbox-dark",
        lineNumbers: true,
        keyMap: "vim",
        mode: "text/x-yaml",
      },
      cmOutputOptions: {
        tabSize: 4,
        theme: "gruvbox-dark",
        lineNumbers: true,
        mode: "text/x-yaml",
        keyMap: "vim",
        readOnly: true,
      },
      loading: false,
    };
  },
  methods: {
    onCmCodeChange(newCode: String): void {
      console.log("I got new code!", newCode);
      this.code = newCode;
    },
    async createEntity(): Promise<void> {
      this.loading = true;
      let inputData = this.inputData;
      const workspace = auth.getCurrentWorkspace();
      inputData["workspaceId"] = workspace.id;
      let data = await this.$apollo.mutate({
        mutation: createEntityMutation,
        variables: inputData,
      });
      console.log("alice in chains says yes", data);
      this.createEntityData = data.data["sshKeyCreateEntity"];
      this.loading = false;
    },
  },
  computed: {
    inputData(): any {
      try {
        let objectData = YAML.parse(this.code);
        console.log("objectData:", objectData);
        return objectData;
      } catch (err) {
        console.log("not today, homie", err);
        return {};
      }
    },
    outputAsYAML(): String {
      if (this.streamEntityEvent["id"]) {
        return YAML.stringify(this.streamEntityEvent);
      } else if (this.createEntityData["entity"]) {
        return YAML.stringify(this.createEntityData);
      } else {
        return YAML.stringify(this.checkComponent);
      }
    },
    selected(): undefined | {} {
      console.log(this.active);
      if (!this.active.length) return undefined;
      const thingy = this.active[0];
      return thingy;
    },
  },
  apollo: {
    $subscribe: {
      entityEvents: {
        query: streamEntityEvents,
        variables() {
          const workspace = auth.getCurrentWorkspace();
          return {
            workspaceId: workspace.id,
          };
        },
        result({ data }) {
          this.streamEntityEvent = data["streamEntityEvents"];
        },
      },
    },
    checkComponent: {
      query: pickComponent,
      update: data => data.sshKeyPickComponent,
      variables() {
        let inputData = this.inputData;
        console.log("my input data", inputData);
        if (inputData["constraints"]) {
          const inputConstraints = inputData["constraints"];
          return {
            keyType: inputConstraints["keyType"],
            keyFormat: inputConstraints["keyFormat"],
            bits: inputConstraints["bits"],
          };
        } else {
          return {};
        }
      },
      skip(): boolean {
        let inputData = this.inputData;

        if (!inputData["constraints"]) {
          return true;
        } else if (
          inputData["constraints"]["keyType"] ||
          inputData["constraints"]["keyFormat"] ||
          inputData["constraints"]["bits"]
        ) {
          return false;
        } else {
          return true;
        }
      },
    },
    workspaceList: {
      query: workspaceList,
      update: data => {
        return [
          {
            id: "All-Entities",
            name: "Entities",
            children: [
              {
                id: "All-Ssh-Key-Entities",
                name: "SSH Key",
                children: data["sshKeyListEntities"]["items"],
              },
            ],
          },
          {
            id: "All-Components",
            name: "Components",
            children: [
              {
                id: "All-Ssh-Key-Components",
                name: "SSH Key",
                children: data["sshKeyListComponents"]["items"],
              },
            ],
          },
        ];
      },
    },
  },
  components: {
    codemirror,
  },
});
</script>
