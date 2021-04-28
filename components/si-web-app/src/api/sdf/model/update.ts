import PQueue from "p-queue";

import { IEntity, Entity } from "@/api/sdf/model/entity";
//import { ISystem, System } from "@/api/sdf/model/system";
//import { INode, Node } from "@/api/sdf/model/node";
//import { IEdge, Edge } from "@/api/sdf/model/edge";
//import {
//  IChangeSet,
//  ChangeSet,
//  IChangeSetParticipant,
//  ChangeSetParticipant,
//} from "@/api/sdf/model/changeSet";
//import { IOpEntitySet, OpEntitySet } from "@/api/sdf/model/ops";
//import { EventLog, IEventLog } from "@/api/sdf/model/eventLog";
//import { Resource, IResource } from "@/api/sdf/model/resource";
//import { PublicKey, IPublicKey } from "@/api/sdf/model/keyPair";
//import { Secret, ISecret } from "@/api/sdf/model/secret";
//import { Event, IEvent } from "@/api/sdf/model/event";
//import { OutputLine, IOutputLine } from "@/api/sdf/model/outputLine";
//import { sdf } from "@/api/sdf";
import { IUpdateClock } from "@/api/sdf/model/updateClock";
//import { db } from "@/api/sdf/dexie";
//import { ApiClient, IApiClient } from "./apiClient";
import Bottle from "bottlejs";
import { UpdateTracker } from "@/api/updateTracker";
import {
  entityQualifications$,
  entityQualificationStart$,
  resources$,
  workflowRuns$,
  workflowRunSteps$,
  workflowRunStepEntities$,
} from "@/observables";

export interface IUpdateClockGlobal extends IUpdateClock {
  id: string;
}

const PQ = new PQueue({ concurrency: 2 });

export class Update {
  socket: WebSocket;

  constructor(websocketUrl: string) {
    this.socket = new WebSocket(websocketUrl);
    this.socket.addEventListener("message", onMessage);
    this.socket.addEventListener("close", onClose);
  }

  async opened(timeout = 10000) {
    const isOpened = () => this.socket.readyState === WebSocket.OPEN;

    if (this.socket.readyState !== WebSocket.CONNECTING) {
      return isOpened();
    } else {
      const intrasleep = 100;
      const ttl = timeout / intrasleep; // time to loop
      let loop = 0;
      while (this.socket.readyState === WebSocket.CONNECTING && loop < ttl) {
        await new Promise(resolve => setTimeout(resolve, intrasleep));
        loop++;
      }
      return isOpened();
    }
  }
}

function onClose(ev: CloseEvent): any {
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;
  if (sdf.token) {
    // console.log("websocket has closed - reconnecting");
    sdf.setupUpdate();
    if (sdf.update) {
      sdf.update
        .opened()
        .then((_success: any) => {
          // console.log("websocket connection re-established");
        })
        .catch((_timeout: any) => {
          // console.log("reconnect failed - scheduling another go");
          setTimeout(() => {
            onClose(ev);
          }, Math.floor(Math.random() * 5000));
        });
    }
  } else {
    console.log("websocket closed, and no token provided - not reconnecting");
  }
}

function onMessage(ev: MessageEvent) {
  const modelData = JSON.parse(ev.data);
  const bottle = Bottle.pop("default");
  const updateTracker: UpdateTracker = bottle.container.UpdateTracker;

  if (modelData.model?.siStorable?.typeName == "entity") {
    const model = new Entity(modelData.model as IEntity);
    PQ.add(() => updateTracker.dispatch("Entity", model));
  } else if (modelData.model?.siStorable?.typeName == "qualification") {
    entityQualifications$.next(modelData.model);
  } else if (modelData.model?.siStorable?.typeName == "qualificationStart") {
    entityQualificationStart$.next(modelData.model);
  } else if (modelData.model?.siStorable?.typeName == "workflowRun") {
    workflowRuns$.next(modelData.model);
  } else if (modelData.model?.siStorable?.typeName == "workflowRunStep") {
    workflowRunSteps$.next(modelData.model);
  } else if (modelData.model?.siStorable?.typeName == "workflowRunStepEntity") {
    workflowRunStepEntities$.next(modelData.model);
  } else if (modelData.model?.siStorable?.typeName == "resource") {
    resources$.next(modelData.model);
  } else {
    //console.log("websocket on message", { ev, model_data: modelData });
  }
  // } else if (model_data.model?.siStorable?.typeName == "system") {
  //   const model = new System(model_data.model as ISystem);
  //   // console.log("system msg", { model });
  //   PQ.add(() => model.save());
  // } else if (model_data.model?.siStorable?.typeName == "edge") {
  //   const model = new Edge(model_data.model as IEdge);
  //   // console.log("edge msg", { model });
  //   PQ.add(() => model.save());
  // } else if (model_data.model?.siStorable?.typeName == "changeSet") {
  //   const model = new ChangeSet(model_data.model as IChangeSet);
  //   // console.log("changeSet msg", { model });
  //   PQ.add(() => model.save());
  // } else if (model_data.model?.siStorable?.typeName == "node") {
  //   const model = new Node(model_data.model as INode);
  //   // console.log("node msg", { model });
  //   PQ.add(() => model.save());
  // } else if (model_data.model?.siStorable?.typeName == "opEntitySet") {
  //   const model = new OpEntitySet(model_data.model as IOpEntitySet);
  //   // console.log("opEntitySet msg", { model });
  //   PQ.add(() => model.save());
  // } else if (model_data.model?.siStorable?.typeName == "changeSetParticipant") {
  //   const model = new ChangeSetParticipant(
  //     model_data.model as IChangeSetParticipant,
  //   );
  //   // console.log("changeSetParticipant msg", { model });
  //   PQ.add(() => model.save());
  // } else if (model_data.model?.siStorable?.typeName == "eventLog") {
  //   const model = new EventLog(model_data.model as IEventLog);
  //   // console.log("eventLog msg", { model });
  //   PQ.add(() => model.save());
  // } else if (model_data.model?.siStorable?.typeName == "resource") {
  //   const model = new Resource(model_data.model as IResource);
  //   // console.log("resource msg", { model });
  //   PQ.add(() => model.save());
  // } else if (model_data.model?.siStorable?.typeName == "keyPair") {
  //   const model = new PublicKey(model_data.model as IPublicKey);
  //   // console.log("keyPair msg", { model, model_data });
  //   PQ.add(() => model.save());
  // } else if (model_data.model?.siStorable?.typeName == "secret") {
  //   const model = new Secret(model_data.model as ISecret);
  //   // console.log("secret msg", { model });
  //   PQ.add(() => model.save());
  // } else if (model_data.model?.siStorable?.typeName == "apiClient") {
  //   const model = new ApiClient(model_data.model as IApiClient);
  //   // console.log("apiClient msg", { model });
  //   PQ.add(() => model.save());
  // } else if (model_data.model?.siStorable?.typeName == "event") {
  //   const model = new Event(model_data.model as IEvent);
  //   // console.log("event msg", { model });
  //   PQ.add(() => model.save());
  // } else if (model_data.model?.siStorable?.typeName == "outputLine") {
  //   const model = new OutputLine(model_data.model as IOutputLine);
  //   // console.log("outputLine msg", { model });
  //   PQ.add(() => model.save());
  // }
}
