import { Module } from "vuex";
import * as jwtLib from "jsonwebtoken";

import { GetCurrentError } from "@/store";
import { User, IUserLoginRequest } from "@/api/sdf/model/user";
import { sdf } from "@/api/sdf";
import { wipe } from "@/api/sdf/dexie";

export interface UserStore {
  current: null | User;
}

export const user: Module<UserStore, any> = {
  namespaced: true,
  state: {
    current: null,
  },
  getters: {
    current(state): User {
      if (state.current) {
        return state.current;
      } else {
        throw new GetCurrentError("user");
      }
    },
  },
  mutations: {
    current(state, payload: User | null) {
      state.current = payload;
    },
  },
  actions: {
    async isAuthenticated({ dispatch, state, commit }): Promise<boolean> {
      const token = sdf.token;
      if (token) {
        let currentTime = Math.floor(Date.now() / 1000);
        let decodedToken = jwtLib.decode(token, {
          complete: true,
        }) as any;
        if (decodedToken && currentTime >= decodedToken["payload"]["exp"]) {
          await dispatch("logout");
          return false;
        }
        // If we made it this far, we have a valid token, but we don't have the
        // associated user object. Lets fix that.
        if (state.current == null) {
          if (decodedToken["payload"]["userId"]) {
            let user = await User.get({
              id: decodedToken["payload"]["userId"],
            });
            commit("current", user);
          }
        }
        return true;
      } else {
        return false;
      }
    },
    async login({ commit }, payload: IUserLoginRequest): Promise<void> {
      const user = await User.login(payload);
      commit("current", user);
    },
    async logout({ commit, state, dispatch }): Promise<void> {
      if (state.current) {
        await User.upgrade(state.current).logout();
        commit("current", null);
        await dispatch("loader/clear", {}, { root: true });
        await dispatch("application/clear", {}, { root: true });
        await dispatch("editor/clear", {}, { root: true });
        localStorage.removeItem("vuex");
      }
    },
    async fromDb({ commit, state }, payload: User): Promise<void> {
      if (state.current?.id === payload.id) {
        commit("current", payload);
      }
    },
  },
};
