import { Viewer, viewer$ } from "@/observable/viewer";

/**
 * Fetch the current viewer
 */
function currentViewer(): typeof viewer$ {
  return viewer$;
}

/**
 * Set the viewer
 */
function setTo(viewer: Viewer) {
  viewer$.next(viewer);
}

/**
 * Providers accessors and mutators to the viewer observable
 */
export const ViewerService = {
  setTo,
  currentViewer,
};
