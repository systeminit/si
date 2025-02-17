import * as _ from "https://deno.land/x/lodash_es@v0.0.2/mod.ts";

// Since a lang-js process only lasts for a single function request, this
// storage will only live for that time also, but every call to a
// Make*RequestStorage points to the same Record instance
// In bin/lang-js/src/sandbox/requestStorage.ts
export type RequestStorage = {
  data: Record<string, unknown>;
  env: Record<string, string>;
};

const requestStorage: RequestStorage = {
  data: {},
  env: {},
};

export const rawStorage = (): RequestStorage => requestStorage;

export const makeMainRequestStorage = () => {
  const rawStorage = (): RequestStorage => requestStorage;
  const getEnv = (key: string) => requestStorage.env[key];
  const getItem = (key: string) => requestStorage.data[key];
  const getEnvKeys = () => _.keys(requestStorage.env);
  const getKeys = () => _.keys(requestStorage.data);

  return {
    rawStorage,
    getEnv,
    getItem,
    getEnvKeys,
    getKeys,
  };
};

export const makeBeforeRequestStorage = (_executionId: string) => {
  const setEnv = (key: string, value: string) => {
    console.log(`Registering environment variable ${key}`);
    requestStorage.env[key] = value;
  };

  const setItem = (key: string, value: unknown) => {
    console.log(`Setting ${key} to requestStorage`);
    requestStorage.data[key] = value;
  };

  const deleteEnv = (key: string) => {
    console.log(`Removing environment variable ${key}`);
    delete requestStorage.env[key];
  };

  const deleteItem = (key: string) => {
    console.log(`Removing ${key} from requestStorage`);
    delete requestStorage.data[key];
  };

  return {
    ...makeMainRequestStorage(),
    setEnv,
    setItem,
    deleteEnv,
    deleteItem,
  };
};

export const toJSON = () => JSON.stringify(rawStorage());
