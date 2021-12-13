<template>
  <div class="flex w-full h-full">
    <SiError
      testId="application-list-wad-error"
      :message="errorMessage"
      @clear="clearErrorMessage"
    />

    <div class="flex flex-col w-full" v-if="applicationList.length">
      <div
        v-for="appEntry in applicationList"
        :key="appEntry.application.id"
        class="mb-6"
      >
        <router-link :to="cardLink(appEntry.application.id)">
          <ApplicationDetailCard
            :linkTo="cardLink(appEntry.application.id)"
            :applicationEntry="appEntry"
            :cardLink="cardLink(appEntry.application.id)"
          />
        </router-link>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { RawLocation } from "vue-router";

import SiError from "@/atoms/SiError.vue";
import ApplicationDetailCard from "@/molecules/ApplicationDetailCard.vue";

import { workspace$, organization$, applicationCreated$ } from "@/observables";
import {
  ApplicationDal,
  IApplicationListReplySuccess,
} from "@/api/sdf/dal/applicationDal";
import { combineLatest, from } from "rxjs";
import { switchMap, tap } from "rxjs/operators";
import _ from "lodash";

interface IData {
  errorMessage: string;
  isLoading: boolean;
  applicationList: IApplicationListReplySuccess["list"];
}

export default Vue.extend({
  name: "ApplicationList",
  props: {
    linkTo: {
      type: String,
    },
  },
  data(): IData {
    return {
      errorMessage: "",
      isLoading: true,
      applicationList: [],
    };
  },
  components: {
    SiError,
    ApplicationDetailCard,
  },
  subscriptions(this: any): Record<string, any> {
    return {
      currentWorkspace: workspace$,
      currentOrganization: organization$,
      applicationListObservable: combineLatest(
        workspace$,
        applicationCreated$,
      ).pipe(
        switchMap(([workspace, _applicationCreated]) => {
          this.isLoading = true;
          if (workspace) {
            return from(
              ApplicationDal.listApplications({ workspaceId: workspace.id }),
            );
          } else {
            return from([
              { error: { code: 42, message: "missing workspace" } },
            ]);
          }
        }),
        tap(reply => {
          this.isLoading = false;
          if (reply.error) {
            if (reply.error.code == 42) {
              return;
            } else {
              this.errorMessage = reply.error.message;
            }
          } else {
            this.applicationList = _.sortBy(reply.list, ["application.name"]);
          }
        }),
      ),
    };
  },
  methods: {
    // I feel shame
    cardLink(this: any, applicationId: string): RawLocation | null {
      if (this.currentWorkspace && this.currentOrganization) {
        return {
          name: "applicationDetails",
          params: {
            workspaceId: this.currentWorkspace.id,
            organizationId: this.currentOrganization.id,
            applicationId: applicationId,
          },
        };
      }
      return null;
    },
    clearErrorMessage() {
      this.errorMessage = "";
    },
  },
});
</script>
