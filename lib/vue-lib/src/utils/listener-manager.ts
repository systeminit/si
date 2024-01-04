/* eslint-disable @typescript-eslint/no-explicit-any */
import * as _ from "lodash-es";

type ListenerCallback = (e: any) => any;
type WindowListenerTypes = keyof WindowEventMap;

export class WindowListenerManager {
  private callbacksByType: Partial<
    Record<
      WindowListenerTypes,
      { priority: number; callback: ListenerCallback }[]
    >
  > = {};

  constructor(private el: Window) {}

  addEventListener<ET extends WindowListenerTypes>(
    type: ET,
    callback: ListenerCallback,
    priority = 1,
  ) {
    if (!this.callbacksByType[type]) {
      this.callbacksByType[type] = [];
    }
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    this.callbacksByType[type]!.push({
      priority,
      callback,
    });
    if (this.callbacksByType[type]?.length === 1) {
      window.addEventListener(type, this.listenerHandler);
    }
  }
  removeEventListener<ET extends WindowListenerTypes>(
    type: ET,
    callback: ListenerCallback,
  ) {
    this.callbacksByType[type] = _.reject(
      this.callbacksByType[type],
      (c) => c.callback === callback,
    );
    if (this.callbacksByType[type]?.length === 0) {
      window.removeEventListener(type, this.listenerHandler);
    }
  }

  private listenerHandler = (e: Event) => {
    (e as any)._isStopped = false;
    const originalStop = e.stopPropagation;
    const originalStopImmediate = e.stopImmediatePropagation;
    e.stopPropagation = () => {
      (e as any)._isStopped = true;
      originalStop.call(e);
    };
    e.stopImmediatePropagation = () => {
      (e as any)._isStopped = true;
      originalStopImmediate.call(e);
    };

    const callbacks = _.orderBy(
      _.filter(this.callbacksByType[e.type as WindowListenerTypes]),
      (l) => l.priority,
      ["desc"],
    );

    for (const callbackWithPriority of callbacks) {
      callbackWithPriority.callback(e);
      // check if our overridden stopPropogation methods were called so we can stop calling the next events
      if ((e as any)._isStopped) {
        break;
      }
    }
  };
}

export const windowListenerManager = new WindowListenerManager(window);
