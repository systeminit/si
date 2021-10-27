import { ISiStorable } from "@/api/sdf/model/siStorable";
import _ from "lodash";
import { Base64 } from "js-base64";

export interface IPublicKey {
  id: string;
  name: string;
  publicKey: Uint8Array | string;
  siStorable: ISiStorable;
}

export class PublicKey implements IPublicKey {
  id: IPublicKey["id"];
  name: IPublicKey["name"];
  publicKey: Uint8Array;
  siStorable: IPublicKey["siStorable"];

  constructor(args: IPublicKey) {
    this.id = args.id;
    this.name = args.name;
    if (_.isString(args.publicKey)) {
      this.publicKey = Base64.toUint8Array(args.publicKey);
    } else {
      this.publicKey = args.publicKey;
    }
    this.siStorable = args.siStorable;
  }

  static upgrade(obj: PublicKey | IPublicKey): PublicKey {
    if (obj instanceof PublicKey) {
      return obj;
    } else {
      return new PublicKey(obj);
    }
  }
}
