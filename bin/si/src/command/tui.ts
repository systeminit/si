import { Context } from "../context.ts";
import { ApiContext } from "../api.ts";
import * as p from 'npm:@clack/prompts';
import color from 'npm:picocolors';
import { apiConfig } from "../si_client.ts";
import { ChangeSetsApi, ChangeSetViewV1 } from "https://jsr.io/@systeminit/api-client/1.9.0/api.ts";

const EXIT = "-1";

export async function callTui(ctx: Context, _apiCtx: ApiContext) {
  const changeSetsApi = new ChangeSetsApi(apiConfig);
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
  cs.start("Retrieving change sets");
  const response = await changeSetsApi.listChangeSets({ workspaceId });
  const changeSets = response.data.changeSets as ChangeSetViewV1[];
  cs.stop();

  const options = changeSets.map((c) => {
    return {
      value: c.id,
      label: c.name,
    }
  })
  const csId = await p.select({
    message: "Choose a change set:",
    options,
  });

  if (p.isCancel(csId)) {
    p.cancel("Cancelled, exiting...")
    process.exit(0);
  }

  const c = p.spinner();
  c.start("Retrieving components");
  c.stop();

  while (true) {
    const componentId = await p.select({
      message: "Choose a component to review:",
      options: [
        {value: EXIT, label: "[quit]"}
      ],
    });
    if (p.isCancel(componentId)) {
      p.cancel("Cancelled, exiting...")
      process.exit(0);
    }
    if (componentId === EXIT) break;
  }

  p.outro("Goodbye");
}
