/* eslint-disable @typescript-eslint/no-explicit-any */

import axios, { AxiosError, AxiosInstance, AxiosResponse } from "axios";
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

export class ApiRequest<
  Response = any,
  RequestParams = Record<string, unknown>,
> {
  constructor(
    public requestSpec: ApiRequestDescription<Response, RequestParams>,
  ) {
    if (!this.requestSpec.api) {
      this.requestSpec.api = (this.constructor as any).api;
    }
    if (!this.requestSpec.method) this.requestSpec.method = "get";
  }

  protected _result?: ApiResult<Response>;

  /**
   * The result of the request.
   */
  get result(): ApiResult<Response> {
    if (this._result === undefined) throw this.unrequestedError();
    return this._result;
  }

  // ApiRequestStatus fields
  /**
   * A promise that resolves when the request completes, whether it succeeds or not.
   *
   * It will resolve with either:
   * - `{ data: Response }` if the request was successful
   * - `{ error: any }` if the request failed
   */
  get completed(): DeferredPromise<ApiResult<Response>> {
    throw this.unrequestedError();
  }
  /**
   * When the current request was made.
   * When this is set, we are in the pending, error or success state.
   */
  get requestedAt(): Date {
    throw this.unrequestedError();
  }
  /**
   * The request payload that was sent.
   *
   * Used to determine if a request is a duplicate.
   */
  get payload(): Record<string, unknown> & { requestUlid: RequestUlid } {
    throw this.unrequestedError();
  }
  /**
   * When the response was received.
   *
   * When this is set, we are in the error or success state.
   */
  get receivedAt(): Date | undefined {
    throw this.unrequestedError();
  }
  // completedAt?: Date; // REMOVED: unused
  /**
   * The error.
   *
   * undefined means there is no error, and it is in the init, pending or success state.
   */
  get error():
    | AxiosResponse<any, any>
    | {
        data: {
          error: {
            message: string;
          };
        };
      }
    | undefined {
    if (this.result?.success !== false) return undefined;
    // TODO maybe use Axios.isAxiosError instead, but don't want to change behavior right now
    if (this.result.err.response) {
      return (this.result.err as AxiosError).response;
    } else {
      // if error was not http error or had no response body
      // we still want some kind of fallback message to show
      // and we keep it in a similar format to what the http error response bodies
      return {
        data: {
          error: {
            message: (this.result.err as Error).message,
          },
        },
      };
    }
    // });
  }

  /**
   * The error message for this request.
   */
  get errorMessage() {
    return getApiStatusRequestErrorMessage(this.error);
  }
  /**
   * The error code for this request.
   */
  get errorCode() {
    return this.error?.data?.error?.type;
  }
  /**
   * The response data from this request.
   *
   * If the response type could be undefined, you must check success to determine if the data
   * has actually been loaded or not.
   */
  get data(): Response | undefined {
    return this.result?.data;
  }

  // eslint-disable-next-line class-methods-use-this
  protected unrequestedError(): Error {
    throw new Error("You must await the ApiRequest before getting the result");
  }

  static send<Response = any, RequestParams = Record<string, unknown>>(
    defaultApi: AxiosInstance,
    requestSpec: ApiRequestDescription<Response, RequestParams>,
    callbackArg: any,
    extraTracingArgs: {
      "si.workspace.id"?: string;
      "si.change_set.id"?: string;
    },
  ): TrackedApiRequest<Response, RequestParams> {
    return new TrackedApiRequest(
      defaultApi,
      requestSpec,
      callbackArg,
      extraTracingArgs,
    );
  }
}

class TrackedApiRequest<Response, RequestParams> extends ApiRequest<
  Response,
  RequestParams
