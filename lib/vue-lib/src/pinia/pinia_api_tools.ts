/* eslint-disable @typescript-eslint/no-explicit-any */

// see pinia docs for more info about plugins - https://pinia.vuejs.org/core-concepts/plugins.html#augmenting-a-store

/*
NOTES / TODOS / IDEAS
  - vue query differentiates between `isFetching` (fetching at any time) and `isLoading` (fetching and no data / first load)
  - set up optimistic update/rollback tooling
  - set up helpers to clear request statuses
    - attach multiple tags to a request, can then clear all request statuses that have a tag (and maybe wildcards)
  - better tooling for making multiple requests together / tracking multiple statuses
  - review tooling/types around keyed request statuses (ie FETCH_THING/1, FETCH_THING/2)
  - return request status directly from action
  - better caching control on requests (ie mark requests with options to only request once, or some timeout, etc)
  - allow request keys to use non-string values - maybe allow objects instead of only arrays
*/

import { PiniaPlugin, PiniaPluginContext } from "pinia";
import { AxiosInstance } from "axios";
import { computed, ComputedRef, reactive, unref, MaybeRef } from "vue";
import * as _ from "lodash-es";
import {
  ApiRequest,
  ApiRequestDebouncer,
  ApiRequestDescription,
  ApiRequestStatus,
  RequestStatusKeyArg,
  RequestUlid,
} from "../utils/api_debouncer";

export * from "../utils/api_debouncer";

// this helper filters an object to only the keys that extend a specific type
// see https://www.piotrl.net/typescript-condition-subset-types/
type SubType<Base, CheckExtends> = {
  [Key in keyof Base]: Base[Key] extends CheckExtends ? Key : never;
};

// here we are filtering all the actions down to those that return an ApiRequest object only
type ApiRequestActionsOnly<A> = SubType<
  A,
  (...args: any) => Promise<ApiRequest<unknown, unknown>>
>;

// augment pinia TS types for our plugin - see https://pinia.vuejs.org/core-concepts/plugins.html#typescript
declare module "pinia" {
  /* eslint-disable @typescript-eslint/no-unused-vars */

  // adds new custom "options" for defineStore fn
  // export interface DefineStoreOptionsBase<S, Store> {}

  // augments the store itself
  export interface PiniaCustomProperties<Id, S, G, A> {
    getRequestStatus(
      requestKey: keyof ApiRequestActionsOnly<A>, // will allow only action names that return an ApiRequest
      ...keyedByArgs: MaybeRef<RequestStatusKeyArg>[]
    ): ComputedRef<ApiRequestStatus>; // TODO add the proper type here

    getRequestStatuses(
      requestKey: keyof ApiRequestActionsOnly<A>, // will allow only action names that return an ApiRequest
      keyedByArgs:
        | MaybeRef<RequestStatusKeyArg>[]
        | ComputedRef<MaybeRef<RequestStatusKeyArg>[]>,
    ): ComputedRef<Record<string, ApiRequestStatus>>;

    clearRequestStatus(
      requestKey: keyof ApiRequestActionsOnly<A>, // will allow only action names that return an ApiRequest
      ...keyedByArgs: MaybeRef<RequestStatusKeyArg>[]
    ): void;
    RETRY_CONFLICT(requestUlid: RequestUlid): Promise<ApiRequest>;
  }

  // augments the store's state
  export interface PiniaCustomStateProperties<S> {
    apiRequestDebouncers: { [key in string]?: ApiRequestDebouncer };
  }
}

export function registerApi(axiosInstance: AxiosInstance) {
  class ApiRequestForSpecificApi<
    Response = any,
    RequestParams = Record<string, unknown>,
  > extends ApiRequest<Response, RequestParams> {
    static api = axiosInstance;
  }

  return ApiRequestForSpecificApi;
}

export type ConflictsForRetry = Record<RequestUlid, [string, ApiRequest]>;

const TRACKING_KEY_SEPARATOR = "%";

