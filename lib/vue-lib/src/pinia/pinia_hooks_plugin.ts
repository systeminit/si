/* eslint-disable @typescript-eslint/no-explicit-any */
import { PiniaPlugin, PiniaPluginContext } from "pinia";
import {
  ComponentInternalInstance,
  computed,
  getCurrentInstance,
  reactive,
} from "vue";
import isPromise from "is-promise";
import * as _ from "lodash-es";

type MaybePromise<T> = T | Promise<T>;
declare module "pinia" {
  /* eslint-disable @typescript-eslint/no-unused-vars */
  export interface DefineStoreOptionsBase<S, Store> {
    // adds our new custom option for activation/deactivation hook
    onActivated?: (this: Store) => MaybePromise<void | (() => void)>;
    // adds our new custom option for hook that first on first use/activation only
    onInit?: (this: Store) => MaybePromise<void>;
  }
  export interface PiniaCustomStateProperties<S> {
    trackStoreUsedByComponent(component: ComponentInternalInstance): void;
  }
}

// TODO: couldnt get the typing of T happy here... but it works for consumers
export function addStoreHooks<T extends () => any>(useStoreFn: T) {
  return (...args: Parameters<T>): ReturnType<T> => {
    const store = useStoreFn.apply(null, [...args]) as ReturnType<T>;
    const component = getCurrentInstance();
    if (component) store.trackStoreUsedByComponent(component);
    return store;
  };
}

export const piniaHooksPlugin: PiniaPlugin = ({
  // pinia,
  // app,
  store,
  options: storeOptions,
}: PiniaPluginContext) => {
  /* eslint-disable no-param-reassign */

  // might not need this check, but not sure this plugin code is guaranteed to only be called once
  if (store._trackedStoreUsers) return;

  store._initHookCalled = false;

  // keep a list of all components using this store
  store._trackedStoreUsers = reactive<Record<string, boolean>>({});
  store._trackedStoreUsersCount = computed(
    () => Object.keys(store._trackedStoreUsers).length,
  );
  // expose this info to devtools
  // TODO: determine the best way to safely check in both vite and webpack setups
  if (import.meta.env.DEV /* || process.env.NODE_ENV === "development" */) {
    store._customProperties.add("_trackedStoreUsers");
    store._customProperties.add("_trackedStoreUsersCount");
  }

  function trackStoreUse(
    component: ComponentInternalInstance,
    trackedComponentId: string,
  ) {
    // bail if already tracked - which can happen when stores are using each other in getters
    if (store._trackedStoreUsers[trackedComponentId]) return;

    // console.log("track store use", trackedComponentId);

    store._trackedStoreUsers[trackedComponentId] = true;

    if (!store._initHookCalled && storeOptions.onInit) {
      // TODO: what to do if this errors?
      // eslint-disable-next-line @typescript-eslint/no-floating-promises
      storeOptions.onInit.call(store);
    }

    // console.log(store.$id, "+");
    // store._trackedStoreUsersCount++;
    if (store._trackedStoreUsersCount === 1 && storeOptions.onActivated) {
      // console.log(`${store.$id} - ACTIVATE`);
      // activation fn can return a deactivate / cleanup fn
      store.onDeactivated = storeOptions.onActivated.call(store);

      // activate could be async, so need to resolve if so...
      // TODO may need to think more about this - what if activate errors out?
      if (isPromise(store.onDeactivated)) {
        // eslint-disable-next-line @typescript-eslint/no-floating-promises
        store.onDeactivated.then((resolvedOnDeactivate) => {
          store.onDeactivated = resolvedOnDeactivate;
        });
      }
    }

    // attach the the unmounted hook here so it only ever gets added once
    // (because we bailed above if this component was already tracked)
    const componentAny = component as any;
    // onBeforeUnmount(() => { store.unmarkStoreUsedByComponent(); });
    componentAny.bum = componentAny.bum || [];
    componentAny.bum.push(() => {
      // console.log(`[${store.$id}] -- ${trackedComponentId} un-used`);
      if (!store._trackedStoreUsers[trackedComponentId]) {
        throw new Error(
          `[${store.$id}] Expected to find component ${trackedComponentId} in list of users`,
        );
      }
      delete store._trackedStoreUsers[trackedComponentId];
      if (
        store._trackedStoreUsersCount === 0 &&
        store.onDeactivated &&
        _.isFunction(store.onDeactivated)
      ) {
        // console.log(`${store.$id} - DEACTIVATE`);
        store.onDeactivated.call(store);
      }
    });
  }

  store.trackStoreUsedByComponent = (component: ComponentInternalInstance) => {
    // console.log(
    //   `[${store.$id}] track use by ${component.type.__name} -- mounted? ${component.isMounted}`,
    //   component,
    // );

    // calling lifecycle hooks here (ie beforeMount) causes problems for useStore calls within other store getters/actions
    // so we're injecting the hooks directly into the vue component instance
    // this is probably inadvisable... but seems to work and I don't believe this will change any time soon

    // as an added bonus, this means `watch` calls in our onActivated hook are not bound to the first component that used this store
    // so they will not be destroyed on unmount. This requires us to clean up after ourselves, but it is the desired behaviour.

    // onBeforeMount(() => { store.markStoreUsedByComponent(); });
    const componentIdForUseTracking = `${component.type.__name}/${component.uid}`;
    const componentAny = component as any;

    // console.log(
    //   `tracking ${componentIdForUseTracking}`,
    //   JSON.stringify(store._trackedStoreUsers),
    // );
    if (component.isMounted) {
      trackStoreUse(component, componentIdForUseTracking);
    } else {
      // note - this can happen multiple times, but we handle this case in `trackStoreUse()`
      componentAny.m = componentAny.m || [];
      componentAny.m.push(() => {
        trackStoreUse(component, componentIdForUseTracking);
      });
    }
  };
};
