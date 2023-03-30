import { StandardModel } from "@/api/sdf/dal/standard_model";

export interface User extends StandardModel {
  name: string;
  email: string;
  // TODO should be camelcase, but changing backend is annoying
  picture_url?: string;
}
