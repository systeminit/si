<template>
  <div>
    <v-container class="justify-start d-flex" style="flex-direction: row;">
      <v-row no-gutters class="justify-start">
        <v-col cols="8" class="flex-grow-0 flex-shrink-0">
          <v-row no-gutters class="flex-grow-0">
            <v-col cols="12">
              <v-card :loading="loading">
                <v-card-title>Create {{ entityName }}</v-card-title>
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
                  <v-btn @click="resetComponentState">
                    Reset
                  </v-btn>
                  <v-btn @click="createEntity" :disabled="cmOptions.readOnly">
                    Create
                  </v-btn>
                </v-card-actions>
              </v-card>
            </v-col>
          </v-row>
          <v-row no-gutters class="mt-2 flex-grow-0">
            <v-col cols="12">
              <v-tabs fixed-tabs v-model="tab">
                <v-tab key="resolution">Resolution</v-tab>
                <v-tab key="output">Output</v-tab>
                <v-tab key="entity">Entity</v-tab>
              </v-tabs>
              <v-tabs-items v-model="tab">
                <v-tab-item key="resolution">
                  <v-card v-if="checkComponent">
                    <v-card-title>
                      {{ checkComponent.component.displayName }}
                    </v-card-title>
                    <v-card-text class="d-flex float">
                      <v-card class="ma-2">
                        <v-card-title>Properties</v-card-title>
                        <v-card-text>
                          <ul>
                            <li>bits: {{ checkComponent.component.bits }}</li>
                            <li>
                              keyFormat:
                              {{ checkComponent.component.keyFormat }}
                            </li>
                            <li>
                              keyType: {{ checkComponent.component.keyType }}
                            </li>
                          </ul>
                        </v-card-text>
                      </v-card>
                      <v-card class="ma-2">
                        <v-card-title>Implicit Constraints</v-card-title>
                        <v-card-text>
                          <ul>
                            <li
                              v-for="item in checkComponent.implicitConstraints"
                              v-bind:key="item.field"
                            >
                              {{ item.field }}: {{ item.value }}
                            </li>
                          </ul>
                        </v-card-text>
                      </v-card>
                      <v-card class="ma-2 flex-grow-1">
                        <v-card-title>Parse Errors</v-card-title>
                        <v-card-text v-if="inputData.error">
                          {{ inputData.error }}
                        </v-card-text>
                        <v-card-text v-else>
                          No parse errors! Congratulations, you TOML master you.
                        </v-card-text>
                      </v-card>
                    </v-card-text>
                  </v-card>
                  <v-card v-else>
                    <v-card-title>Solving default component...</v-card-title>
                  </v-card>
                </v-tab-item>
                <v-tab-item key="output">
                  <v-card
                    v-if="streamEntityEvent"
                    :loading="!streamEntityEvent.finalized"
                  >
                    <v-card-title>
                      {{ streamEntityEvent.typeName }}
                      {{ streamEntityEvent.actionName }}
                    </v-card-title>
                    <v-card-text class="d-flex float">
                      <v-card class="ma-2">
                        <v-card-title>{{
                          streamEntityEvent.entityId
                        }}</v-card-title>
                        <v-card-text>
                          <ul>
                            <li>Entity ID: {{ streamEntityEvent.entityId }}</li>
                            <li>Success: {{ streamEntityEvent.success }}</li>
                            <li>
                              Create Time: {{ streamEntityEvent.createTime }}
                            </li>
                            <li>
                              Updated Time: {{ streamEntityEvent.updatedTime }}
                            </li>
                            <li>
                              Final Time: {{ streamEntityEvent.finalTime }}
                            </li>
                          </ul>
                        </v-card-text>
                      </v-card>
                      <v-card class="ma-2 flex-grow-1">
                        <v-card-title>Agent</v-card-title>
                        <v-card-text>
                          <v-tabs v-model="agentOutputTab" fixed-tabs>
                            <v-tab>Output</v-tab>
                            <v-tab>Error</v-tab>
                          </v-tabs>
                          <v-tabs-items v-model="agentOutputTab">
                            <v-tab-item>
                              <v-textarea
                                no-resize
                                outlined
                                full-width
                                flat
                                readonly
                                :value="
                                  streamEntityEvent.outputLines.join('\n')
                                "
                              >
                              </v-textarea>
                            </v-tab-item>
                            <v-tab-item>
                              <v-textarea
                                no-resize
                                outlined
                                full-width
                                flat
                                readonly
                                :value="streamEntityEvent.errorLines.join('\n')"
                              >
                              </v-textarea>
                            </v-tab-item>
                          </v-tabs-items>
                        </v-card-text>
                      </v-card>
                    </v-card-text>
                  </v-card>
                  <v-card v-else>
                    <v-card-title>No Entity Events</v-card-title>
                    <v-card-text>Maybe hit the "Create" button?</v-card-text>
                  </v-card>
                </v-tab-item>
                <v-tab-item key="entity">
                  <div
                    v-if="streamEntityEvent && streamEntityEvent.outputEntity"
                  >
                    <EntityShow :entityId="streamEntityEvent.outputEntity.id" />
                  </div>
                  <v-card v-else>
                    <v-card-title>No Entity</v-card-title>
                    <v-card-text>
                      No Entity yet; maybe hit the "Create" button?
                    </v-card-text>
                  </v-card>
                </v-tab-item>
              </v-tabs-items>
            </v-col>
          </v-row>
        </v-col>
        <v-col cols="4">
          <v-card class="ml-2" height="100%">
            <v-card-title>Hints</v-card-title>
            <v-card-text>
              <h3>Constraints</h3>
              <ul>
                <li>constraints.keyType = "RSA" | "DSA" | "ED25519"</li>
                <li>constraints.keyFormat = "RFC4716" | "PKCS8" | "PEM"</li>
                <li>constraints.bits = Number</li>
              </ul>
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
import "codemirror/mode/toml/toml.js";
import TOML from "@iarna/toml";
import NameGenerator from "project-name-generator";