> {
  #completed = createDeferredPromise<ApiResult<Response>>();
  #requestedAt = new Date();
  private requestUlid = ulid();
  #payload: RequestParams & { requestUlid: RequestUlid };
  #receivedAt?: Date;

  constructor(
    defaultApi: AxiosInstance,
    requestSpec: ApiRequestDescription<Response, RequestParams>,
    callbackArg: any,
    extraTracingArgs: {
      "si.workspace.id"?: string;
      "si.change_set.id"?: string;
    },
  ) {
    super(requestSpec);

    this.#payload = (requestSpec.params ?? {}) as RequestParams & {
      requestUlid: RequestUlid;
    };
    this.#payload.requestUlid = this.requestUlid;
    if (!requestSpec.params) requestSpec.params = this.#payload;

    // eslint-disable-next-line @typescript-eslint/no-floating-promises
    this.send(defaultApi, callbackArg, extraTracingArgs);
  }

  // ApiRequestStatus fields
  override get completed() {
    return this.#completed;
  }
  override get requestedAt() {
    return this.#requestedAt;
  }
  override get payload() {
    return this.#payload;
  }
  override get receivedAt() {
    return this.#receivedAt;
  }

  // triggers a named api request passing in a payload
  // this makes the api request, tracks the request status, handles errors, etc
  // TODO: probably will rework this a bit to get better type-checking
  private async send(
    defaultApi: AxiosInstance,
    callbackArg: any,
    extraTracingArgs: {
      "si.workspace.id"?: string;
      "si.change_set.id"?: string;
    },
  ) {
    const { requestSpec } = this;
    const api = requestSpec.api ?? defaultApi;

    // if optimistic update logic is defined, we trigger it here, before actually making the API request
    // that fn should return a fn to call which rolls back any optimistic updates in case the request fails
    let optimisticRollbackFn: OptimisticReturn;
    if (requestSpec.optimistic) {
      optimisticRollbackFn = requestSpec.optimistic(this.requestUlid);
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
        "si.requestUlid": this.requestUlid,
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
        this.#receivedAt = new Date();
        // });

        // call success handler if one was defined - this will usually be what updates the store
        // we may want to bundle this change together with onSuccess somehow? maybe doesnt matter?
        if (typeof onSuccess === "function") {
          await onSuccess.call(callbackArg, response.data);
        }

        this._result = { success: true, data: response.data };
        this.completed.resolve(this.result);
        span.setAttributes({ "http.status_code": response.status });
        span.end();
        return this.result;

        // normally we want to get any response data from the store directly
        // but there are cases where its useful to be able to get it from the return value
        // like redirecting to a newly created ID, so we return the api response
      } catch (err: Error | AxiosError | any) {
        // store.$patch((state) => {
        this.#receivedAt = new Date();
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

        // return false so caller can easily detect a failure
        this._result = {
          success: false,
          // the raw error object - usually an AxiosError
          err,
          error: err,
          // the (json) body of the failed request, if applicable
          ...(axios.isAxiosError(err) && {
            errBody: err.response?.data,
            statusCode: err.response?.status,
          }),
        };
        this.completed.resolve(this.result);
        // TODO if the error is *not* an AxiosError, we throw instead of ending the span!
        span.setAttributes({ "http.status_code": err.response.status });
        span.end();
        return this.result;
      }
    });
  }

  static noop = Symbol("API_REQUEST_NOOP");
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
  /** API to use for this request (if not, a default will be provided) */
  api?: AxiosInstance;
  /** additional args to key the request status */
  keyRequestStatusBy?: RequestStatusKeyArg | RequestStatusKeyArg[];
}

// TODO: need to rework these types, and be more flexible... See vue-query for ideas
export type RequestStatusKeyArg = string | number | undefined | null;

/**
 * Status of a debounced request.
 *
 * Requests are in one of 2 states, determined by `receivedAt` and `error`:
 * - **pending**: receivedAt is undefined.
 * - **completed**: receivedAt is set. If `error` is undefined, it was a success.
 *
 * completed, requestedAt and payload are always set.
 */
export interface ApiRequestStatus<
  Response = any,
  RequestParams = Record<string, unknown>,
