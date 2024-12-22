/* eslint-disable @typescript-eslint/no-explicit-any */

import { AxiosError, AxiosInstance, AxiosResponse } from "axios";
import * as _ from "lodash-es";
import { ulid } from "ulid";
import opentelemetry, { Span } from "@opentelemetry/api";
import { UseAsyncStateReturn } from "@vueuse/core";
import {
  promiseDelay,
  createDeferredPromise,
  DeferredPromise,
} from "@si/ts-lib";

const tracer = opentelemetry.trace.getTracer("si-vue");

export type RequestUlid = string;

// types to describe our api request definitions
type OptimisticReturn = (() => void) | void;
type OptimisticFn = (requestUlid: RequestUlid) => OptimisticReturn;

// accepting null | undefined just to allow other parts of the codebase flexibility
// throwing if we ever hit that :(
export type URLPattern = Array<
  string | Record<string, string | undefined | null>
>;

function describePattern(pattern: URLPattern): [string, string] {
  const _url: string[] = [];
  const _urlName: string[] = [];
  pattern.forEach((p) => {
    if (typeof p === "string") {
      _url.push(p);
      _urlName.push(p);
    } else {
      const vals = Object.values(p);
      if (!vals[0]) throw Error(`Bad URLPattern ${pattern} with: ${p}`);
      else _url.push(vals[0]); // url gets the value
      const keys = Object.keys(p);
      if (keys.length > 0) _urlName.push(`:${keys[0]}`); // name gets the str
    }
  });
  return [_url.join("/"), _urlName.join("/")];
}

export interface ApiRequestDescription<
  Response = any,
  RequestParams = Record<string, unknown>,
> {
  /** http request method, defaults to "get" */
  method?: "get" | "patch" | "post" | "put" | "delete"; // defaults to "get" if empty
  /** url to request, or url pattern for improved instrumentation when the url path constains data */
  url?: string | URLPattern;
  /** request data, passed as querystring for GET, body for everything else */
  params?: RequestParams & { requestUlid?: RequestUlid };
  /** if a multipart form is being sent in a put/post/patch */
  formData?: FormData;
  /** function to call if request is successfull (2xx) - usually contains changes to the store */
  onSuccess?(response: Response): Promise<void> | void;
  /**
   * function to call that will run after a new changeset is created as a result of this function
   * Note that in this scenario both funcs are being called on the "original" store,
   * not the new store that will be constructed once you are routed to the new change set
   */
  onNewChangeSet?(
    newChangeSetId: string,
    response: Response,
  ): Promise<void> | void;
  /** function to call if request fails (>=400) - not common */
  onFail?(response: any): any | void;
  /** additional headers to pass with request */
  headers?: Record<string, any>;
  /** additional axios options */
  options?: Record<string, any>; // TODO: pull in axios options type?
  /** optional optimistic update fn to call before api request is made, should return a rollback fn called on api error */
  optimistic?: OptimisticFn;
  /** add artificial delay (in ms) before fetching */
  _delay?: number;
}

// TODO: need to rework these types, and be more flexible... See vue-query for ideas
export type RequestStatusKeyArg = string | number | undefined | null;

/**
 * type describing how we store a single request status
 *
 * Requests are in one of 2 states, determined by `receivedAt` and `error`:
 * - **pending**: receivedAt is undefined.
 * - **completed**: receivedAt is set. If `error` is undefined, it was a success.
 *
 * completed, requestedAt and payload are always set.
 */
interface RawApiRequestStatus<
  Response = any,
  RequestParams = Record<string, unknown>,
> {
  /**
   * When the current request was made.
   * When this is set, we are in the pending, error or success state.
   */
  requestedAt: Date;
  /**
   * When the response was received.
   *
   * When this is set, we are in the error or success state.
   */
  receivedAt?: Date;
  // completedAt?: Date; // REMOVED: unused
  /**
   * The request payload that was sent. Used to determine if a request is a duplicate.
   */
  payload: RequestParams & { requestUlid: RequestUlid };
  /**
   * The error.
   *
   * undefined means there is no error, and it is in the init, pending or success state.
   */
  error?: AxiosResponse | { data: { error: { message: string } } };
  /**
   * A promise that resolves when the request completes, whether it succeeds or not.
   *
   * It will resolve with either:
   * - `{ data: Response }` if the request was successful
   * - `{ error: any }` if the request failed
   */
  completed: DeferredPromise<DataOrError<Response>>;
}

/**
 * Debounced API request status. type describing the computed getter with some convenience properties */
export interface ApiRequestStatus<
  Response = any,
  RequestParams = Record<string, unknown>,
> extends Readonly<Partial<RawApiRequestStatus<Response, RequestParams>>> {
  /**
   * The last time the request was successful (if ever).
   */
  lastSuccessAt?: Date;
  isRequested: boolean;
  isPending: boolean;
  isFirstLoad: boolean;
  isError: boolean;
  isSuccess: boolean;
  errorMessage?: string;
  errorCode?: string;
}

export type DataOrError<Response = any> =
  | { data: Response; error?: undefined }
  | { error: any; data?: undefined };

export class ApiRequestDebouncer<
  Response = any,
  RequestParams = Record<string, unknown>,
