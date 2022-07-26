<template>
  <div
    id="workspace"
    class="overflow-hidden flex flex-col w-full h-full select-none"
  >
    <Navbar />

    <router-view class="overflow-hidden" />

    <StatusBar class="flex-initial" />
  </div>
</template>

<script setup lang="ts">
import Navbar from "@/organisms/Navbar.vue";
import StatusBar from "@/organisms/StatusBar.vue";
import { ApplicationService } from "@/service/application";
import { ChangeSetService } from "@/service/change_set";
import { GlobalErrorService } from "@/service/global_error";
import { onMounted } from "vue";
import { take } from "rxjs";
import _ from "lodash";

// FIXME(nick,adam): create an application and a changeset when the editor is loaded.
// We need both in order to create and drag nodes.

// we use this lock to guarantee that we won't loop the application creation, since some calls during the process
// also notify the ApplicationService.currentApplication's observers. -- victor

onMounted(() => {
  ApplicationService.currentApplication()
    .pipe(take(1))
    .subscribe((application) => {
      if (application === null) {
        ApplicationService.listApplications()
          .pipe(take(1))
          .subscribe((response) => {
            if (response.error) {
              console.log("oopsie poopsie! we could not list applications!");
              GlobalErrorService.set(response);
              return;
            }

            let potentialApplication = response.list.shift();
            if (potentialApplication) {
              ApplicationService.setCurrentApplication({
                applicationId: potentialApplication.application.id,
              })
                .pipe(take(1))
                .subscribe((response) => {
                  if (response.error) {
                    console.log("could not set current application!");
                    GlobalErrorService.set(response);
                    return;
                  }
                });
            } else {
              ApplicationService.createApplication({
                name: `poop-${_.uniqueId()}`,
              })
                .pipe(take(1))
                .subscribe((response) => {
                  if (response.error) {
                    console.log(
                      "oopsie poopsie! we could not create an application!",
                    );
                    GlobalErrorService.set(response);
                    return;
                  }

                  ApplicationService.setCurrentApplication({
                    applicationId: response.application.id,
                  })
                    .pipe(take(1))
                    .subscribe((response) => {
                      if (response.error) {
                        console.log("could not set current application!");
                        GlobalErrorService.set(response);
                        return;
                      }
                    });
                });
            }
          });
      }
    });

  ChangeSetService.currentChangeSet()
    .pipe(take(1))
    .subscribe((changeSet) => {
      if (changeSet === null) {
        ChangeSetService.createChangeSet({
          changeSetName: `canoe-${_.uniqueId()}`,
        })
          .pipe(take(1))
          .subscribe((response) => {
            if (response.error) {
              console.log("oopsie poopsie! we could not create a change set!");
              GlobalErrorService.set(response);
              return;
            }

            ChangeSetService.startEditSession({
              changeSetPk: response.changeSet.pk,
            })
              .pipe(take(1))
              .subscribe((response) => {
                if (response.error) {
                  console.log("could not start edit session!");
                  GlobalErrorService.set(response);
                  return;
                }
              });
          });
      }
    });
});
</script>
