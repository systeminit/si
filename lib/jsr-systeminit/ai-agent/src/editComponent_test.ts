import { assertEquals } from "@std/assert";
import { editComponent, proposeEdits } from "../mod.ts";

if (Deno.env.has("OPENAI_API_KEY")) {
  Deno.test("editComponent - adds Name tag to EC2 instance", async () => {
    // Create a test component
    const componentName = "wordpressInstance";
    const kind = "AWS::EC2::Instance";
    const properties = {
      si: {
        name: "WordPress EC2 Instance",
        type: "aws-ec2-instance",
      },
      domain: {
        InstanceType: "t3.medium",
        KeyName: "my-key-pair",
        ImageId: "ami-0abcdef1234567890",
        SubnetId: "subnet-0123456789abcdef0",
        SecurityGroupIds: [
          "sg-0123456789abcdef0",
        ],
        UserData:
          "#!/bin/bash\napt update\napt install -y apache2 php php-mysql",
      },
    };

    // Define the edit request
    const request =
      "I want to add a tag Name that is the same as the property si/name";

    // Call the editComponent function
    const result = await editComponent(
      componentName,
      kind,
      properties,
      request,
    );
    console.log(JSON.stringify(result, null, 2));

    // Check that we have an update operation for the component
    assertEquals(result.status, "ok");
    assertEquals(!!result.ops.update[componentName], true);

    // Check that the Tags property was added with a Name tag matching the si.name
    const updatedProperties = result.ops.update[componentName].properties;
    assertEquals(!!updatedProperties.domain, true);

    if (
      updatedProperties.domain && Array.isArray(updatedProperties.domain.Tags)
    ) {
      const nameTag = updatedProperties.domain.Tags.find((tag) =>
        tag.Key === "Name"
      );
      assertEquals(!!nameTag, true);
      assertEquals(nameTag.Value, properties.si.name);
    } else {
      throw new Error("Tags property was not properly updated");
    }
  });

  Deno.test("editComponent - handles non-AWS component types", async () => {
    // Create a test component with a non-AWS kind
    const componentName = "genericFrame";
    const kind = "Generic Frame";
    const properties = {
      si: {
        name: "Test Frame",
        type: "generic-frame",
      },
      domain: {
        customProp1: "value1",
        customProp2: "value2",
      },
    };

    // Define the edit request
    const request = "Update the customProp1 value to 'new value'";

    // Call the editComponent function - this should not throw errors despite non-AWS kind
    const result = await editComponent(
      componentName,
      kind,
      properties,
      request,
    );
    console.log(JSON.stringify(result, null, 2));

    // Check that we have an update operation for the component
    assertEquals(result.status, "ok");
    assertEquals(!!result.ops.update[componentName], true);

    // The domain object should be present in the result
    const updatedProperties = result.ops.update[componentName].properties;
    assertEquals(!!updatedProperties.domain, true);
  });

  Deno.test("proposeEdits - suggests Name tag for EC2 instance", async () => {
    // Create a test component
    const kind = "AWS::EC2::Instance";
    const properties = {
      si: {
        name: "WordPress EC2 Instance",
        type: "aws-ec2-instance",
      },
      domain: {
        InstanceType: "t3.medium",
        KeyName: "my-key-pair",
        ImageId: "ami-0abcdef1234567890",
        SubnetId: "subnet-0123456789abcdef0",
        SecurityGroupIds: [
          "sg-0123456789abcdef0",
        ],
      },
    };

    // Define the edit request
    const request = "Add a tag with the key Name and value that matches the component name";

    // Call the proposeEdits function
    const suggestions = await proposeEdits(kind, properties, request);
    console.log(JSON.stringify(suggestions, null, 2));

    // Check that we have at least one suggestion
    assertEquals(Array.isArray(suggestions.properties), true);
    assertEquals(suggestions.properties.length > 0, true);

    // At least one of the suggestions should be for Tags
    const tagSuggestion = suggestions.properties.find((prop: any) => 
      prop.path.includes("Tags") || 
      (prop.value && prop.value.includes("Tags"))
    );
    assertEquals(!!tagSuggestion, true);
  });

  Deno.test("proposeEdits and editComponent integration", async () => {
    // Create a test component
    const componentName = "testInstance";
    const kind = "AWS::EC2::Instance";
    const properties = {
      si: {
        name: "Test EC2 Instance",
        type: "aws-ec2-instance",
      },
      domain: {
        InstanceType: "t3.micro",
      },
    };

    // Define the edit request
    const request = "Update the instance type to t3.medium";

    // First call proposeEdits
    const suggestions = await proposeEdits(kind, properties, request);
    
    // Then use those suggestions to call editComponent
    const result = await editComponent(
      componentName,
      kind,
      properties,
      request,
      suggestions
    );
    
    // Check that the instance type was updated
    assertEquals(result.status, "ok");
    assertEquals(!!result.ops.update[componentName], true);
    
    const updatedProperties = result.ops.update[componentName].properties;
    assertEquals(!!updatedProperties.domain, true);
    assertEquals(updatedProperties.domain && updatedProperties.domain.InstanceType, "t3.medium");
  });

  Deno.test("editComponent - handles deeply nested structures in ECS Task Definition", async () => {
    // Create a test ECS Task Definition component with nested ContainerDefinitions and LogConfiguration
    const componentName = "appTaskDefinition";
    const kind = "AWS::ECS::TaskDefinition";
    const properties = {
      si: {
        name: "Application Task Definition",
        type: "aws-ecs-taskdefinition",
      },
      domain: {
        Family: "app-task",
        ExecutionRoleArn: "arn:aws:iam::123456789012:role/ecsTaskExecutionRole",
        NetworkMode: "awsvpc",
        RequiresCompatibilities: ["FARGATE"],
        Cpu: "256",
        Memory: "512",
        ContainerDefinitions: [
          {
            Name: "app-container",
            Image: "123456789012.dkr.ecr.us-west-2.amazonaws.com/app:latest",
            Essential: true,
            PortMappings: [
              {
                ContainerPort: 80,
                HostPort: 80,
                Protocol: "tcp",
              },
            ],
          },
        ],
      },
    };

    // Define the edit request to add log configuration
    const request = 
      "Add CloudWatch logging to the first container definition. Use 'awslogs' as the LogDriver and set the awslogs-group to '/ecs/app-task' and awslogs-region to 'us-west-2'";

    // Call the editComponent function
    const result = await editComponent(
      componentName,
      kind,
      properties,
      request,
    );
    console.log(JSON.stringify(result, null, 2));

    // Check that we have an update operation for the component
    assertEquals(result.status, "ok");
    assertEquals(!!result.ops.update[componentName], true);

    // Check that the LogConfiguration was added correctly
    const updatedProperties = result.ops.update[componentName].properties;
    assertEquals(!!updatedProperties.domain, true);
    
    if (updatedProperties.domain && 
        Array.isArray(updatedProperties.domain.ContainerDefinitions) && 
        updatedProperties.domain.ContainerDefinitions.length > 0) {
      
      const container = updatedProperties.domain.ContainerDefinitions[0];
      assertEquals(!!container.LogConfiguration, true);
      assertEquals(container.LogConfiguration.LogDriver, "awslogs");
      assertEquals(!!container.LogConfiguration.Options, true);
      assertEquals(container.LogConfiguration.Options["awslogs-group"], "/ecs/app-task");
      assertEquals(container.LogConfiguration.Options["awslogs-region"], "us-west-2");
    } else {
      throw new Error("LogConfiguration was not properly added");
    }
  });

  Deno.test("proposeEdits - suggests LogConfiguration for ECS container", async () => {
    // Create a test ECS Task Definition
    const kind = "AWS::ECS::TaskDefinition";
    const properties = {
      si: {
        name: "Application Task Definition",
        type: "aws-ecs-taskdefinition",
      },
      domain: {
        Family: "app-task",
        ExecutionRoleArn: "arn:aws:iam::123456789012:role/ecsTaskExecutionRole",
        ContainerDefinitions: [
          {
            Name: "app-container",
            Image: "nginx:latest",
            Essential: true,
          },
        ],
      },
    };

    // Define the edit request
    const request = "Configure CloudWatch logs for the container with log group /ecs/app-logs";

    // Call the proposeEdits function
    const suggestions = await proposeEdits(kind, properties, request);
    console.log(JSON.stringify(suggestions, null, 2));

    // Check that we have at least one suggestion
    assertEquals(Array.isArray(suggestions.properties), true);
    assertEquals(suggestions.properties.length > 0, true);

    // At least one of the suggestions should be for LogConfiguration
    const logSuggestion = suggestions.properties.find((prop: any) => 
      prop.path.join('.').includes('LogConfiguration') || 
      (prop.value && prop.value.includes('LogConfiguration'))
    );
    assertEquals(!!logSuggestion, true);

    // Verify that the schema definition is included
    if (logSuggestion) {
      assertEquals(!!logSuggestion.schemaDefinition, true);
    }
  });
}
