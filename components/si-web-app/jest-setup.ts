import "@testing-library/jest-dom";
import "@/plugins/vue-js-modal";

import fetch from "node-fetch";
import { Request } from "node-fetch";
// @ts-ignore
global.fetch = fetch;
// @ts-ignore
global.Request = Request;
// @ts-ignore
global.document.body.createTextRange = function() {
  return {
    setEnd: function() {},
    setStart: function() {},
    getBoundingClientRect: function() {
      return { right: 0 };
    },
    getClientRects: function() {
      return {
        length: 0,
        left: 0,
        right: 0,
      };
    },
  };
};

process.on("unhandledRejection", err => {
  fail(err);
});

jest.setTimeout(45000);
