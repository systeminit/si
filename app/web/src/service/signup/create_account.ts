import { ApiResponse, SDF } from "@/api/sdf";
import Bottle from "bottlejs";
import { Observable } from "rxjs";

export interface CreateAccountRequest {
  billingAccountName: string;
  userName: string;
  userEmail: string;
  userPassword: string;
  signupSecret: string;
}

export interface CreateAccountResponse {
  success: boolean;
}

export function createAccount(
  request: CreateAccountRequest,
): Observable<ApiResponse<CreateAccountResponse>> {
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;

  return sdf.post<ApiResponse<CreateAccountResponse>>(
    "signup/create_account",
    request,
  );
}
