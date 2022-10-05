/* eslint-disable @typescript-eslint/no-explicit-any */
import { defineStore, PiniaPlugin, PiniaPluginContext } from "pinia";
import { getCurrentInstance, ref } from "vue";
import isPromise from "is-promise";

type MaybePromise<T> = T | Promise<T>;
declare module "pinia" {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  export interface DefineStoreOptionsBase<S, Store> {
    // adds our new custom option for activation/deactivation hook
    onActivated?: (this: Store) => MaybePromise<void | (() => void)>;
  }
}

export function addStoreHooks<T extends ReturnType<typeof defineStore>>(
  useStoreFn: T,
) {
  return (...args: Parameters<T>): ReturnType<T> => {
    // NOTE - was fighting against the TS types in here and couldn't quite get it right
    // however it's not important to have it working here versus in _consumers_ of the store
    // and it does all work correctly there

    const store = useStoreFn(...(args as any[])) as any;

    const component = getCurrentInstance();
    const componentAny = component as any;

    if (component) {
      // console.log(
      //   `${store.$id} - useStore called by ${component.type.__name} -- mounted? ${component.isMounted}`,
      // );

      // calling lifecycle hooks here (ie beforeMount) causes problems for useStore calls within other store getters/actions
      // so we're injecting the hooks directly into the vue component instance
      // this is probably inadvisable... but seems like it will work

      // as an added bonus, this means `watch` calls in our onActivated hook are not bound to the first component that used this store
      // so they will not be destroyed on unmount. This requires us to clean up after ourselves, but it is the desired behaviour.

      // onBeforeMount(() => { store.incrementUseCounter(); });
      if (component.isMounted) {
        // console.log("component already mounted");
        store.incrementUseCounter();
      } else {
        componentAny.m = componentAny.m || [];
        componentAny.m.push(() => {
          store.incrementUseCounter();
        });
      }
      // onBeforeUnmount(() => { store.decrementUseCounter(); });
      componentAny.bum = componentAny.bum || [];
      componentAny.bum.push(() => {
        store.decrementUseCounter();
      });
    }
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

  // console.log("plugin called on store -", store.$id);
  // console.log(store, storeOptions);

  // might not need this check, but not sure this plugin code is guaranteed to only be called once
  if (store._numStoreUsers) return;

  // attach new counter to track number of components using this store
  store._numStoreUsers = ref(0);
  // expose our new store property in devtools
  // TODO: determine the best way to safely check in both vite and webpack setups
  if (import.meta.env.DEV || process.env.NODE_ENV === "development") {
    store._customProperties.add("_numStoreUsers");
  }

  store.incrementUseCounter = () => {
    // console.log(store.$id, "+");
    store._numStoreUsers++;
    if (store._numStoreUsers === 1 && storeOptions.onActivated) {
      // activation fn can return a deactivate / cleanup fn
      store.onDeactivated = storeOptions.onActivated.call(store);

      // activate could be async, so need to resolve if so...
      // TODO may need to think more about this - what if activate errors out?
      if (isPromise(store.onDeactivated)) {
        store.onDeactivated.then((resolvedOnDeactivate) => {
          store.onDeactivated = resolvedOnDeactivate;
        });
      }
    }
  };
  store.decrementUseCounter = () => {
    // console.log(store.$id, "-");
    store._numStoreUsers--;
    if (store._numStoreUsers === 0 && store.onDeactivated) {
      store.onDeactivated.call(store);
    }
    if (store._numStoreUsers < 0) {
      throw new Error("pinia_hooks_plugin - store use counter below 0!");
    }
  };
};
