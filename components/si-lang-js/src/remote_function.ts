export interface RemoteFunctionRequestResolver {
  kind: "resolver";
  code: string;
  containerImage: string;
  containerTag: string;
}

export type RemoteFunctionRequest = RemoteFunctionRequestResolver;

export interface RemoteFunctionOutputLine {
  stream: "stdout" | "stderr";
  level: "debug" | "info" | "warn" | "error";
  group: string;
  message: string;
}

export interface RemoteFunctionResultFailure {
  status: "failure";
  kind: string;
  error: {
    message: string;
  };
  data?: never;
}

export interface RemoteFunctionResultResolver {
  status: "success";
  kind: "resolver";
  error?: never;
  data: unknown;
}

export type RemoteFunctionResult =
  | RemoteFunctionRequestResolver
  | RemoteFunctionResultFailure;
