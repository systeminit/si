import { ActorAndTimestamp } from "./components.store";

export type SecretId = string;
export type DefinitionId = string;

export type Secret = {
  id: SecretId;
  definition: DefinitionId;
  name: string;
  description: string;
  createdInfo: ActorAndTimestamp;
  updatedInfo: ActorAndTimestamp;
};

// backend endpoint
// give me all secrets organized by definition id
// create a secret of the given defintion id with the given encrypted payload

// store functions
// give me all secrets of one definition id
// create a secret of the given defintion id (this includes encrypting the secret itself and sending it to the backend)

// next turn of the crank -
// delete a secret
// replace a secret
