import { IEntity, Entity } from "@/api/sdf/model/entity";
import { ISystem, System } from "@/api/sdf/model/system";
import { IEdge, Edge } from "@/api/sdf/model/edge";
import { IUpdateClock } from "@/api/sdf/model/updateClock";
import { sdf } from "@/api/sdf";

export interface IUpdateClockGlobal {
  [key: string]: IUpdateClock;
}

export const UPDATECLOCK: IUpdateClockGlobal = {};

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
    let updateClock: IUpdateClock;
    if (UPDATECLOCK[workspaceId]) {
      updateClock = UPDATECLOCK[workspaceId];
    } else {
      updateClock = {
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

function onMessage(ev: MessageEvent) {
  const model_data = JSON.parse(ev.data);
  console.log("typename", {
    model_data,
    typeName: model_data.model.siStorable?.typeName,
  });
  if (model_data.model?.siStorable?.updateClock) {
    let modelUpdateClock = model_data.model.siStorable.updateClock;
    if (
      modelUpdateClock.epoch >= UPDATECLOCK.epoch &&
      modelUpdateClock.updateCount > UPDATECLOCK.updateCount
    ) {
      UPDATECLOCK.epoch = modelUpdateClock.epoch;
      UPDATECLOCK.updateCount = modelUpdateClock.updateCount;
    }
  }
  if (model_data.model?.siStorable?.typeName == "entity") {
    const model = new Entity(model_data.model as IEntity);
    model.save();
  } else if (model_data.model?.siStorable?.typeName == "system") {
    const model = new System(model_data.model as ISystem);
    model.save();
  } else if (model_data.model?.siStorable?.typeName == "edge") {
    const model = new Edge(model_data.model as IEdge);
    model.save();
  }

  console.log("I have a message", { ev });
}
