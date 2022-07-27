import _ from "lodash";
import { take } from "rxjs";
import { ApplicationService } from "@/service/application";
import { ChangeSetService } from "@/service/change_set";
import { GlobalErrorService } from "@/service/global_error";

export function setupWorkspaceWithDefaults() {
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

            const potentialApplication = response.list.shift();
            if (potentialApplication) {
              ApplicationService.setCurrentApplication({
                applicationId: potentialApplication.application.id,
              })
                .pipe(take(1))
                .subscribe((response) => {
                  if (response.error) {
                    console.log("could not set current application");
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
                        console.log("could not set current application");
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
                  console.log("could not start edit session");
                  GlobalErrorService.set(response);
                  return;
                }
              });
          });
      }
    });
}