> {
  private request?: RawApiRequestStatus;
  private lastSuccessAt?: Date;

  // triggers a named api request passing in a payload
  // this makes the api request, tracks the request status, handles errors, etc
  // TODO: probably will rework this a bit to get better type-checking
  async triggerApiRequest(
    api: AxiosInstance,
    requestSpec: ApiRequestDescription<Response, RequestParams>,
    callbackArg: any,
    extraTracingArgs: {
      "si.workspace.id"?: string;
      "si.change_set.id"?: string;
    },
  ): Promise<DataOrError<Response>> {
    /* eslint-disable no-param-reassign,consistent-return */
    // console.log('trigger api request', actionName, requestSpec);

    if (
      !!this.request &&
      !this.request.receivedAt &&
      _.isEqual(this.request.payload, requestSpec.params)
    ) {
      // return original promise so caller can use the result directly if necessary
      return this.request.completed?.promise;
    }

    const requestUlid = ulid();

    const payload = (requestSpec.params ?? {}) as RequestParams & {
      requestUlid: RequestUlid;
    };
    payload.requestUlid = requestUlid;
    if (!requestSpec.params) requestSpec.params = payload;

    // mark the request as pending in the store
    // and attach a deferred promise we'll resolve when completed
    // which we'll use to not make the same request multiple times at the same time, but still be able to await the result
    const completed = createDeferredPromise<DataOrError<Response>>();
    // store.$patch((state) => {
    this.request = {
      requestedAt: new Date(),
      payload,
      completed,
    };
    // });

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
      onNewChangeSet,
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
        ...extraTracingArgs,
        ...(formData && requestParams
          ? { "http.params": JSON.stringify(requestParams) }
          : {}),
      });
      try {
        if (!headers) headers = {};
        opentelemetry.propagation.inject(
          opentelemetry.context.active(),
          headers,
        );

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

        let response: AxiosResponse<Response>;
        if (method === "get") {
          response = await api({
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
            response = await api({
              method,
              url: _url,
              ...(headers && { headers }),
              data: formData,
              params: requestParams,
              ...options,
            });
          } else {
            response = await api({
              method,
              url: _url,
              ...(headers && { headers }),
              data: requestParams,
              ...options,
            });
          }
        }

        if (response.headers.force_change_set_id)
          if (typeof onNewChangeSet === "function")
            await onNewChangeSet.call(
              callbackArg,
              response.headers.force_change_set_id,
              response.data,
            );

        // request was successful if reaching here
        // because axios throws an error if http status >= 400, timeout, etc

        // TODO: trigger global success hook that can be added on plugin init (or split by api)

        // mark request as received, which in absence of an error also means successful
        // TODO: we may want to reverse the order here of calling success and marking received?
        // ideally we would mark received at the same time as the changes made during onSuccess, but not sure it's possible
        // store.$patch((state) => {
        this.lastSuccessAt = new Date();
        (this.request as RawApiRequestStatus).receivedAt = new Date();
        // });

        // call success handler if one was defined - this will usually be what updates the store
        // we may want to bundle this change together with onSuccess somehow? maybe doesnt matter?
        if (typeof onSuccess === "function") {
          await onSuccess.call(callbackArg, response.data);
        }

        completed.resolve({ data: response.data });
        span.setAttributes({ "http.status_code": response.status });
        span.end();
        return await completed.promise;

        // normally we want to get any response data from the store directly
        // but there are cases where its useful to be able to get it from the return value
        // like redirecting to a newly created ID, so we return the api response
      } catch (err: any) {
        // store.$patch((state) => {
        (this.request as RawApiRequestStatus).receivedAt = new Date();
        // });

        /* eslint-disable-next-line no-console */
        console.log(err);
        // TODO: trigger global error hook that can be added on plugin init (or split by api)

        // if we made an optimistic update, we'll roll it back here
        if (optimisticRollbackFn) optimisticRollbackFn();

        // call explicit failure handler if one is defined (usually rare)
        if (typeof onFail === "function") {
          const convertedData = onFail(err);

          if (convertedData) {
            err.response = {
              ...err.response,
              data: convertedData,
            };
          }
        }

        // mark the request as failure and store the error info
        // TODO maybe use Axios.isAxiosError instead, but don't want to change behavior right now
        if (err.response) {
          (this.request as RawApiRequestStatus).error = (
            err as AxiosError
          ).response;
        } else {
          // if error was not http error or had no response body
          // we still want some kind of fallback message to show
          // and we keep it in a similar format to what the http error response bodies
          (this.request as RawApiRequestStatus).error = {
            data: {
              error: {
                message: err.message,
              },
            },
          };
        }
        // });

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

  getRawStatus() {
    const rawStatus = this.request;
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
      lastSuccessAt: this.lastSuccessAt,
      isRequested: true,
      isPending: !rawStatus.receivedAt,
      isFirstLoad: !rawStatus.receivedAt && !this.lastSuccessAt,
      isSuccess: !!rawStatus.receivedAt && !rawStatus.error,
      isError: !!rawStatus.error,
      ...(rawStatus.error && {
        errorMessage: getApiStatusRequestErrorMessage(rawStatus.error),
        errorCode: rawStatus.error.data?.error?.type,
      }),
    };
  }
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
