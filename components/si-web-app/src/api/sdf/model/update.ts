import { IEntity, Entity } from "@/api/sdf/model/entity";
import { ISystem, System } from "@/api/sdf/model/system";
import { INode, Node } from "@/api/sdf/model/node";
import { IEdge, Edge } from "@/api/sdf/model/edge";
import {
  IChangeSet,
  ChangeSet,
  IChangeSetParticipant,
  ChangeSetParticipant,
} from "@/api/sdf/model/changeSet";
import { IOpEntitySet, OpEntitySet } from "@/api/sdf/model/ops";
import { EventLog, IEventLog } from "@/api/sdf/model/eventLog";
import { Resource, IResource } from "@/api/sdf/model/resource";
import { PublicKey, IPublicKey } from "@/api/sdf/model/keyPair";
import { Secret, ISecret } from "@/api/sdf/model/secret";
import { Event, IEvent } from "@/api/sdf/model/event";
import { OutputLine, IOutputLine } from "@/api/sdf/model/outputLine";
import { sdf } from "@/api/sdf";
import { IUpdateClock } from "@/api/sdf/model/updateClock";
import { db } from "@/api/sdf/dexie";
import store from "@/store";

export interface IUpdateClockGlobal extends IUpdateClock {
  id: string;
}

export interface IUpdateLoadDataRequest {
  loadData: {
    workspaceId: string;
    updateClock: IUpdateClock;
  };
}

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

  async loadData(workspaceId: string) {
    await this.opened();
    let updateClock = await db.globalUpdateClock.get(workspaceId);
    if (!updateClock) {
      updateClock = {
        id: workspaceId,
        epoch: 0,
        updateCount: 0,
      };
    }
    let request: IUpdateLoadDataRequest = {
      loadData: {
        workspaceId,
        updateClock,
      },
    };
    this.socket.send(JSON.stringify(request));
  }
}

function onClose(ev: CloseEvent): any {
  if (sdf.token) {
    console.log("websocket has closed - reconnecting");
    sdf.setupUpdate();
    if (sdf.update) {
      sdf.update
        .opened()
        .then(_success => {
          console.log("websocket connection re-established");
        })
        .catch(_timeout => {
          console.log("reconnect failed - scheduling another go");
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
  const model_data = JSON.parse(ev.data);

  console.log("onMessage", { ev, model_data });
  if (model_data == "loadFinished") {
    console.log("loading finished");
    store.commit("loader/loaded", true);
    return;
  }

  if (model_data.model?.siStorable?.updateClock) {
    let modelUpdateClock = model_data.model.siStorable.updateClock;
    let workspaceId = model_data.model.siStorable.workspaceId;
    db.globalUpdateClock.get(workspaceId).then(currentUpdateClock => {
      if (
        currentUpdateClock &&
        modelUpdateClock.epoch >= currentUpdateClock.epoch &&
        modelUpdateClock.updateCount > currentUpdateClock.updateCount
      ) {
        db.globalUpdateClock.put({ id: workspaceId, ...modelUpdateClock });
      } else {
        if (modelUpdateClock.epoch && modelUpdateClock.updateCount) {
          db.globalUpdateClock.put({ id: workspaceId, ...modelUpdateClock });
        }
      }
    });
  }
  if (model_data.model?.siStorable?.typeName == "entity") {
    const model = new Entity(model_data.model as IEntity);
    console.log("entity msg", { model });
    model.save();
  } else if (model_data.model?.siStorable?.typeName == "system") {
    const model = new System(model_data.model as ISystem);
    console.log("system msg", { model });
    model.save();
  } else if (model_data.model?.siStorable?.typeName == "edge") {
    const model = new Edge(model_data.model as IEdge);
    console.log("edge msg", { model });
    model.save();
  } else if (model_data.model?.siStorable?.typeName == "changeSet") {
    const model = new ChangeSet(model_data.model as IChangeSet);
    console.log("changeSet msg", { model });
    model.save();
  } else if (model_data.model?.siStorable?.typeName == "node") {
    const model = new Node(model_data.model as INode);
    console.log("node msg", { model });
    model.save();
  } else if (model_data.model?.siStorable?.typeName == "opEntitySet") {
    const model = new OpEntitySet(model_data.model as IOpEntitySet);
    console.log("opEntitySet msg", { model });
    model.save();
  } else if (model_data.model?.siStorable?.typeName == "changeSetParticipant") {
    const model = new ChangeSetParticipant(
      model_data.model as IChangeSetParticipant,
    );
    console.log("changeSetParticipant msg", { model });
    model.save();
  } else if (model_data.model?.siStorable?.typeName == "eventLog") {
    const model = new EventLog(model_data.model as IEventLog);
    console.log("eventLog msg", { model });
    model.save();
  } else if (model_data.model?.siStorable?.typeName == "resource") {
    const model = new Resource(model_data.model as IResource);
    console.log("resource msg", { model });
    model.save();
  } else if (model_data.model?.siStorable?.typeName == "keyPair") {
    const model = new PublicKey(model_data.model as IPublicKey);
    console.log("keyPair msg", { model });
    model.save();
  } else if (model_data.model?.siStorable?.typeName == "secret") {
    const model = new Secret(model_data.model as ISecret);
    console.log("secret msg", { model });
    model.save();
  } else if (model_data.model?.siStorable?.typeName == "event") {
    const model = new Event(model_data.model as IEvent);
    console.log("event msg", { model });
    model.save();
  } else if (model_data.model?.siStorable?.typeName == "outputLine") {
    const model = new OutputLine(model_data.model as IOutputLine);
    console.log("outputLine msg", { model });
    model.save();
  }
}
