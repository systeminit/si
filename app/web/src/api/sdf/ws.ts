import Bottle from "bottlejs";
import { eventChangeSetApplied$, eventChangeSetCreated$ } from "@/observable/change_set";

export class SdfWs {
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
        await new Promise((resolve) => setTimeout(resolve, intrasleep));
        loop++;
      }
      return isOpened();
    }
  }
}

function onClose(ev: CloseEvent): any {
  const bottle = Bottle.pop("default");
  const sdf = bottle.container.SDF;
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
  console.log("modelData", { modelData, ev });
  if (modelData.payload.kind == "ChangeSetCreated") {
    eventChangeSetCreated$.next(modelData.payload.data);
  } else if (modelData.payload.kind == "ChangeSetApplied") {
    eventChangeSetApplied$.next(modelData.payload.data);
  }
}