import { auth } from "@/auth";
import workspaceList from "@/graphql/queries/workspaceList.gql";
import pickComponent from "@/graphql/queries/pickComponent.gql";
import createEntityMutation from "@/graphql/mutations/createEntity.gql";
import streamEntityEvents from "@/graphql/subscription/streamEntityEvents.gql";
import {
  SshKeyKeyType,
  SshKeyPickComponentReply,
  SshKeyCreateEntityReply,
  SshKeyEntityEvent,
  SshKeyCreateEntityRequest,
  StreamEntityEventsRequest,
  StreamEntityEventsSubscription,
  SshKeyImplicitConstraint,
} from "@/graphql-types";
import EntityShow from "@/components/EntityShow.vue";
import { SshKeyPickComponentRequest } from "../graphql-funtimes";

interface DataField {
  code: string;
  tab: null | number;
  agentOutputTab: null | number;
  createEntityData: null | SshKeyCreateEntityReply;
  checkComponent: null | SshKeyPickComponentReply;
  streamEntityEvent: null | SshKeyEntityEvent;
  cmOptions: {
    tabSize: number;
    theme: "gruvbox-dark";
    lineNumbers: boolean;
    keyMap: "vim" | "emacs" | "sublime";
    mode: "text/x-toml";
    readOnly: boolean;
  };
  loading: boolean;
}

interface InputData {
  parsed: null | SshKeyCreateEntityRequest;
  error: string;
}

export default Vue.extend({
  name: "Editor",
  props: {
    entityName: String,
  },
  data(): DataField {
    const entityName = NameGenerator.generate({ words: 4, number: true });
    return {
      code: `name = "${entityName.dashed}"
displayName = "${entityName.spaced}"
description = "SSH Key ${entityName.spaced}"`,
      tab: null,
      agentOutputTab: null,
      createEntityData: null,
      checkComponent: null,
      streamEntityEvent: null,
      cmOptions: {
        tabSize: 4,
        theme: "gruvbox-dark",
        lineNumbers: true,
        keyMap: "vim",
        mode: "text/x-toml",
        readOnly: false,
      },
      loading: false,
    };
  },
  methods: {
    resetComponentState(): void {
      const entityName = NameGenerator.generate({ words: 4, number: true });
      this.createEntityData = null;
      this.streamEntityEvent = null;
      this.cmOptions["readOnly"] = false;
      this.tab = 0;
      this.code = `name = "${entityName.dashed}"
displayName = "${entityName.spaced}"
description = "SSH Key ${entityName.spaced}"`;
    },
    onCmCodeChange(newCode: string): void {
      this.code = newCode;
    },
    async createEntity(): Promise<void> {
      this.loading = true;
      this.tab = 1;
      this.cmOptions["readOnly"] = true;
      let inputData;
      if (this.inputData.parsed) {
        inputData = this.inputData.parsed;
      } else {
        inputData = {};
      }
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
    inputData(): InputData {
      try {
        let objectData = TOML.parse(this.code);
        return {
          parsed: objectData,
          error: "",
        };
      } catch (err) {
        return {
          parsed: null,
          error: `${err}`,
        };
      }
    },
  },
  apollo: {
    $subscribe: {
      entityEvents: {
        query: streamEntityEvents,
        variables(): StreamEntityEventsRequest {
          const workspace = auth.getCurrentWorkspace();
          return {
            workspaceId: workspace.id || "",
          };
        },
        result({ data }: { data: StreamEntityEventsSubscription }): void {
          this.streamEntityEvent = data["streamEntityEvents"];
          if (this.streamEntityEvent && this.streamEntityEvent.finalized) {
            this.tab = 2;
          }
        },
      },
    },
    checkComponent: {
      query: pickComponent,
      update: data => {
        data.sshKeyPickComponent.implicitConstraints.sort(function(
          a: SshKeyImplicitConstraint,
          b: SshKeyImplicitConstraint,
        ) {
          if (!a || !a["field"] || !b || !b["field"]) {
            return 0;
          }
          if (a.field < b.field) {
            return -1;
          } else if (a.field > b.field) {
            return 1;
          } else {
            return 0;
          }
        });
        return data.sshKeyPickComponent;
      },
      variables(): SshKeyPickComponentRequest {
        let inputData = this.inputData;
        console.log("my input data", inputData);
        if (inputData.parsed && inputData.parsed["constraints"]) {
          const inputConstraints = inputData.parsed["constraints"];
          return {
            keyType: inputConstraints["keyType"],
            keyFormat: inputConstraints["keyFormat"],
            bits: inputConstraints["bits"],
          };
        } else {
          return {};
        }
      },
    },
  },
  components: {
    codemirror,
    EntityShow,
  },
});
</script>
