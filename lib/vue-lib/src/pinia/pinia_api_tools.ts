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
import { AxiosError, AxiosInstance, AxiosResponse } from "axios";
import { computed, ComputedRef, reactive, Ref, unref } from "vue";
import * as _ from "lodash-es";
import {
  createDeferredPromise,
  DeferredPromise,
  promiseDelay,
} from "@si/ts-lib";
import { ulid } from "ulid";
import opentelemetry, { Span } from "@opentelemetry/api";
import { UseAsyncStateReturn } from "@vueuse/core";
import { describePattern, URLPattern } from "../utils/api_utils";

const tracer = opentelemetry.trace.getTracer("si-vue");

export type RequestUlid = string;

// TODO: need to rework these types, and be more flexible... See vue-query for ideas
type RawRequestStatusKeyArg = string | number | undefined | null;
type RequestStatusKeyArg = RawRequestStatusKeyArg | Ref<RawRequestStatusKeyArg>;

// this helper filters an object to only the keys that extend a specific type
// see https://www.piotrl.net/typescript-condition-subset-types/
type SubType<Base, CheckExtends> = Pick<
  Base,
  {
    [Key in keyof Base]: Base[Key] extends CheckExtends ? Key : never;
  }[keyof Base]
>;

// here we are filtering all the actions down to those that return an ApiRequest object only
type ApiRequestActionsOnly<A> = SubType<
  A,
  (
    ...args: any
  ) => Promise<ApiRequest<unknown, unknown> | typeof ApiRequest.noop>
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
      ...keyedByArgs: RequestStatusKeyArg[]
    ): ComputedRef<ApiRequestStatus>;

    getRequestStatuses(
      requestKey: keyof ApiRequestActionsOnly<A>, // will allow only action names that return an ApiRequest
      keyedByArgs: RequestStatusKeyArg[] | ComputedRef<RequestStatusKeyArg[]>,
    ): ComputedRef<Record<string, ApiRequestStatus>>;

    clearRequestStatus(
      requestKey: keyof ApiRequestActionsOnly<A>, // will allow only action names that return an ApiRequest
      ...keyedByArgs: RequestStatusKeyArg[]
    ): void;
    RETRY_CONFLICT(requestUlid: RequestUlid): Promise<ApiRequest>;
  }

  // augments the store's state
  export interface PiniaCustomStateProperties<S> {
    apiRequestStatuses: RawRequestStatusesByKey;
  }
}

export class ApiRequest<
  Response = any,
  RequestParams = Record<string, unknown>,
