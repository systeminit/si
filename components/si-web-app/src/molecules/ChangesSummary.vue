<template>
  <SummaryCard>
    <template v-slot:title>Changes</template>

    <template v-slot:content>
      <div class="flex flex-col w-full h-full mx-1" v-if="changesData">
        <div class="flex flex-row opened-indicator">
          <div>Opened:</div>
          <div class="ml-1">{{ changesData.openChangeSetCount }}</div>
        </div>

        <div
          class="mt-1"
          v-if="
            showSelectedChangesetData &&
              changesData.currentChangeSet &&
              changeSet
          "
        >
          <div class="flex flex-row changeset-indicator">
            <div class="">{{ changeSet.name }}</div>
          </div>

          <div class="flex flex-row pl-2 edits-indicator">
            <div>Nodes:</div>

            <div class="ml-1 additions">+</div>
            <div class="additions">
              {{ changesData.currentChangeSet.newNodes }}
            </div>

            <div class="ml-1 removals">-</div>
            <div class="removals">
              {{ changesData.currentChangeSet.deletedNodes }}
            </div>

            <div class="ml-1 updates">u</div>
            <div class="updates">
              {{ changesData.currentChangeSet.modifiedNodes }}
            </div>
          </div>

          <!--
          <div class="flex flex-row pl-2 edits-indicator">
            <div>Edits:</div>
            <div class="ml-1 additions">+</div>
            <div class="additions">
              {{ changesData.currentChangeset.nodeEdits }}
            </div>
          </div>
          -->
        </div>
      </div>
    </template>
  </SummaryCard>
</template>

<script lang="ts">
import Vue from "vue";

import { changesData, Changes } from "@/api/visualization/changesData";

import SummaryCard from "@/atoms/SummaryCard.vue";
import {
  IChangesSummaryReplySuccess,
  IChangesSummaryRequest,
  ApplicationDal,
} from "@/api/sdf/dal/applicationDal";
import { workspace$, changeSet$, refreshChangesSummary$ } from "@/observables";
import { combineLatest } from "rxjs";
import { tap, pluck } from "rxjs/operators";
import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";

interface IData {
  changesData: IChangesSummaryReplySuccess | null;
}

export default Vue.extend({
  name: "ChangeSummary",
  components: {
    SummaryCard,
  },
  props: {
    showSelectedChangesetData: {
      type: Boolean,
      default: true,
    },
    applicationId: {
      type: String,
    },
  },
  subscriptions(): Record<string, any> {
    let applicationId$ = this.$watchAsObservable("applicationId", {
      immediate: true,
    }).pipe(pluck("newValue"));
    return {
      changeSet: changeSet$,
      activityDataBackend: combineLatest(
        applicationId$,
        workspace$,
        changeSet$,
        refreshChangesSummary$,
      ).pipe(
        tap(async ([applicationId, workspace, changeSet]) => {
          if (applicationId && workspace) {
            let request: IChangesSummaryRequest = {
              applicationId,
              workspaceId: workspace.id,
            };
            if (changeSet) {
              request["changeSetId"] = changeSet.id;
            }
            let reply = await ApplicationDal.changesSummary(request);
            if (reply.error) {
              emitEditorErrorMessage(reply.error.message);
            } else {
              // @ts-ignore
              this.changesData = reply;
            }
          }
        }),
      ),
    };
  },

  data(): IData {
    return {
      changesData: null,
    };
  },
});
</script>

<style scoped>
.opened-indicator {
  font-size: 10px;
  font-weight: 400;
  color: #b7b9c9;
}

.changeset-indicator {
  font-size: 10px;
  font-weight: 600;
  color: #b7b9c9;
}

.edits-indicator {
  font-size: 9px;
  font-weight: 400;
  color: #b7b9c9;
}

.additions {
  color: #a6e2a5;
}

.removals {
  color: #e2a5a5;
}

.updates {
  color: #e2c8a5;
}
</style>
