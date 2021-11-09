export * from "./session/login";
export * from "./session/is_authenticated";
import { login } from "./session/login";
import { isAuthenticated } from "./session/is_authenticated";
import { logout } from "./session/logout";
import { getDefaults } from "./session/get_defaults";

export const SessionService = {
  login,
  isAuthenticated,
  logout,
  getDefaults,
};
