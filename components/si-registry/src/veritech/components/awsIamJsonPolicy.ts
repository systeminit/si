import { registry } from "@/registry";
import { EntityObject } from "@/systemComponent";
import {
  ActionRequest,
  ActionReply,
  CalculatePropertiesRequest,
  CalculatePropertiesResult,
  ResourceHealth,
  ResourceStatus,
  SyncResourceReply,
  SyncResourceRequest,
} from "../../veritech/intelligence";
import { Event, EventLogLevel } from "../../veritech/eventLog";
import { siExec } from "../siExec";
import { failSyncResourceReply, failActionReply } from "../syncResource";
import { AwsCliEnv, awsCredential, awsRegion } from "./awsShared";
import { promises as fs } from "fs";
import os from "os";
import path from "path";

const intelligence = (registry.get("awsIamJsonPolicy") as EntityObject)
  .intelligence;

intelligence.calculateProperties = function(
  req: CalculatePropertiesRequest,
): CalculatePropertiesResult {
  const result: CalculatePropertiesResult = {
    inferredProperties: {
      __baseline: {},
    },
  };

  const setObjectJson = req.entity.manualProperties.__baseline.objectJson;
  if (setObjectJson) {
    result.inferredProperties.__baseline.object = setObjectJson;
  }
  const setObject = req.entity.manualProperties.__baseline.object;
  if (setObject) {
    result.inferredProperties.__baseline.objectJson = setObject;
  }

  return result;
};

intelligence.syncResource = async function(
  request: SyncResourceRequest,
  event: Event,
): Promise<SyncResourceReply> {
  let awsEnv: AwsCliEnv;
  const awsCredResult = awsCredential(request);
  if (awsCredResult.syncResourceReply) {
    return awsCredResult.syncResourceReply;
  }
  if (awsCredResult.awsCliEnv) {
    awsEnv = awsCredResult.awsCliEnv as AwsCliEnv;
  } else {
    throw new Error("aws cli function didn't return an environment");
  }

  const arn = request.resource.state.data?.Policy?.Arn;
  if (!arn) {
    return failSyncResourceReply(request, {
      infoMsg: "missing arn; resource is not created",
    });
  }

  const awsCmd = await siExec(
    event,
    "aws",
    ["iam", "get-policy", "--policy-arn", arn],
    {
      reject: false,
      env: awsEnv,
    },
  );
  if (awsCmd.failed) {
    return failSyncResourceReply(request, {
      errorMsg: "aws iam get-policy failed",
      errorOutput: awsCmd.stderr,
    });
  }

  const awsJson = JSON.parse(awsCmd.stdout);

  const reply = {
    resource: {
      state: {
        data: awsJson,
      },
      health: ResourceHealth.Ok,
      status: ResourceStatus.Created,
    },
  };
  return reply;
};

intelligence.actions = {
  async create(request: ActionRequest, event: Event): Promise<ActionReply> {
    let awsEnv: AwsCliEnv;
    const awsCredResult = awsCredential(request);
    if (awsCredResult.syncResourceReply) {
      return {
        resource: awsCredResult.syncResourceReply.resource,
        actions: [],
      };
    }
    if (awsCredResult.awsCliEnv) {
      awsEnv = awsCredResult.awsCliEnv as AwsCliEnv;
    } else {
      throw new Error("aws cli function didn't return an environment");
    }

    const name = request.entity.name;

    // Not much can usefully done after this point if we're hypothetical, so early return
    if (request.hypothetical) {
      console.log("hypothetical, do nothing!");
      return failActionReply(request);
    }

    const logEntry = event.log(
      EventLogLevel.Debug,
      "creating IAM policy tempfile",
      { name },
    );

    let policyDocument: string;
    try {
      const tempdir = await fs.mkdtemp(path.join(os.tmpdir(), "aws-"));
      policyDocument = path.join(tempdir, "policy.json");
      await fs.writeFile(
        policyDocument,
        request.entity.properties.__baseline.object,
      );
    } catch (err) {
      logEntry.payload["failure"] = `${err}`;
      logEntry.fatal();
      return failActionReply(request);
    }

    const awsCmd = await siExec(
      event,
      "aws",
      [
        "iam",
        "create-policy",
        "--policy-name",
        name,
        "--policy-document",
        // OMFG, really? REALLY???
        `file://${policyDocument}`,
      ],
      {
        reject: false,
        env: awsEnv,
      },
    );
    if (awsCmd.failed) {
      return failActionReply(request, {
        errorMsg: "aws iam get-policy failed",
        errorOutput: awsCmd.stderr,
        status: ResourceStatus.Failed,
      });
    }

    const awsJson = JSON.parse(awsCmd.stdout);

    return {
      resource: {
        state: {
          data: awsJson,
        },
        health: ResourceHealth.Ok,
        status: ResourceStatus.Created,
      },
      actions: [],
    };
  },
};
