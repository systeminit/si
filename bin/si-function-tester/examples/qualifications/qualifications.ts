type Input = {
  domain: {
    region?: string;
    Filters?: Array<{ Name: string; Value: string }>;
    ExecutableUsers?: string;
    Owners?: string;
    ImageId?: string;
    UseMostRecent?: boolean;
  };
};

type Output = {
  result: "success" | "failure";
  message: string;
};

async function main(
  {
    domain: {
      region,
      Filters,
      ExecutableUsers,
      Owners,
      ImageId,
      UseMostRecent,
    },
  }: Input,
): Promise<Output> {
  async function awsEc2DescribeImages(region: string, queryArgs: string[]) {
    const response = await siExec.waitUntilEnd("aws", [
      "ec2",
      "describe-images",
      "--region",
      region ?? "",
      "--query",
      "reverse(sort_by(Images, &CreationDate))",
      ...queryArgs,
    ]);
    if (response.exitCode !== 0) {
      throw new Error(
        `aws command failed with exit code ${response.exitCode}. Output:\n${response.all}`,
      );
    }
    const images = JSON.parse(response.stdout) as { ImageId: string }[];
    return images.map((image) => image.ImageId);
  }

  if (!region) {
    return {
      result: "failure",
      message: "You must specify a region",
    };
  }
  const queryArgs = [];
  for (const f of Filters ?? []) {
    queryArgs.push("--filters", `Name=${f.Name},Values=${f.Value}`);
  }
  if (ExecutableUsers) queryArgs.push("--executable-users", ExecutableUsers);
  if (Owners) queryArgs.push("--owners", Owners);

  // If there is a query to be done, check that it yields the specified image.
  if (queryArgs.length > 0) {
    const imageIds = await awsEc2DescribeImages(region, queryArgs);
    if (imageIds.length > 1 && !UseMostRecent) {
      return {
        result: "failure",
        message:
          "Multiple images match the query! Either narrow the query, or set UseMostRecent=true.",
      };
    }
    if (imageIds.length == 0) {
      return {
        result: "failure",
        message: "No images match the query!",
      };
    }
    if (ImageId != imageIds[0]) {
      return {
        result: "failure",
        message: `ImageId incorrect or out of date (query returned ${
          imageIds[0]
        })`,
      };
    }
    return {
      result: "success",
      message: "Query returns the correct image",
    };

    // If there is no query to be done, check if ImageId exists
  } else if (ImageId) {
    const imageIds = await awsEc2DescribeImages(region, [
      "--filter",
      `Name=image-id,Values=${ImageId}`,
    ]);
    if (imageIds[0] != ImageId) {
      return {
        result: "failure",
        message: "Image ID does not exist",
      };
    }
    return {
      result: "success",
      message: "Image ID exists",
    };

    // If there is no ImageId, just return success--there's no query to be done here)
  } else {
    return {
      result: "success",
      message: "New empty AMI component",
    };
  }
}

export default main;