> {
  // these are used to attach the result which can be used directly by the caller
  static noop = Symbol("API_REQUEST_NOOP");
  // most data and request status info should be used via the store, but it is useful sometimes
  rawResponseData: Response | undefined;
  rawResponseError: Error | AxiosError | undefined;
  rawSuccess?: boolean;

  // eslint-disable-next-line @typescript-eslint/no-useless-constructor
  constructor(
    public requestSpec: ApiRequestDescription<Response, RequestParams>,
  ) {
    if (!this.requestSpec.api) {
      this.requestSpec.api = (this.constructor as any).api;
    }
    if (!this.requestSpec.method) this.requestSpec.method = "get";
  }

  // we use a getter to get the result so that we can add further type restrictions

  // ie, checking success guarantees data is present
  get result():
    | {
        success: true;
        data: Response;
      }
    | {
        success: false;
        err: Error;
        errBody?: any;
        statusCode?: number | undefined;
        data?: Response extends undefined ? never : undefined;
      } {
    /* eslint-disable @typescript-eslint/no-non-null-assertion */
    if (this.rawSuccess === undefined) {
      throw new Error("You must await the request to access the result");
    }

    if (this.rawSuccess) {
      return { success: true, data: this.rawResponseData! };
    } else {
      return {
        success: false,
        // the raw error object - usually an AxiosError
        err: this.rawResponseError!,
        // the (json) body of the failed request, if applicable
        ...(this.rawResponseError instanceof AxiosError && {
          errBody: this.rawResponseError.response?.data,
          statusCode: this.rawResponseError.response?.status,
        }),
      };
    }
  }

  setSuccessfulResult(data: Response | undefined) {
    this.rawSuccess = true;
    this.rawResponseData = data;
  }

  setFailedResult(err: AxiosError | Error) {
    this.rawSuccess = false;
    this.rawResponseError = err;
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

// types to describe our api request definitions
type ApiRequestDescriptionGenerator = (payload: any) => ApiRequestDescription;
type OptimisticReturn = (() => void) | void;
type OptimisticFn = (requestUlid: RequestUlid) => OptimisticReturn;

export type ApiRequestDescription<
  Response = any,
  RequestParams = Record<string, unknown>,
> = {
  api?: AxiosInstance;
  /** http request method, defaults to "get" */
  method?: "get" | "patch" | "post" | "put" | "delete"; // defaults to "get" if empty
  /** url to request, or url pattern for improved instrumentation when the url path constains data */
  url?: string | URLPattern;
  /** request data, passed as querystring for GET, body for everything else */
  params?: RequestParams;
  /** if a multipart form is being sent in a put/post/patch */
  formData?: FormData;
  /** additional args to key the request status */
  keyRequestStatusBy?: RawRequestStatusKeyArg | RawRequestStatusKeyArg[];
  /** function to call if request is successfull (2xx) - usually contains changes to the store */
  onSuccess?(response: Response): Promise<void> | void;
  /** function to call if request fails (>=400) - not common */
  onFail?(response: any, requestUlid: RequestUlid): any | void;
  /** additional headers to pass with request */
  headers?: Record<string, any>;
  /** additional axios options */
  options?: Record<string, any>; // TODO: pull in axios options type?
  /** optional optimistic update fn to call before api request is made, should return a rollback fn called on api error */
  optimistic?: OptimisticFn;
  /** add artificial delay (in ms) before fetching */
  _delay?: number;
  /**
   * Set true to *not* pass the request ULID as a query parameter or JSON parameter.
   * (It is always passed as a header.)
   */
  passRequestUlidInHeadersOnly?: boolean;
};

/** type describing how we store the request statuses */
type RawApiRequestStatus = {
  requestedAt: Date;
  receivedAt?: Date;
  completedAt?: Date;
  lastSuccessAt?: Date;
  payload?: any;
  error?: AxiosResponse | { data: { error: { message: string } } };
  completed?: DeferredPromise<any>;
};
/** type describing the computed getter with some convenience properties */
export type ApiRequestStatus = Partial<RawApiRequestStatus> & {
  isRequested: boolean;
  isPending: boolean;
  isFirstLoad: boolean;
  isError: boolean;
  isSuccess: boolean;
  errorMessage?: string;
  errorCode?: string;
};

type RawRequestStatusesByKey = Record<string, RawApiRequestStatus>;
export type ConflictsForRetry = Record<RequestUlid, [string, ApiRequest]>;

const TRACKING_KEY_SEPARATOR = "%";

export const initPiniaApiToolkitPlugin = (config: { api: AxiosInstance }) => {
  const plugin: PiniaPlugin = ({
    // pinia,
    // app,
    store,
    options: storeOptions,
  }: PiniaPluginContext) => {
    if (store.$id === "heimdall") return;
    /* eslint-disable no-param-reassign */

    // bail if plugin already called - not sure if necessary but previous pinia version needed it
    if (store.apiRequestStatuses) return;

    // have to attach our new state to both the store itself and store.$state
    store.apiRequestStatuses = reactive({} as RawRequestStatusesByKey);
    (store.$state as any).apiRequestStatuses = store.apiRequestStatuses;

    // make available to devtools
    if (import.meta.env.DEV) {
      // eslint-disable-next-line no-underscore-dangle
      store._customProperties.add("apiRequestStatuses");
    }

    // triggers a named api request passing in a payload
    // this makes the api request, tracks the request status, handles errors, etc
    // TODO: probably will rework this a bit to get better type-checking
    async function triggerApiRequest(
      actionName: string,
      requestSpec: ApiRequestDescription,
      requestUlid: RequestUlid,
    ): Promise<any> {
      /* eslint-disable no-param-reassign,consistent-return */
      // console.log('trigger api request', actionName, requestSpec);

      // determine the key we will use when storing the request status
      // most requests are tracked only by their name, for example LOGIN
      // but some requests we may want to track multiple instances of and split by id or other params
      // for example GET_THING%1, GET_THING%2 or GET_OAUTH_ACCOUNT%google%abc123
      const trackingKeyArray: RawRequestStatusKeyArg[] = [actionName];
      if (requestSpec.keyRequestStatusBy) {
        if (_.isArray(requestSpec.keyRequestStatusBy)) {
          trackingKeyArray.push(...requestSpec.keyRequestStatusBy);
        } else {
          trackingKeyArray.push(requestSpec.keyRequestStatusBy);
        }
      }
      const trackingKey = trackingKeyArray.join(TRACKING_KEY_SEPARATOR);

      // check if we have already have a pending identical request (same tracking key, and identical payload)
      // if so, we can skip triggering the new api call
      // TODO: probably need to add more options here for caching/dedupe request/logic
      // ex: let us skip certain requests if already successful, not just pending
      const existingRequest = store.getRequestStatus(trackingKey).value;
      if (
        existingRequest.isPending &&
        _.isEqual(existingRequest.payload, requestSpec.params)
      ) {
        // return original promise so caller can use the result directly if necessary
        return existingRequest.completed?.promise;
      }

      if (!requestSpec.params) requestSpec.params = {};
      if (
        !requestSpec.passRequestUlidInHeadersOnly &&
        requestSpec.method !== "get"
      ) {
        requestSpec.params.requestUlid = requestUlid;
      }

      // mark the request as pending in the store
      // and attach a deferred promise we'll resolve when completed
      // which we'll use to not make the same request multiple times at the same time, but still be able to await the result
      const completed = createDeferredPromise();
      store.$patch((state) => {
        state.apiRequestStatuses[trackingKey] = {
          requestedAt: new Date(),
          payload: requestSpec.params,
          completed,
          // do not clear "last success at" so we know if this request has ever succeeded
          lastSuccessAt: state.apiRequestStatuses[trackingKey]?.lastSuccessAt,
        };
      });

      // if optimistic update logic is defined, we trigger it here, before actually making the API request
      // that fn should return a fn to call which rolls back any optimistic updates in case the request fails
      let optimisticRollbackFn: OptimisticReturn;
      if (requestSpec.optimistic) {
        optimisticRollbackFn = requestSpec.optimistic(requestUlid);
      }

      const {
        method,
        url,
        params: requestParams,
        options,
        formData,
        onSuccess,
        onFail,
      } = requestSpec;
      let { headers } = requestSpec;
      let _url: string;

      let urlName;
      if (Array.isArray(url)) {
        [_url, urlName] = describePattern(url);
      } else if (typeof url === "string") {
        urlName = url; // string
        _url = url;
      } else {
        throw Error("URL is required");
      }

      const name = `${method?.toUpperCase()} ${urlName}`;
      return tracer.startActiveSpan(name, async (span: Span) => {
        const time = window.performance.getEntriesByType(
          "navigation",
        )[0] as PerformanceNavigationTiming;
        const dns_duration = time.domainLookupEnd - time.domainLookupStart;
        const tcp_duration = time.connectEnd - time.connectStart;
        span.setAttributes({
          "http.body": formData
            ? "multipart form"
            : JSON.stringify(requestParams),
          "http.url": _url,
          "http.method": method,
          "si.requestUlid": requestUlid,
          dns_duration,
          tcp_duration,
          "si.workspace.id": store.workspaceId,
          "si.change_set.id": store.changeSetId,
          ...(formData && requestParams
            ? { "http.params": JSON.stringify(requestParams) }
            : {}),
        });
        try {
          if (!headers) headers = {};
          headers["X-SI-REQUEST-ULID"] = requestUlid;
          opentelemetry.propagation.inject(
            opentelemetry.context.active(),
            headers,
          );

          // the api (axios instance) to use can be set several ways:
          // - passed in with the specific request (probably not common)
          // - use registerApi(api) to create new SpecificApiRequest class with api attached
          // - fallback to default api that was set when initializing the plugin
          const api = requestSpec.api || config.api;

          // add artificial delay - helpful to test loading states in UI when using local API which is very fast
          if (import.meta.env.VITE_DELAY_API_REQUESTS) {
            await promiseDelay(
              parseInt(import.meta.env.VITE_DELAY_API_REQUESTS as string),
            );
          } else if (requestSpec._delay) {
            await promiseDelay(requestSpec._delay);
          }

          // actually trigger the API request (uses the axios instance that was passed in)
          // may need to handle registering multiple apis if we need to hit more than 1

          let request;
          if (method === "get") {
            request = await api({
              method,
              url: _url,
              ...(headers && { headers }),
              params: requestParams,
              ...options,
            });
          } else {
            // delete, post, patch, put. Axios's types forbid formData on the
            // request if method is not one of these , so we have to do branch
            // on the method types to make a formData request
            if (formData) {
              headers["Content-Type"] = "multipart/form-data";
              request = await api({
                method,
                url: _url,
                ...(headers && { headers }),
                data: formData,
                params: requestParams,
                ...options,
              });
            } else {
              request = await api({
                method,
                url: _url,
                ...(headers && { headers }),
                data: requestParams,
                ...options,
              });
            }
          }

          // request was successful if reaching here
          // because axios throws an error if http status >= 400, timeout, etc

          // TODO: trigger global success hook that can be added on plugin init (or split by api)

          // mark request as received, which in absence of an error also means successful
          // TODO: we may want to reverse the order here of calling success and marking received?
          // ideally we would mark received at the same time as the changes made during onSuccess, but not sure it's possible
          store.$patch((state) => {
            state.apiRequestStatuses[trackingKey].lastSuccessAt = new Date();
            state.apiRequestStatuses[trackingKey].receivedAt = new Date();
          });

          // call success handler if one was defined - this will usually be what updates the store
          // we may want to bundle this change together with onSuccess somehow? maybe doesnt matter?
          if (request) {
            if (typeof onSuccess === "function") {
              await onSuccess.call(store, request.data);
            }

            completed.resolve({
              data: request.data,
            });
            span.setAttributes({ "http.status_code": request.status });
          } else {
            completed.resolve({});
          }
          span.end();
          return await completed.promise;

          // normally we want to get any response data from the store directly
          // but there are cases where its useful to be able to get it from the return value
          // like redirecting to a newly created ID, so we return the api response
        } catch (err: any) {
          store.$patch((state) => {
            state.apiRequestStatuses[trackingKey].receivedAt = new Date();
          });

          /* eslint-disable-next-line no-console */
          console.log(err);
          // TODO: trigger global error hook that can be added on plugin init (or split by api)

          // if we made an optimistic update, we'll roll it back here
          if (optimisticRollbackFn) optimisticRollbackFn();

          // call explicit failure handler if one is defined (usually rare)
          if (typeof onFail === "function") {
            const convertedData = onFail(err, requestUlid);

            if (convertedData) {
              err.response = {
                ...err.response,
                data: convertedData,
              };
            }
          }

          // mark the request as failure and store the error info
          store.$patch((state) => {
            const apiRequestStatus = state.apiRequestStatuses[
              trackingKey
            ] as ApiRequestStatus;
            // TODO maybe use Axios.isAxiosError instead, but don't want to change behavior right now
            if (err.response) {
              apiRequestStatus.error = (err as AxiosError).response;
            } else {
              // if error was not http error or had no response body
              // we still want some kind of fallback message to show
              // and we keep it in a similar format to what the http error response bodies
              apiRequestStatus.error = {
                data: {
                  error: {
                    message: err.message,
                  },
                },
              };
            }
          });

          // return false so caller can easily detect a failure
          completed.resolve({
            error: err,
          });
          span.setAttributes({ "http.status_code": err.response.status });
          span.end();
          return await completed.promise;
        }
      });
    }

    async function fireActionResult(
      actionName: string,
      actionResult: ApiRequest,
      requestUlid: RequestUlid,
    ) {
      const request = actionResult;
      const triggerResult = await triggerApiRequest(
        actionName,
        request.requestSpec,
        requestUlid,
      );
      if (!triggerResult) {
        throw new Error(`No trigger result for ${actionName}`);
      }

      if (triggerResult.error) {
        request.setFailedResult(triggerResult.error);
      } else {
        request.setSuccessfulResult(triggerResult.data);
      }
    }

    // wrap each action in a fn that will take an action result that is an ApiRequest
    // and actually trigger the request, waiting to finish until the request is complete
    function wrapApiAction(
      actionName: string,
      originalActionFn: (...args: any[]) => any,
    ) {
      // NOTE - have to be careful here to deal with non-async actions properly
      return async function wrappedActionFn(...args: any[]) {
        const requestUlid = ulid();
        if ("registerRequestsBegin" in store) {
          store.registerRequestsBegin(requestUlid, actionName);
        }
        const actionResult: any = await originalActionFn(...args);
        if (actionResult instanceof ApiRequest) {
          await fireActionResult(actionName, actionResult, requestUlid);
        }
        if ("registerRequestsEnd" in store) {
          store.registerRequestsEnd(requestUlid);
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

    // helper to get the current status of a request in a format that is easy to work with
    const getRequestStatus = (
      requestKey: string,
      ...keyedByArgs: RawRequestStatusKeyArg[]
    ) => {
      return computed(() => {
        const rawKeyedByArgs = _.map(keyedByArgs, unref);
        const fullKey = [requestKey, ..._.compact(rawKeyedByArgs)].join(
          TRACKING_KEY_SEPARATOR,
        );

        const rawStatus = store.$state.apiRequestStatuses[fullKey];
        if (!rawStatus?.requestedAt) {
          return {
            isRequested: false,
            isFirstLoad: false,
            isPending: false,
            isError: false,
            isSuccess: false,
          };
        }
        return {
          ...rawStatus,
          isRequested: true,
          isPending: !rawStatus.receivedAt,
          isFirstLoad: !rawStatus.receivedAt && !rawStatus.lastSuccessAt,
          isSuccess: !!rawStatus.receivedAt && !rawStatus.error,
          isError: !!rawStatus.error,
          ...(rawStatus.error && {
            errorMessage: getApiStatusRequestErrorMessage(rawStatus.error),
            errorCode: rawStatus.error.data?.error?.type,
          }),
        };
      });
    };
    const getRequestStatuses = (
      requestKey: string,
      arrayOfArgs: string[] | ComputedRef<string[]>,
    ) => {
      return computed(() => {
        return _.mapValues(
          _.keyBy(unref(arrayOfArgs)),
          (arg: string) => getRequestStatus(requestKey, arg).value,
        );
      });
    };

    const clearRequestStatus = (
      requestKey: string,
      ...keyedByArgs: RawRequestStatusKeyArg[]
    ) => {
      const rawKeyedByArgs = _.map(keyedByArgs, unref);
      const fullKey = [requestKey, ..._.compact(rawKeyedByArgs)].join(
        TRACKING_KEY_SEPARATOR,
      );

      delete store.$state.apiRequestStatuses[fullKey];
    };

    return {
      getRequestStatus,
      getRequestStatuses,
      clearRequestStatus,
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

type AnyStatus = {
  requestStatus?: ApiRequestStatus;
  asyncState?: UseAsyncStateReturn<unknown, unknown[], boolean>;
};

/** Get the error message from an ApiRequestStatus or UseAsyncState */
export function getErrorMessage({ requestStatus, asyncState }: AnyStatus) {
  return (
    requestStatus?.errorMessage ??
    getApiStatusRequestErrorMessage(
      asyncState?.error.value as ApiRequestStatus["error"],
    ) ??
    (asyncState?.error.value as Error | undefined)?.message
  );
}

export type LoadStatus = "uninitialized" | "loading" | "error" | "success";

/** Get the state of an ApiRequestStatus or UseAsyncState */
export function getLoadStatus({
  requestStatus,
  asyncState,
}: AnyStatus): LoadStatus {
  if (requestStatus?.isPending || asyncState?.isLoading.value) return "loading";
  if (requestStatus?.isError || asyncState?.error.value) return "error";
  if (requestStatus?.isSuccess || asyncState?.isReady.value) return "success";
  return "uninitialized";
}

function getApiStatusRequestErrorMessage(
  error: ApiRequestStatus["error"],
): string | undefined {
  // TODO the statusText bit doesn't seem to ever happen
  return (
    error?.data?.error?.message ||
    error?.data?.message ||
    (error as any)?.statusText
  );
}

/**
 * Turns the response from an API action into an async function
 * that returns data on success and throws error on error.
 */
export async function apiData<T>(request: Promise<ApiRequest<T>>) {
  const { result } = await request;
  if (!result.success) throw result.err;
  return result.data;
}
