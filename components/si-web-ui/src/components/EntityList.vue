<template>
  <v-container class="justify-start d-flex" sytle="flex-direction: row;">
    <v-row no-gutters class="justify-start">
      <v-col cols="12">
        <v-card cols="12">
          <v-card-title>
            {{ entityType }}
            <v-spacer></v-spacer>
            <v-btn>
              <v-icon>mdi-plus</v-icon>
            </v-btn>
          </v-card-title>
          <v-card-text>
            <v-data-table
              :headers="headers"
              :items="sshKeyListEntities.items"
              :server-items-length="sshKeyListEntities.totalCount"
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
            Total Count: {{ sshKeyListEntities.totalCount }} Items Per Page:
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
import {
  Query,
  SshKeyListEntitiesReply,
  SshKeyListEntitiesRequest,
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
  loading: boolean;
  nextPageToken: string;
  showMoreDisabled: boolean;
}

export default Vue.extend({
  name: "EntityList",
  props: {
    entityType: String,
    organizationId: String,
    workspaceId: String,
  },
  data(): EntityListData {
    return {
      options: {
        itemsPerPage: 10,
        sortBy: ["displayName"],
        sortDesc: [false],
        page: 1,
      },
      headers: [
        { text: "Display Name", value: "displayName" },
        { text: "Key Type", value: "keyType" },
        { text: "Key Format", value: "keyFormat" },
        { text: "Bits", value: "bits" },
        { text: "State", value: "state" },
        { text: "Actions", value: "action" },
      ],
      sshKeyListEntities: {
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
      this.$apollo.queries.sshKeyListEntities.fetchMore({
        // New variables
        variables: {
          pageToken: this.nextPageToken,
        },
        // Transform the previous result with new data
        updateQuery: (previousResult, { fetchMoreResult }) => {
          this.loading = false;
          const newItems = fetchMoreResult.sshKeyListEntities.items;
          const nextPageToken =
            fetchMoreResult.sshKeyListEntities.nextPageToken;

          this.nextPageToken = nextPageToken;

          if (this.nextPageToken == "") {
            this.showMoreDisabled = true;
          } else {
            this.showMoreDisabled = false;
          }

          return {
            sshKeyListEntities: {
              __typename: previousResult.sshKeyListEntities.__typename,
              // Merging the tag list
              items: [...previousResult.sshKeyListEntities.items, ...newItems],
              totalCount: fetchMoreResult.sshKeyListEntities.totalCount,
              nextPageToken,
            },
          };
        },
      });
    },
  },
  apollo: {
    sshKeyListEntities: {
      query: sshKeyListEntities,
      update(data: Query): SshKeyListEntitiesReply {
        this.loading = false;
        let sshKeyListEntities;
        if (data["sshKeyListEntities"]) {
          sshKeyListEntities = data["sshKeyListEntities"];
        } else {
          // We got bullshit data, so.. just use the old data? <shrub>
          sshKeyListEntities = this.sshKeyListEntities;
        }
        this.nextPageToken = sshKeyListEntities.nextPageToken || "";
        if (this.nextPageToken == "") {
          this.showMoreDisabled = true;
        } else {
          this.showMoreDisabled = false;
        }
        return sshKeyListEntities;
      },
      variables(): SshKeyListEntitiesRequest {
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
    },
  },
});
</script>
