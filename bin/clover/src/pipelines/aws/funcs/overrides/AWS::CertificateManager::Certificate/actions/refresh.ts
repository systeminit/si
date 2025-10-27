async function main(component: Input): Promise<Output> {
  const resourceId = component.properties?.si?.resourceId;
  const domain = component.properties?.domain;

  if (!resourceId) {
    return {
      status: "error",
      message: "ResourceId (Certificate ARN) is required for refresh",
    };
  }

  const child = await siExec.waitUntilEnd("aws", [
    "acm",
    "describe-certificate",
    "--certificate-arn",
    resourceId,
    "--region",
    domain?.extra?.Region || "us-east-1",
  ]);

  if (child.exitCode !== 0) {
    console.error(child.stderr);

    if (child.stderr.includes("ResourceNotFoundException")) {
      console.log("Certificate not found upstream, removing resource");
      return {
        status: "ok",
        payload: null,
      };
    }

    return {
      status: "error",
      message: `Refresh error; exit code ${child.exitCode}`,
    };
  }

  const response = JSON.parse(child.stdout);
  const certificate = response.Certificate;

  const payload = {
    CertificateArn: certificate.CertificateArn,
    DomainName: certificate.DomainName,
    SubjectAlternativeNames: certificate.SubjectAlternativeNames,
    Status: certificate.Status,
    Type: certificate.Type,
    KeyAlgorithm: certificate.KeyAlgorithm,
    InUseBy: certificate.InUseBy,
    DomainValidationOptions: certificate.DomainValidationOptions,
    NotBefore: certificate.NotBefore,
    NotAfter: certificate.NotAfter,
    CreatedAt: certificate.CreatedAt,
    IssuedAt: certificate.IssuedAt,
  };

  console.log("Certificate Status:", certificate.Status);

  return {
    payload,
    status: "ok",
  };
}
