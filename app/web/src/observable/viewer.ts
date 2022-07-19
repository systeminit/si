import { ReplaySubject } from "rxjs";
import { persistToSession } from "@/observable/session_state";
import router from "@/router";

/**
 * The type used to describe available viewers
 */
export type Viewer = "compose" | "lab" | "view" | "runtime";

/**
 * The currently selected viewer
 */
export const viewer$ = new ReplaySubject<Viewer>(1);
persistToSession("viewer", viewer$);

/**
 * The action taken when the viewer is changed
 */
viewer$.subscribe(async (newViewer: Viewer) => {
  if (newViewer === "compose") {
    await router.push({
      name: "workspace-compose",
      path: "c",
    });
  } else if (newViewer === "lab") {
    await router.push({
      name: "workspace-lab",
      path: "l",
    });
  } else if (newViewer === "view") {
    await router.push({
      name: "workspace-view",
      path: "v",
    });
  } else {
    await router.push({
      name: "workspace-runtime",
      path: "r",
    });
  }
});

/**
 * The default value
 */
viewer$.next("compose");
