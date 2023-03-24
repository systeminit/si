/*
  Tools for dealing with errors in our API
  consists of a custom ApiError class and an error handling middleware

  In general, if an error is expected (even rarely) then we should throw an ApiError
  Otherwise if it is something that should never happen / is totally unexpected, then
  we can let the code explode or throw a regular Error

  All errors caught that are not an instance of ApiError will be logged to sentry

  Examples of using our ApiError class:
  - `throw new ApiError('BadRequest', 'Invalid email address')`
  - `throw new ApiError('Conflict', 'EmailAlreadyTaken', 'Email is already registered')`
  - `throw new ApiError('Conflict', 'EmailUsesOauth', 'Email is registered using oauth', { provider: 'google' })`

  The error handling middlware handles formatting the returned error, hiding details of 500s
  and storing the captured error to pass through to the logging middleware which will send to sentry
*/

import _ from "lodash";
import * as Koa from 'koa';

// copied from https://github.com/jshttp/statuses/blob/master/src/node.json
export const ErrorCodes = Object.freeze({
  // freeze helps limit the TS type of the keys below
  BadRequest: 400,
  Unauthorized: 401,
  PaymentRequired: 402,
  Forbidden: 403,
  NotFound: 404,
  MethodNotAllowed: 405,
  NotAcceptable: 406,
  ProxyAuthenticationRequired: 407,
  RequestTimeout: 408,
  Conflict: 409,
  Gone: 410,
  LengthRequired: 411,
  PreconditionFailed: 412,
  PayloadTooLarge: 413,
  URITooLong: 414,
  UnsupportedMediaType: 415,
  RangeNotSatisfiable: 416,
  ExpectationFailed: 417,
  ImATeapot: 418,
  MisdirectedRequest: 421,
  UnprocessableEntity: 422,
  Locked: 423,
  FailedDependency: 424,
  TooEarly: 425,
  UpgradeRequired: 426,
  PreconditionRequired: 428,
  TooManyRequests: 429,
  RequestHeaderFieldsTooLarge: 431,
  UnavailableForLegalReasons: 451,
  InternalServerError: 500,
  NotImplemented: 501,
  BadGateway: 502,
  ServiceUnavailable: 503,
  GatewayTimeout: 504,
  HTTPVersionNotSupported: 505,
  VariantAlsoNegotiates: 506,
  InsufficientStorage: 507,
  LoopDetected: 508,
  BandwidthLimitExceeded: 509,
  NotExtended: 510,
  NetworkAuthenticationRequired: 511,
});

// use a string union type instead of an enum
// so we still get autocomplete but dont have to import anything
export type HttpErrorCodeNames = keyof typeof ErrorCodes;
type GenericObject = Record<string, unknown>;

export class ApiError extends Error {
  // name + message props exist from base Error class
  public kind: string; // avoiding the name "type"...

  // http status error code name, also used as code unless specific code is set
  public generalKind: HttpErrorCodeNames;
  public specificKind?: string; // optional arbitrary code to expose
  public httpStatusCode: number; // http status code

  public details?: GenericObject; // arbitrary details to expose when serializing error

  // overload the constructor to help make autocomplete clearer
  constructor(generalErrorKind: HttpErrorCodeNames, errorMessage: string);
  constructor(
    generalErrorKind: HttpErrorCodeNames,
    specificErrorKind: string,
    errorMessage?: string,
    errorDetails?: GenericObject,
  );
  constructor(
    generalErrorKind: HttpErrorCodeNames,
    specificErrorKindOrMessage: string,
    errorMessage?: string,
    errorDetails?: GenericObject,
  ) {
    // do some argument shuffling to make the specific error Kind optional
    let specificErrorKind: string | undefined;
    if (!errorMessage) {
      errorMessage = specificErrorKindOrMessage;
      specificErrorKind = undefined;
    } else {
      specificErrorKind = specificErrorKindOrMessage;
    }
    // convert our error code name into an http code
    const httpStatusCode = ErrorCodes[generalErrorKind] || 500;

    super(errorMessage);

    // generic errors expect these to be filled
    this.name = specificErrorKind || generalErrorKind;
    this.kind = specificErrorKind || generalErrorKind;

    this.generalKind = generalErrorKind;
    this.specificKind = specificErrorKind;
    this.httpStatusCode = httpStatusCode;
    this.message = errorMessage;
    this.details = errorDetails;

    if (typeof Error.captureStackTrace === "function") {
      Error.captureStackTrace(this, this.constructor);
    } else {
      this.stack = new Error(errorMessage).stack;
    }
  }
  get expectedError() {
    return this.httpStatusCode < 500;
  }
}

export async function errorHandlingMiddleware(ctx: Koa.Context, next: Koa.Next) {
  // disable koa's built-in ctx.throw with a helpful message
  ctx.throw = (..._args) => {
    throw new Error("Do not use ctx.throw, use `throw new ApiError()` instead");
  };

  try {
    await next();
  } catch (err) {
    ctx.state.capturedError = err;

    // check if it's an ApiError, which signals that this error was not totally unexpected
    if (err instanceof ApiError) {
      ctx.status = err.httpStatusCode;
      // this is the format exposed to clients
      // we can alter it here and vary with api version request if necessary
      ctx.body = {
        ..._.pick(err, "kind", "message", "details"),
      };

      // otherwise, it was unexpected, so we want to respond with a 500
    } else {
      ctx.status = 500;
      // hide all details from users
      ctx.body = {
        kind: "InternalServerError",
        message:
          "An unexpected error occurred - please try again or contact customer service at support@systeminit.com",
      };

      // error object is still attached to ctx.state.capturedError
      // and will be logged by our request logging middleware
    }
  }
}
