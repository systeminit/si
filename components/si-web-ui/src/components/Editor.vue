<template>
  <div>
    <v-container class="justify-start d-flex" style="flex-direction: row;">
      <v-row no-gutters class="justify-start">
        <v-col cols="8" class="flex-grow-0 flex-shrink-0">
          <v-row no-gutters class="flex-grow-0">
            <v-col cols="12">
              <v-card :loading="loading">
                <v-card-title>Create {{ siComponent.name }}</v-card-title>
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
                            <li
                              v-for="cprop in siComponent.componentProperties"
                              v-bind:key="cprop"
                            >
                              {{ cprop }}: {{ checkComponent.component[cprop] }}
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
                    <EntityShow
                      :entityType="entityType"
                      :entityId="streamEntityEvent.outputEntity.id"
                    />
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
                <li
                  v-for="hint in siComponent.hints"
                  v-bind:key="hint.constraintName"
                >
                  constraints.{{ hint.constraintName }} = {{ hint.hintValue }}
                </li>
              </ul>
            </v-card-text>
          </v-card>
        </v-col>
      </v-row>
    </v-container>
  </div>
</template>

<script lang="js">
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
import gql from "graphql-tag";
import { DocumentNode } from "graphql";

import { auth } from "@/auth";
import { siComponentRegistry } from "@/registry";
import EntityShow from "@/components/EntityShow.vue";
import { SiComponent } from "../registry/siComponent";

export default Vue.extend({
  name: "Editor",
  props: {
    entityType: String,
  },
  data() {
    const newEntityName = NameGenerator.generate({ words: 4, number: true });
    const siComponent = siComponentRegistry.lookup(this.entityType);
    return {
      code: `name = "${newEntityName.dashed}"
displayName = "${newEntityName.spaced}"
description = "${siComponent.name} ${newEntityName.spaced}"`,
      tab: null,
      agentOutputTab: null,
      createEntityData: null,
      checkComponent: null,
      streamEntityEvent: null,
      siComponent,
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
  watch: {
    $route: "resetComponentState",
  },
  methods: {
    resetComponentState() {
      const newEntityName = NameGenerator.generate({ words: 4, number: true });
      const siComponent = siComponentRegistry.lookup(this.entityType);
      this.siComponent = siComponent;
      this.loading = false;
      this.createEntityData = null;
      this.streamEntityEvent = null;
      this.checkComponent = null;
      this.cmOptions["readOnly"] = false;
      this.tab = 0;
      this.code = `name = "${newEntityName.dashed}"
displayName = "${newEntityName.spaced}"
description = "${siComponent.name} ${newEntityName.spaced}"`;
      this.$apollo.queries.checkComponent.refresh();
      this.$apollo.subscriptions.entityEvents.refresh();
    },
    onCmCodeChange(newCode) {
      this.code = newCode;
    },
    async createEntity() {
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

      let siComponent = siComponentRegistry.lookup(this.entityType);

      let data = await this.$apollo.mutate({
        mutation: siComponent.createEntity,
        variables: inputData,
      });
      this.createEntityData = data.data[siComponent.createEntityResultString()];
      this.loading = false;
    },
  },
  computed: {
    inputData() {
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
        query() {
          let siComponent = siComponentRegistry.lookup(this.entityType);
          return siComponent.streamEntityEvents;
        },
        variables() {
          const workspace = auth.getCurrentWorkspace();
          return {
            workspaceId: workspace.id || "",
          };
        },
        result({ data }) {
          let siComponent = siComponentRegistry.lookup(this.entityType);
          this.streamEntityEvent =
            data[siComponent.streamEntityEventsResultString()];
          if (this.streamEntityEvent && this.streamEntityEvent.finalized) {
            this.tab = 2;
          }
        },
      },
    },
    checkComponent: {
      query() {
        let siComponent = siComponentRegistry.lookup(this.entityType);
        return siComponent.pickComponent;
      },
      update(data) {
        let siComponent = siComponentRegistry.lookup(this.entityType);
        const pickResultString = siComponent.pickComponentResultString();
        data[pickResultString].implicitConstraints.sort(function(a, b,) {
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
        return data[pickResultString];
      },
      variables() {
        let inputData = this.inputData;
        if (inputData.parsed && inputData.parsed["constraints"]) {
          return { input: inputData.parsed["constraints"] };
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