export const initPiniaApiToolkitPlugin = (config: { api: AxiosInstance }) => {
  const plugin: PiniaPlugin = ({
    // pinia,
    // app,
    store,
    options: storeOptions,
  }: PiniaPluginContext) => {
    /* eslint-disable no-param-reassign */

    // bail if plugin already called - not sure if necessary but previous pinia version needed it
    if (store.apiRequestDebouncers) return;

    // have to attach our new state to both the store itself and store.$state
    store.apiRequestDebouncers = {};
    (store.$state as any).apiRequestDebouncers = reactive(
      {} as typeof store.apiRequestDebouncers,
    );

    // make available to devtools
    if (import.meta.env.DEV) {
      // eslint-disable-next-line no-underscore-dangle
      store._customProperties.add("apiRequestDebouncers");
    }

    function getTrackingKey(
      actionName: string,
      requestSpec: ApiRequestDescription,
    ) {
      // determine the key we will use when storing the request status
      // most requests are tracked only by their name, for example LOGIN
      // but some requests we may want to track multiple instances of and split by id or other params
      // for example GET_THING%1, GET_THING%2 or GET_OAUTH_ACCOUNT%google%abc123
      const trackingKeyArray: RequestStatusKeyArg[] = [actionName];
      if (requestSpec.keyRequestStatusBy) {
        if (_.isArray(requestSpec.keyRequestStatusBy)) {
          trackingKeyArray.push(...requestSpec.keyRequestStatusBy);
        } else {
          trackingKeyArray.push(requestSpec.keyRequestStatusBy);
        }
      }
      return trackingKeyArray.join(TRACKING_KEY_SEPARATOR);
    }

    // wrap each action in a fn that will take an action result that is an ApiRequest
    // and actually trigger the request, waiting to finish until the request is complete
    function wrapApiAction(
      actionName: string,
      originalActionFn: (...args: any[]) => any,
    ) {
      // NOTE - have to be careful here to deal with non-async actions properly
      return async function wrappedActionFn(...args: any[]) {
        const actionResult: any = await originalActionFn(...args);
        if (actionResult instanceof ApiRequest) {
          const trackingKey = getTrackingKey(
            actionName,
            actionResult.requestSpec,
          );
          store.apiRequestDebouncers[trackingKey] ??= new ApiRequestDebouncer();

          // check if we have already have a pending identical request (same tracking key, and identical payload)
          // if so, we can skip triggering the new api call
          // TODO: probably need to add more options here for caching/dedupe request/logic
          // ex: let us skip certain requests if already successful, not just pending
          const triggerResult = await store.apiRequestDebouncers[
            trackingKey
          ].trigger(config.api, actionResult, store, {
            "si.workspace.id": store.workspaceId,
            "si.change_set.id": store.changeSetId,
          });
          if (!triggerResult) {
            throw new Error(`No trigger result for ${trackingKey}`);
          }
          return triggerResult;
        }
        return actionResult;
      };
    }

    const apiRequestActions: any = {};
    _.each(storeOptions.actions, (actionDef: any, actionName: string) => {
      // we wrap all async actions with a function that checks if the result is an ApiRequest
      // and if so, triggers the api call

      // TODO: this means we must mark our api actions as async... might want something more bulletproof here?
      const isAsync = actionDef.constructor.name === "AsyncFunction";
      if (isAsync) {
        apiRequestActions[actionName] = wrapApiAction(
          actionName,
          store[actionName],
        );
      } else {
        // added this warning to make sure api actions are async, but probably want to do something else
        const originalAction = store[actionName];
        apiRequestActions[actionName] = (...args: any[]) => {
          const actionResult = originalAction(...args);
          if (actionResult instanceof ApiRequest) {
            throw new Error(
              `ApiActions must be async! - mark ${actionName} as async`,
            );
          }
          return actionResult;
        };
      }
    });

    function getKey(
      requestKey: string,
      ...keyedByArgs: MaybeRef<RequestStatusKeyArg>[]
    ) {
      const rawKeyedByArgs = _.map(keyedByArgs, unref);
      return [requestKey, ..._.compact(rawKeyedByArgs)].join(
        TRACKING_KEY_SEPARATOR,
      );
    }

    return {
      getRequestStatus(
        requestKey: string, // will allow only action names that return an ApiRequest
        ...keyedByArgs: MaybeRef<RequestStatusKeyArg>[]
      ): ComputedRef<ApiRequestStatus> {
        const fullKey = getKey(requestKey, ...keyedByArgs);
        return computed(() => {
          store.apiRequestDebouncers[fullKey] ??= new ApiRequestDebouncer();
          return store.apiRequestDebouncers[fullKey];
        });
      },
      getRequestStatuses(
        requestKey: string, // will allow only action names that return an ApiRequest
        keyedByArgs:
          | MaybeRef<RequestStatusKeyArg>[]
          | ComputedRef<MaybeRef<RequestStatusKeyArg>[]>,
      ): ComputedRef<Record<string, ApiRequestStatus>> {
        return computed(() => {
          return _.mapValues(
            _.keyBy(unref(keyedByArgs)),
            (arg) => store.getRequestStatus(requestKey, arg).value,
          );
        });
      },
      clearRequestStatus(
        requestKey: string, // will allow only action names that return an ApiRequest
        ...keyedByArgs: MaybeRef<RequestStatusKeyArg>[]
      ): void {
        const fullKey = getKey(requestKey, ...keyedByArgs);
        delete store.apiRequestDebouncers[fullKey];
      },
      ...apiRequestActions,
    };
  };

  return plugin;
};

export function getCombinedRequestStatus(
  statuses: ComputedRef<ApiRequestStatus>[],
) {
  return computed<ApiRequestStatus>(() => {
    return {
      isRequested: _.every(statuses, { isRequested: true }),
      isFirstLoad: _.some(statuses, { isFirstLoad: true }),
      isPending: _.some(statuses, { isPending: true }),
      isSuccess: _.every(statuses, { isSuccess: true }),
      isError: _.some(statuses, { isError: true }),
      // TODO: do we want to return the first error? an array of errors?
    };
  });
}
