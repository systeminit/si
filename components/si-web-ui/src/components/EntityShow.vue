<template>
  <v-container class="fluid">
    <v-card>
      <v-card-title>{{ sshKeyGetEntity.displayName }}</v-card-title>
      <v-card-subtitle>SSH Key</v-card-subtitle>
      <v-card-text>
        <v-btn-toggle>
          <v-btn>Rotate</v-btn>
          <v-btn>Replace</v-btn>
          <v-btn>Clone</v-btn>
          <v-btn>Delete</v-btn>
        </v-btn-toggle>
        <v-list flat>
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
              <v-textarea
                no-resize
                outlined
                full-width
                flat
                single-line
                readonly
                :value="sshKeyGetEntity.publicKey"
              >
              </v-textarea>
            </v-list-item-content>
          </v-list-item>
        </v-list>
      </v-card-text>
    </v-card>
  </v-container>
</template>

<script lang="ts">
import Vue from "vue";

import sshKeyGetEntity from "@/graphql/queries/sshKeyGetEntity.gql";
import { Query, SshKeyEntity, SshKeyGetEntityRequest } from "@/graphql-types";

interface DataField {
  sshKeyGetEntity: SshKeyEntity;
  loading: boolean;
  cmOutputOptions: {
    tabSize: number;
    theme: "gruvbox-dark";
    mode: "text/plain";
    lineNumbers: false;
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
        lineNumbers: false,
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
});
</script>
