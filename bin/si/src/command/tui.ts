import { Context } from "../context.ts";
import { ApiContext } from "../api.ts";
import * as p from 'npm:@clack/prompts';
import color from 'npm:picocolors';
import { apiConfig } from "../si_client.ts";
// import { ChangeSetsApi, ChangeSetViewV1, MvApi } from "https://jsr.io/@systeminit/api-client/1.9.0/api.ts";
import { ChangeSetsApi, ChangeSetViewV1, MvApi } from "../../../../generated-sdks/typescript/api.ts";

const EXIT = "-1";

export async function callTui(ctx: Context, _apiCtx: ApiContext) {
  const changeSetsApi = new ChangeSetsApi(apiConfig);
  const mvSetsApi = new MvApi(apiConfig);
  const workspaceId = ctx.workspaceId;
  if (!workspaceId) throw new Error("No Workspace");

  p.updateSettings({
		aliases: {
			w: 'up',
			s: 'down',
			a: 'left',
			d: 'right',
		},
	});

  p.intro("Welcome! Let's review propsed changes in your workspace")

  const cs = p.spinner({
    onCancel: () => {
      process.exit(0);
    }
  });
  cs.start(`${color.bgBlack(color.greenBright("Retrieving change sets"))}`);
  const response = await changeSetsApi.listChangeSets({ workspaceId });
  const changeSets = response.data.changeSets as ChangeSetViewV1[];
  cs.stop();

  const options = changeSets.filter((c) => !c.isHead).map((c) => {
    return {
      value: c.id,
      label: c.name,
    }
  })
  const changeSetId = await p.select({
    message: "Choose a change set:",
    options,
  });

  if (p.isCancel(changeSetId)) {
    p.cancel("Cancelled, exiting...")
    process.exit(0);
  }

  const c = p.spinner();

  c.start(`${color.bgBlack(color.greenBright("Retrieving components"))}`);
  const componentList = await mvSetsApi.get({
    workspaceId,
    changeSetId,
    entityId: workspaceId,
    kind: "ComponentList"
  })

  // N+1 requests are horribly in-efficient
  // if we wanted to invest in a TUI we could do the same
  // "sync all the MVs" on start, open a web socket, etc
  const componentDetails = await Promise.all(componentList.data.data.components.map((c) => {
    return mvSetsApi.get({
      workspaceId,
      changeSetId,
      entityId: c.id,
      kind: "ComponentInList"
    })
  }));
  c.stop();

  const componentOptions = componentDetails.filter((req) => {
    const c = req.data.data;
    return c.diffStatus !== "None";
  }).map((req) => {
    const c = req.data.data;
    return {
      value: c.id,
      label: c.name,
    }
  });

  if (componentOptions.length === 0) {
    p.outro(`${color.bgBlack(color.redBright("There are no modifications on this simulated change set."))}`);
    p.outro("Goodbye!");
    process.exit(0);
  }
  componentOptions.unshift({value: EXIT, label: "[quit]"})

  while (true) {
    const componentId = await p.select({
      message: "Choose a modified component to review:",
      options: componentOptions,
    });
    if (p.isCancel(componentId)) {
      p.cancel("Cancelled, exiting...")
      process.exit(0);
    }
    if (componentId === EXIT) break;

    const diff = await mvSetsApi.get({
      workspaceId,
      changeSetId,
      entityId: componentId,
      kind: "ComponentDiff"
    });

    p.outro(`Here is what changed:`)
    console.log(JSON.stringify(diff.data.data, undefined, 2));
  }

  p.outro("Goodbye");
}
