import { ExpandedPkgSpecWithSockets } from "../spec/pkgs.ts";
import { bfsPropTree } from "../spec/props.ts";
import { setAnnotationOnSocket } from "../spec/sockets.ts";

export function prettifySocketNames(
  specs: readonly ExpandedPkgSpecWithSockets[],
) {
  for (const spec of specs) {
    const { variants: [variant] } = spec.schemas[0];
    const sockets = variant.sockets;

    for (const socket of sockets) {
      const newName = toSpaceCase(socket.name);

      socket.name = newName;
      socket.data.name = newName;

      setAnnotationOnSocket(socket, newName);
    }

    bfsPropTree([variant.domain, variant.resourceValue], (prop) => {
      if (prop.data.inputs) {
        for (const input of prop.data.inputs) {
          if (input.kind !== "prop") {
            input.socket_name = toSpaceCase(input.socket_name);
          }
        }
      }
    });
  }
}

function toSpaceCase(name: string) {
  return name
    // separate any sequence of lowercase letters followed by an uppercase letter
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    // Separate any sequence of more than 3 of uppercase letters (acronyms) from the next word
    .replace(/([A-Z]{3,})([A-Z][a-z])/g, "$1 $2");
}
