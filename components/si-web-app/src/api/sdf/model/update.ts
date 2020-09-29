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

function onClose(ev: MessageEvent) {
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
    console.log("entity", { model });
    model.save();
  } else if (model_data.model?.siStorable?.typeName == "system") {
    const model = new System(model_data.model as ISystem);
    console.log("system", { model });
    model.save();
  } else if (model_data.model?.siStorable?.typeName == "edge") {
    const model = new Edge(model_data.model as IEdge);
    console.log("edge", { model });
    model.save();
  } else if (model_data.model?.siStorable?.typeName == "changeSet") {
    const model = new ChangeSet(model_data.model as IChangeSet);
    console.log("changeSet", { model });
    model.save();
  } else if (model_data.model?.siStorable?.typeName == "node") {
    const model = new Node(model_data.model as INode);
    console.log("node", { model });
    model.save();
  } else if (model_data.model?.siStorable?.typeName == "changeSetParticipant") {
    const model = new ChangeSetParticipant(
      model_data.model as IChangeSetParticipant,
    );
    console.log("change set participant", { model });
    model.save();
  }
}
