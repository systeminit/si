<template>
  <v-container class="fluid">
    <v-card>
      <v-card-title>{{ getEntity.displayName }}</v-card-title>
      <v-card-subtitle>{{ entityName }}</v-card-subtitle>
      <v-card-text v-if="entityType == 'sshKey'">
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
              {{ getEntity.keyType }}
            </v-list-item-content>
          </v-list-item>
          <v-list-item>
            <v-list-item-content>
              <v-list-item-title>
                Key Format
              </v-list-item-title>
              {{ getEntity.keyFormat }}
            </v-list-item-content>
          </v-list-item>
          <v-list-item>
            <v-list-item-content>
              <v-list-item-title>
                Bits
              </v-list-item-title>
              {{ getEntity.bits }}
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
                :value="getEntity.publicKey"
              >
              </v-textarea>
            </v-list-item-content>
          </v-list-item>
        </v-list>
      </v-card-text>
      <v-card-text v-else>
        <v-btn-toggle>
          <v-btn>Deploy</v-btn>
          <v-btn>Clone</v-btn>
          <v-btn>Delete</v-btn>
        </v-btn-toggle>
        <v-list flat>
          <v-list-item>
            <v-list-item-content>
              <v-list-item-title>
                ID
              </v-list-item-title>
              {{ getEntity.id }}
            </v-list-item-content>
          </v-list-item>
          <v-list-item>
            <v-list-item-content>
              <v-list-item-title>
                Kubernetes Version
              </v-list-item-title>
              {{ getEntity.kubernetesVersion }}
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
import awsEksClusterRuntimeGetEntity from "@/graphql/queries/awsEksClusterRuntimeGetEntity.gql";
import {
  Query,
  SshKeyEntity,
  AwsEksClusterRuntimeGetEntityRequest,
  SshKeyGetEntityRequest,
} from "@/graphql-types";

interface DataField {
  getEntity: SshKeyEntity | AwsEksClusterRuntimeEntity;
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
    entityName: String,
    entityType: String,
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
      getEntity: {},
      loading: true,
    };
  },
  created: function(): void {
    let queryString;
    let resultString: string;
    if (this.entityType == "awsEksClusterRuntime") {
      queryString = awsEksClusterRuntimeGetEntity;
      resultString = "awsEksClusterRuntimeGetEntity";
    } else if (this.entityType == "sshKey") {
      queryString = sshKeyGetEntity;
      resultString = "sshKeyGetEntity";
    } else {
      // Um.. what?
      queryString = sshKeyGetEntities;
      resultString = "sshKeyGetEntities";
    }
    this.$apollo.addSmartQuery("getEntity", {
      query: queryString,
      update(data: Query): SshKeyEntity | AwsEksClusterRuntimeEntity {
        this.loading = false;
        if (data[resultString] && data[resultString]["entity"]) {
          return data[resultString]["entity"];
        } else {
          return {};
        }
      },
      variables():
        | SshKeyGetEntityRequest
        | awsEksClusterRuntimeGetEntityRequest {
        return {
          entityId: this.entityId,
        };
      },
    });
  },
});
</script>
