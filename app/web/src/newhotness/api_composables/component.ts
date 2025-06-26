import { ComponentId } from "@/api/sdf/dal/component";

export type UpdateComponentAttributesArgs = Record<
  AttributeJsonPointer,
  SetAttributeTo
>;

export type ComponentIdType =
  | {
      schemaType: string;
      schemaVariantId: string;
    }
  | {
      schemaType: string;
      schemaId: string;
    };
export type CreateComponentPayload = ComponentIdType & {
  parentId: null;
  x: "0";
  y: "0";
  height: "0";
  width: "0";
};
export const createComponentPayload = (
  idType: ComponentIdType,
): CreateComponentPayload => {
  if (
    ("schemaId" in idType && !idType.schemaId) ||
    ("schemaVariantId" in idType && !idType.schemaVariantId)
  )
    throw new Error("schemaId or schemaVariantId required");
  return {
    ...idType,
    parentId: null,
    x: "0",
    y: "0",
    height: "0",
    width: "0",
  };
};

// Things you can set an attribute to
export type SetAttributeTo =
  // Set attribute to a static JS value (can be any JSON--object, array, string, number, boolean, null)
  | unknown
  // Set attribute to a subscription (another component's value feeds it)
  | {
      $source: "subscription";
      component: ComponentId | string;
      path: AttributeJsonPointer;
    }
  // Unset the value by not passing "value" field
  | { $source: "value"; value?: undefined }
  // Set attribute to a static JS value (use this to safely set object values that could have "$source" property in them)
  | { $source: "value"; value: unknown };

// JSON pointer to the attribute, relative to the component root (e.g. /domain/IpAddresses/0 or /si/name)
export type AttributeJsonPointer = string;

export type UpdateComponentNameArgs = {
  name: string;
};

export type UpdateComponentManageArgs = {
  componentId: string;
};

export enum SecretVersion {
  V1 = "v1",
}

export enum SecretAlgorithm {
  Sealedbox = "sealedbox",
}
export interface CreateSecret {
  attributeValueId: string;
  propId: string;
  name: string;
  definition: string;
  description?: string;
  crypted: number[];
  keyPairPk: string;
  version: SecretVersion;
  algorithm: SecretAlgorithm;
}

export interface PublicKey {
  /**
   * The PK of the public key
   */
  pk: string;
  /**
   * The name of the public key
   */
  name: string;
  /**
   * The public key contents, encoded as a Base64 string
   *
   * # Examples
   *
   * Decoding a public key into a `Uint8Array` type:
   *
   * ```ts
   * Base64.toUint8Array(key.public_key);
   * ```
   */
  public_key: string;
  /**
   * A created lamport clock, used to sort multiple generations of key pairs
   */
  created_lamport_clock: string;

  created_at: string;
  updated_at: string;
}
