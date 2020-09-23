import { IEntity, Entity } from "@/api/sdf/model/entity";

export class Update {
  socket: WebSocket;

  constructor(websocketUrl: string) {
    this.socket = new WebSocket(websocketUrl);
    this.socket.addEventListener("message", onMessage);
  }
}

function onMessage(ev: MessageEvent) {
  const model_data = JSON.parse(ev.data);
  console.log("typename", {
    model_data,
    typeName: model_data.model.siStorable?.typeName,
  });
  if (model_data.model?.siStorable?.typeName == "entity") {
    console.log("trying to save the new entity");
    const model = new Entity(model_data.model as IEntity);
    model.save();
  }
  console.log("I have a message", { ev });
}
