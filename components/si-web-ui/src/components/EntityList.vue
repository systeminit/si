<template>
  <v-container class="justify-start d-flex" sytle="flex-direction: row;">
    <v-row no-gutters class="justify-start">
      <v-col cols="12">
        <v-card cols="12">
          <v-card-title>
            {{ entityName }}
            <v-spacer></v-spacer>
            <v-btn>
              <v-icon>mdi-plus</v-icon>
            </v-btn>
          </v-card-title>
          <v-card-text>
            <v-data-table
              :headers="headers"
              :items="listEntities.items"
              :server-items-length="listEntities.totalCount"
              :options.sync="options"
              :loading="loading"
              v-on:update:page="showMore"
              hide-default-footer
            >
              <template v-slot:item.action="{ item }">
                <v-btn
                  icon
                  :to="{
                    name: 'workspaceShowEntity',
                    params: {
                      organizationId: item.organizationId,
                      workspaceId: item.workspaceId,
                      entityId: item.id,
                      entityType: entityType,
                      entityName: entityName,
                    },
                  }"
                >
                  <v-icon>
                    mdi-eye
                  </v-icon>
                </v-btn>
              </template>
            </v-data-table>
          </v-card-text>
          <v-card-actions>
            <v-spacer> </v-spacer>
            Total Count: {{ listEntities.totalCount }} Items Per Page:
            {{ options.itemsPerPage }}
            <v-btn @click="showMore" :disabled="showMoreDisabled">
              Load More
            </v-btn>
          </v-card-actions>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script lang="ts">
import Vue from "vue";

import { auth } from "@/auth";
import sshKeyListEntities from "@/graphql/queries/sshKeyListEntities.gql";
import awsEksClusterRuntimeListEntities from "@/graphql/queries/awsEksClusterRuntimeListEntities.gql";
import {
  Query,
  SshKeyListEntitiesReply,
  SshKeyListEntitiesRequest,
  AwsEksClusterRuntimeListEntitiesReply,
  AwsEksClusterRuntimeListEntitiesRequest,
  DataOrderByDirection,
} from "@/graphql-types";

interface EntityListData {
  options: {
    itemsPerPage: number;
    sortBy: string[];
    sortDesc: boolean[];
    page: number;
  };
  headers: { text: string; value: string }[];
  sshKeyListEntities: SshKeyListEntitiesReply;
  awsEksClusterRuntimeListEntities: AwsEksClusterRuntimeListEntitiesReply;
  listEntities: SshKeyListEntitiesReply | AwsEksClusterRuntimeListEntitiesReply;
  loading: boolean;
  nextPageToken: string;
  showMoreDisabled: boolean;
}

export default Vue.extend({
  name: "EntityList",
  props: {
    entityName: String,
    entityType: String,
    organizationId: String,
    workspaceId: String,
  },
  data(): EntityListData {
    let headers;
    if (this.entityType == "sshKey") {
      headers = [
        { text: "Display Name", value: "displayName" },
        { text: "Key Type", value: "keyType" },
        { text: "Key Format", value: "keyFormat" },
        { text: "Bits", value: "bits" },
        { text: "State", value: "state" },
        { text: "Actions", value: "action" },
      ];
    } else {
      headers = [
        { text: "Display Name", value: "displayName" },
        { text: "Kubernetes Version", value: "kubernetesVersion" },
        { text: "State", value: "state" },
        { text: "Actions", value: "action" },
      ];
    }
    return {
      options: {
        itemsPerPage: 10,
        sortBy: ["displayName"],
        sortDesc: [false],
        page: 1,
      },
      headers,
      sshKeyListEntities: {
        items: [],
        totalCount: 0,
        nextPageToken: "",
      },
      awsEksClusterRuntimeListEntities: {
        items: [],
        totalCount: 0,
        nextPageToken: "",
      },
      listEntities: {
        items: [],
        totalCount: 0,
        nextPageToken: "",
      },
      loading: true,
      nextPageToken: "",
      showMoreDisabled: true,
    };
  },
  methods: {
    showMore(): void {
      this.loading = true;
      // Fetch more data and transform the original result
      this.$apollo.queries.listEntities.fetchMore({
        // New variables
        variables: {
          pageToken: this.nextPageToken,
        },
        // Transform the previous result with new data
        updateQuery: (previousResult, { fetchMoreResult }) => {
          this.loading = false;
          let newItems;
          let nextPageToken;
          let nextTotalCount;
          let previousResultList;
          if (this.entityType == "sshKey") {
            newItems = fetchMoreResult.sshKeyListEntities.items;
            nextPageToken = fetchMoreResult.sshKeyListEntities.nextPageToken;
            nextTotalCount = fetchMoreResult.sshKeyListEntities.totalCount;
            previousResultList = previousResult.sshKeyListEntities;
          } else if (this.entityType == "awsEksClusterRuntime") {
            newItems = fetchMoreResult.awsEksClusterRuntimeListEntities.items;
            nextPageToken =
              fetchMoreResult.awsEksClusterRuntimeListEntities.nextPageToken;
            nextTotalCount =
              fetchMoreResult.awsEksClusterRuntimeListEntities.totalCount;
            previousResultList =
              previousResult.awsEksClusterRuntimeListEntities;
          }

          this.nextPageToken = nextPageToken;

          if (this.nextPageToken == "") {
            this.showMoreDisabled = true;
          } else {
            this.showMoreDisabled = false;
          }

          let key = `${this.entityType}ListEntities`;

          return {
            [key]: {
              __typename: previousResultList.__typename,
              // Merging the tag list
              items: [...previousResultList.items, ...newItems],
              totalCount: nextTotalCount,
              nextPageToken,
            },
          };
        },
      });
    },
  },
  created: function(): void {
    let queryString;
    let resultString: string;
    if (this.entityType == "awsEksClusterRuntime") {
      queryString = awsEksClusterRuntimeListEntities;
      resultString = "awsEksClusterRuntimeListEntities";
    } else if (this.entityType == "sshKey") {
      queryString = sshKeyListEntities;
      resultString = "sshKeyListEntities";
    } else {
      // Um.. what?
      queryString = sshKeyListEntities;
      resultString = "sshKeyListEntities";
    }
    let thisType = this;
    this.$apollo.addSmartQuery("listEntities", {
      query: queryString,
      update(
        this: typeof thisType,
        data: Query,
      ): SshKeyListEntitiesReply | AwsEksClusterRuntimeListEntitiesReply {
        this.loading = false;
        let listEntities;

        // @ts-ignore - we know, you can't index it. but you can.
        if (data[resultString]) {
          // @ts-ignore same same
          listEntities = data[resultString];
        } else {
          // We got bullshit data, so.. just use the old data? <shrub>
          listEntities = this.listEntities;
        }
        this.nextPageToken = listEntities.nextPageToken || "";
        if (this.nextPageToken == "") {
          this.showMoreDisabled = true;
        } else {
          this.showMoreDisabled = false;
        }
        return listEntities;
      },
      variables(
        this: typeof thisType,
      ): SshKeyListEntitiesRequest | AwsEksClusterRuntimeListEntitiesRequest {
        let orderByDirection: DataOrderByDirection;
        if (this.options["sortDesc"][0]) {
          orderByDirection = DataOrderByDirection.Desc;
        } else {
          orderByDirection = DataOrderByDirection.Asc;
        }
        return {
          pageSize: this.options["itemsPerPage"],
          orderBy: this.options["sortBy"][0],
          orderByDirection,
        };
      },
    });
  },
});
</script>