> extends Pick<
    Partial<ApiRequest<Response, RequestParams>>,
    | "completed"
    | "requestedAt"
    | "receivedAt"
    | "payload"
    | "result"
    | "error"
    | "errorMessage"
    | "errorCode"
    | "data"
  > {
  /**
   * The last time the request was successful (if ever).
   */
  lastSuccessAt?: Date;
  /**
   * Whether this request has ever been made.
   */
  isRequested: boolean;
  /**
   * Whether this request is pending (has not completed).
   */
  isPending: boolean;
  /**
   * Whether the request has ever completed (true if it is not currently an error and has never succeeded).
   */
  isFirstLoad: boolean;
  /** Whether this request was an error. */
  isError: boolean;
  /** Whether this request was successful. */
  isSuccess: boolean;
}

export type ApiResult<Response = any> =
  | {
      /** Whether the request was successful. */
      success: true;
      /** The data in the successful response. */
      data: Response;
    }
  | {
      /** Whether the request was successful. */
      success: false;
      /** The error that was thrown on failure. */
      err: Error | AxiosError | any;
      /** The error that was thrown on failure. */
      // A copy of the error with a different name, since we use the same object for
      // ApiRequest.result and for the result of the promise.
      error: Error | AxiosError | any;
      /** The body of the error response. */
      errBody?: any;
      /** The HTTP status code of the response (or undefined if it was not an HTTP error). */
      statusCode?: AxiosError["status"];
      /** The data in the successful response. */
      data?: Response extends undefined ? never : undefined;
    };

export class ApiRequestDebouncer<
  Response = any,
  RequestParams = Record<string, unknown>,
> implements ApiRequestStatus<Response, RequestParams>
{
  private request?: TrackedApiRequest<Response, RequestParams>;
  private lastSuccess?: TrackedApiRequest<Response, RequestParams>;

  // triggers a named api request passing in a payload
  // this makes the api request, tracks the request status, handles errors, etc
  // TODO: probably will rework this a bit to get better type-checking
  trigger(
    defaultApi: AxiosInstance,
    request: ApiRequest<Response, RequestParams>,
    callbackArg: any,
    extraTracingArgs: {
      "si.workspace.id"?: string;
      "si.change_set.id"?: string;
    },
  ) {
    // console.log('trigger api request', actionName, requestSpec);

    if (
      !!this.request &&
      !this.request.receivedAt &&
      _.isEqual(this.request.payload, request.requestSpec.params)
    ) {
      // return original promise so caller can use the result directly if necessary
      return this.request;
    }

    const trackedRequest = ApiRequest.send(
      defaultApi,
      request.requestSpec,
      callbackArg,
      extraTracingArgs,
    );
    this.request = trackedRequest;
    // eslint-disable-next-line @typescript-eslint/no-floating-promises
    trackedRequest.completed.promise.then((result) => {
      // If the request succeeds, set this.lastSuccess
      if (result.success) {
        this.lastSuccess = trackedRequest;
      }
    });
    return trackedRequest;
  }

  // ApiRequestStatus fields
  get completed() {
    return this.request?.completed;
  }
  get requestedAt() {
    return this.request?.requestedAt;
  }
  get payload() {
    return this.request?.payload;
  }
  get receivedAt() {
    return this.request?.receivedAt;
  }
  // completedAt?: Date; // REMOVED: unused
  get error() {
    return this.request?.error;
  }
  get lastSuccessAt() {
    return this.lastSuccess?.receivedAt;
  }
  get isRequested() {
    return this.request !== undefined;
  }
  get isFirstLoad() {
    return this.isPending && !this.lastSuccess;
  }
  get isPending() {
    return this.request?.result === undefined;
  }
  get isError() {
    return this.request?.result?.success === false;
  }
  get isSuccess() {
    return this.request?.result?.success === true;
  }
  get errorMessage() {
    return this.request?.errorMessage;
  }
  get errorCode() {
    return this.request?.errorCode;
  }
  get data() {
    return this.lastSuccess?.data;
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
