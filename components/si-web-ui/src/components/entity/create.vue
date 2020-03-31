<template>
  <v-container class="justify-start d-flex" style="flex-direction: row">
    <v-card :loading="$apollo.loading">
      <v-alert type="error" dismissible v-if="errorMessage">{{ errorMessage }}</v-alert>
      <v-card-title>Create {{ siComponent.name }}</v-card-title>
      <v-card-text>
        <div v-for="item in siComponent.coreProperties.entries()" :key="item.name">
          <v-text-field
            v-model="coreProperties[item.name]"
            :label="item.label"
            :name="item.name"
            :rules="item.rules"
            v-if="item.kind() == 'text'"
          ></v-text-field>
        </div>
        <v-card flat>
          <v-card-title>Constraints</v-card-title>
          <v-card-text>
            <div v-for="item in siComponent.constraints.entries()" :key="item.name">
              <v-select
                v-model="constraints[item.name]"
                :label="item.label"
                :name="item.name"
                :rules="item.rules"
                :items="item.options"
                v-if="item.kind() == 'select'"
              ></v-select>
            </div>
          </v-card-text>
        </v-card>
        <v-card flat>
          <v-card-title>Properties</v-card-title>
          <v-card-text>
            <div v-for="item in siComponent.properties.entries()" :key="item.name">
              {{ item.label }}
              <codemirror
                v-model="properties[item.name]"
                :label="item.label"
                :name="item.name"
                :options="cmOptions"
              ></codemirror>
            </div>
          </v-card-text>
        </v-card>
      </v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn @click="createEntity">Create</v-btn>
      </v-card-actions>
    </v-card>
  </v-container>
</template> 

<script lang="ts">
import Vue from "vue";
import NameGenerator from "project-name-generator";

import { codemirror } from "vue-codemirror";
import "codemirror/lib/codemirror.css";
import "codemirror/theme/gruvbox-dark.css";
import "codemirror/keymap/vim.js";
import "codemirror/keymap/emacs.js";
import "codemirror/keymap/sublime.js";
import "codemirror/mode/toml/toml.js";

import { auth } from "@/auth";
import { siComponentRegistry } from "@/registry";
import { SiComponent, EntityPropDefaultValues } from "@/registry/siComponent";

interface EntityCreateData {
  errorMessage: string | null;
  siComponent: SiComponent;
  cmOptions: Object;
  coreProperties: EntityPropDefaultValues;
  constraints: EntityPropDefaultValues;
  properties: EntityPropDefaultValues;
}

export default Vue.extend({
  name: "EntityCreate",
  props: {
    entityType: String,
    organizationId: String,
    workspaceId: String,
  },
  data(): EntityCreateData {
    const newEntityName = NameGenerator.generate({ words: 2, number: true });
    const siComponent = siComponentRegistry.lookup(this.entityType);
    return {
      errorMessage: null,
      siComponent,
      cmOptions: {
        tabSize: 4,
        theme: "gruvbox-dark",
        lineNumbers: true,
        keyMap: "vim",
        mode: "text/x-toml",
        readOnly: false,
      },
      coreProperties: siComponent.coreProperties.createValueObject({
        name: newEntityName.dashed,
        displayName: newEntityName.spaced,
        description: `${siComponent.name} ${newEntityName.spaced}`,
      }),
      constraints: siComponent.constraints.createValueObject(),
      properties: siComponent.properties.createValueObject(),
    };
  },
  methods: {
    async createEntity() {
      this.errorMessage = null;
      try {
        let data = await this.$apollo.mutate({
          mutation: this.siComponent.createEntity,
          variables: {
            name: this.coreProperties["name"],
            displayName: this.coreProperties["displayName"],
            description: this.coreProperties["description"],
            workspaceId: this.workspaceId,
            constraints: this.constraints,
            props: this.siComponent.properties.realValues(this.properties),
          },
        });
        this.$router.push({
          name: "workspaceShowEntity",
          params: {
            organizationId: this.organizationId,
            workspaceId: this.workspaceId,
            entityType: this.entityType,
            entityId:
              data.data[this.siComponent.createEntityResultString()].entity.id,
          },
        });
      } catch (error) {
        this.errorMessage = `Create error: ${error.message}`;
      }
    },
  },
  components: {
    codemirror,
  },
});
</script>
