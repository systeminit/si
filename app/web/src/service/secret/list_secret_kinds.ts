import { SecretKind } from "@/api/sdf/dal/secret";

export function listSecretKinds(): SecretKind[] {
  return [
    {
      name: "Docker Hub",
      objectType: "Credential",
      fields: [
        {
          name: "username",
          displayName: "Docker Hub Username",
          password: false,
        },
        {
          name: "password",
          displayName: "Docker Hub Password",
          password: true,
        },
      ],
    },
  ];
}
