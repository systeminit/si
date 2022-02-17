import { Secret } from "@/api/sdf/dal/secret";

export function listSecrets(): Secret[] {
  return [
    {
      id: 1,
      name: "ilikemybutt",
      kind: "DockerHub",
      objectType: "Credential",
      contents: [0],
    },
  ];
}
