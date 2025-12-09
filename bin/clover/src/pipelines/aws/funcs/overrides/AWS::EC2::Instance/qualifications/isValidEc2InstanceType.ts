  async function main(input: Input): Promise < Output > {
      if (!input.domain.InstanceType) {
          return {
              result: "success",
              message: "No instanceType to validate"
          }
      }
      const child = await siExec.waitUntilEnd("aws", [
          "ec2",
          "describe-instance-type-offerings",
          "--location-type",
          "region",
          "--filters",
          `Name=instance-type,Values=${input.domain?.InstanceType}`,
          "--region",
          input.domain?.extra.Region!
      ]);


      if (child.exitCode !== 0) {
          console.error(child.stderr);
          return {
              result: "failure",
              message: "Error from API"
          }
      }

      const output = JSON.parse(child.stdout);
      const valid =
          Array.isArray(output.InstanceTypeOfferings) &&
          output.InstanceTypeOfferings.length > 0;

      if (!valid) {
          return {
              result: "failure",
              message: `Instance Type is not valid for this region: ${input.domain?.extra.Region!}`
          };
      }

      return {
          result: "success",
          message: 'Instance Type is valid for this region'
      };

  }
