import _ from "lodash";
import { makeConsole } from "./console";

// Since a lang-js process only lasts for a single function request, this
// storage will only live for that time also, but every call to a
// Make*RequestStorage points to the same Record instance
const requestStorage: Record<string, unknown> = {};

export const makeMainRequestStorage = () => {
  const getItem = (key: string) => requestStorage[key];
  const getKeys = () => _.keys(requestStorage);

  return { getItem, getKeys };
};

export const makeBeforeRequestStorage = (executionId: string) => {
  const console = makeConsole(executionId);

  const setItem = (key: string, value: unknown) => {
    console.log(`Setting ${key} to requestStorage`);
    requestStorage[key] = value;
  };

  const deleteItem = (key: string) => {
    console.log(`Removing ${key} from requestStorage`);
    delete requestStorage[key];
  };

  return { ...makeMainRequestStorage(), setItem, deleteItem };
};
