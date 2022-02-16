import { SecretKind } from "@/api/sdf/dal/secret";

export function listSecretKinds(): SecretKind[] {
  return [
    {
      name: "Docker Hub",
      type: "Credential",
      fields: [
        {
          name: "Docker Hub Username",
          password: false,
        },
        {
          name: "Docker Hub Password",
          password: true,
        },
      ],
    },
  ];
}
