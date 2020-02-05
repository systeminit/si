<template>
  <v-container class="justify-start d-flex" sytle="flex-direction: row;">
    <v-row no-gutters class="justify-start">
      <v-col cols="12">
        <v-card cols="12">
          <v-card-title>{{ sshKeyGetEntity.displayName }}</v-card-title>
          <v-card-subtitle>SSH Key</v-card-subtitle>
          <v-card-text>
            <v-list>
              <v-list-item>
                <v-list-item-content>
                  <v-list-item-title>
                    Key Type
                  </v-list-item-title>
                  {{ sshKeyGetEntity.keyType }}
                </v-list-item-content>
              </v-list-item>
              <v-list-item>
                <v-list-item-content>
                  <v-list-item-title>
                    Key Format
                  </v-list-item-title>
                  {{ sshKeyGetEntity.keyFormat }}
                </v-list-item-content>
              </v-list-item>
              <v-list-item>
                <v-list-item-content>
                  <v-list-item-title>
                    Bits
                  </v-list-item-title>
                  {{ sshKeyGetEntity.bits }}
                </v-list-item-content>
              </v-list-item>
              <v-list-item>
                <v-list-item-content>
                  <v-list-item-title>
                    Public Key
                  </v-list-item-title>
                  <v-col cols="4">
                    <codemirror
                      :value="sshKeyGetEntity.publicKey"
                      :options="cmOutputOptions"
                    >
                    </codemirror>
                  </v-col>
                </v-list-item-content>
              </v-list-item>
            </v-list>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script lang="ts">
import Vue from "vue";
import { codemirror } from "vue-codemirror";
import "codemirror/lib/codemirror.css";
import "codemirror/theme/gruvbox-dark.css";
import "codemirror/keymap/vim.js";

import sshKeyGetEntity from "@/graphql/queries/sshKeyGetEntity.gql";
import { Query, SshKeyEntity, SshKeyGetEntityRequest } from "@/graphql-types";

interface DataField {
  sshKeyGetEntity: SshKeyEntity;
  loading: boolean;
  cmOutputOptions: {
    tabSize: number;
    theme: "gruvbox-dark";
    mode: "text/plain";
    lineNumbers: true;
    keyMap: "vim";
    readOnly: true;
    lineWrapping: true;
  };
}

export default Vue.extend({
  name: "EntityShow",
  props: {
    entityId: String,
  },
  data(): DataField {
    return {
      cmOutputOptions: {
        tabSize: 4,
        theme: "gruvbox-dark",
        lineNumbers: true,
        lineWrapping: true,
        mode: "text/plain",
        keyMap: "vim",
        readOnly: true,
      },
      sshKeyGetEntity: {},
      loading: true,
    };
  },
  apollo: {
    sshKeyGetEntity: {
      query: sshKeyGetEntity,
      update(data: Query): SshKeyEntity {
        this.loading = false;
        if (data["sshKeyGetEntity"] && data["sshKeyGetEntity"]["entity"]) {
          return data.sshKeyGetEntity.entity;
        } else {
          console.log("Wha happa?");
          return {};
        }
      },
      variables(): SshKeyGetEntityRequest {
        return {
          entityId: this.entityId,
        };
      },
    },
  },
  components: {
    codemirror,
  },
});
</script>
