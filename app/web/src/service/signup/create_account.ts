import { ApiResponse } from "@/api/sdf";
import Bottle from "bottlejs";

export interface CreateAccountRequest {
  billingAccountName: string;
  userName: string;
  userEmail: string;
  userPassword: string;
}

export interface CreateAccountResponse {
  success: boolean;
}

export async function createAccount(
  request: CreateAccountRequest,
): Promise<ApiResponse<CreateAccountResponse>> {
  const bottle = Bottle.pop("default");
  const sdf = bottle.container.SDF;

  const response = await sdf.post("signup/create_account", request);
  return response;
}
