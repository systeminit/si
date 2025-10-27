async function main(component: Input): Promise<Output> {
  if (component.properties.resource?.payload) {
    return {
      status: "error",
      message: "Resource already exists",
      payload: component.properties.resource.payload,
    };
  }

  const domain = component.properties?.domain;
  const domainName = domain?.DomainName;
  const validationMethod = domain?.ValidationMethod || "DNS";

  if (!domainName) {
    return {
      status: "error",
      message: "DomainName is required",
    };
  }

  const args = [
    "acm",
    "request-certificate",
    "--domain-name",
    domainName,
    "--validation-method",
    validationMethod,
  ];

  if (
    domain?.SubjectAlternativeNames && domain.SubjectAlternativeNames.length > 0
  ) {
    args.push("--subject-alternative-names");
    domain.SubjectAlternativeNames.forEach((san: string) => {
      args.push(san);
    });
  }

  if (
    domain?.DomainValidationOptions && domain.DomainValidationOptions.length > 0
  ) {
    const options = domain.DomainValidationOptions.map((opt: any) => ({
      DomainName: opt.DomainName,
      ValidationDomain: opt.ValidationDomain,
    })).filter((opt: any) => opt.ValidationDomain);

    if (options.length > 0) {
      args.push("--domain-validation-options");
      args.push(JSON.stringify(options));
    }
  }

  if (domain?.CertificateAuthorityArn) {
    args.push("--certificate-authority-arn");
    args.push(domain.CertificateAuthorityArn);
  }

  if (domain?.KeyAlgorithm) {
    args.push("--key-algorithm");
    args.push(domain.KeyAlgorithm);
  }

  args.push("--region");
  args.push(domain?.extra?.Region || "us-east-1");

  const child = await siExec.waitUntilEnd("aws", args);

  if (child.exitCode !== 0) {
    console.error(child.stderr);
    return {
      status: "error",
      message:
        `Unable to create; AWS CLI exited with non zero code: ${child.exitCode}`,
    };
  }

  const response = JSON.parse(child.stdout);
  const certificateArn = response.CertificateArn;

  console.log("Certificate ARN:", certificateArn);

  return {
    resourceId: certificateArn,
    status: "ok",
  };
}
