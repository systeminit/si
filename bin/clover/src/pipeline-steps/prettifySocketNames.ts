import _logger from "../logger.ts";
import _ from "npm:lodash";
import { ExpandedPkgSpec } from "../spec/pkgs.ts";
import { setAnnotationOnSocket } from "../spec/sockets.ts";

export function prettifySocketNames(
  specs: ExpandedPkgSpec[],
): ExpandedPkgSpec[] {
  const newSpecs = [] as ExpandedPkgSpec[];

  for (const spec of specs) {
    const sockets = spec.schemas[0].variants[0].sockets;

    for (const socket of sockets) {
      const newName = socket.name
        // separate any sequence of lowercase letters followed by an uppercase letter
        .replace(/([a-z])([A-Z])/g, "$1 $2")
        // Separate any sequence of more than 1 of uppercase letters (acronyms) from the next word
        .replace(/([A-Z]+)([A-Z][a-z])/g, "$1 $2");

      socket.name = newName;
      socket.data.name = newName;

      setAnnotationOnSocket(socket, newName);
    }

    newSpecs.push(spec);
    break;
  }

  return newSpecs;
}
