import _ from "lodash";
import { firstValueFrom } from "rxjs";
import { ApplicationService } from "@/service/application";
import { GlobalErrorService } from "@/service/global_error";

export async function setupWorkspaceWithDefaults() {
  const currentApp = await firstValueFrom(
    ApplicationService.currentApplication(),
  );

  if (currentApp === null) {
    const applications = await firstValueFrom(
      ApplicationService.listApplications(),
    );

    if (applications.error) {
      console.log("oopsie poopsie! we could not list applications!");
      GlobalErrorService.set(applications);
      return;
    }

    // NOTE(nick,victor): on first mount, this should only have one application, "default", which was created
    // when signing up for an account.
    const applicationToEnable = applications.list.shift();
    if (applicationToEnable === undefined) {
      console.log("oopsie poopsie! account does not have any applications!");
      return;
    }

    const setCurrentApplicationResponse = await firstValueFrom(
      ApplicationService.setCurrentApplication({
        applicationId: applicationToEnable.application.id,
      }),
    );

    if (setCurrentApplicationResponse.error) {
      console.log("could not set current application");
      GlobalErrorService.set(setCurrentApplicationResponse);
      return;
    }
  }
}
