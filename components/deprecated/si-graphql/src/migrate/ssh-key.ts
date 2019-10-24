import { Integration } from "@/datalayer/integration";
import { SshKey, SshKeyComponent } from "@/datalayer/component/ssh-key";

export async function getSshKeyData(): Promise<SshKeyComponent[]> {
  const globalIntegration = await Integration.getByName("Global");
  const keyTypes = ["RSA", "DSA", "ECDSA", "ED25519"];
  const keyFormats = ["RFC4716", "PKCS8", "PEM"];
  const bitLengths = {
    RSA: [1024, 2048, 3072, 4096],
    DSA: [1024],
    ECDSA: [256, 384, 521],
    ED25519: [256],
  };

  const uuidStub = "669e70b-1010-479d-97d8-38270596c";
  let uuidCounter = 100;

  const data = [];
  for (const keyType of keyTypes) {
    for (const keyFormat of keyFormats) {
      for (const bitLength of bitLengths[keyType]) {
        uuidCounter = uuidCounter + 1;
        data.push(
          SshKey.New({
            id: `${uuidStub}${uuidCounter}`,
            name: `${keyType} ${bitLength} Bits ${keyFormat}`,
            description: `${keyType} ${bitLength} Bits ${keyFormat} SSH Key`,
            rawDataJson: "{}",
            integrationId: globalIntegration.fqId,
            supportedActions: [
              "create",
              "delete",
              "changePassphrase",
              "unlock",
            ],
            keyType: keyType as SshKeyComponent["keyType"],
            keyFormat: keyFormat as SshKeyComponent["keyFormat"],
            bits: bitLength,
          }),
        );
      }
    }
  }

  return data;
}
