import { createApplication } from "./application/create_application";
import { listApplications } from "./application/list_application";
import { setCurrentApplication } from "./application/set_current_application";
import { clearCurrentApplication } from "./application/clear_current_application";
import { currentApplication } from "./application/current_application";

export const ApplicationService = {
  createApplication,
  listApplications,
  setCurrentApplication,
  currentApplication,
  clearCurrentApplication,
};
