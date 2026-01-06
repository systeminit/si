---
outline:
  level: [2, 3, 4]
---

# How to Create an AWS Service Control Policy for Cost Control

This how-to assumes:

- Basic [familiarity with System Initiative](../tutorials/setup)
- The [AI agent](../reference/ai-agent) initialized to your workspace
- AWS Credentials and Region configured in System Initiative
- Familiarity with
  [AWS Service Control Policies (SCPs)](https://docs.aws.amazon.com/organizations/latest/userguide/orgs_manage_policies_scps.html)

It will teach you how to create and test an AWS Service Control Policy that
prevents the launch of expensive EC2 instance types.

:::info

[Service Control Policies (SCPs)](https://docs.aws.amazon.com/organizations/latest/userguide/orgs_manage_policies_scps.html) are a type of organization policy in AWS
Organizations that allow you to manage permissions across your organization.
Unlike IAM policies that grant permissions, SCPs set the maximum available
permissions and can be used to prevent certain actions across all accounts in
your organization.

:::

## Start with the Policy

We'll create an SCP that denies the launch of expensive instance types,
specifically:

- All `p` family instances (GPU compute optimized)
- All `g` family instances (Graphics intensive)
- All `m7i` instances larger than 2xlarge
- All `c7gn` instances (Network optimized)

Here's the policy we'll implement:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "DenyLargeFamilies",
      "Effect": "Deny",
      "Action": "ec2:RunInstances",
      "Resource": "*",
      "Condition": {
        "ForAnyValue:StringLike": {
          "ec2:InstanceType": ["p*.*", "g*.*", "m7i.*2xlarge", "c7gn.*"]
        }
      }
    }
  ]
}
```

## Verify the Policy with the AI Agent

Copy the policy into your clipboard. Let's use the System Initiative AI Agent to
verify our policy is correctly formatted and understand what it does.

```prompt
Verify that the below policy is a valid AWS Service Control Policy. Explain what it does and whether there are any security or cost optimization considerations.

{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "DenyLargeFamilies",
      "Effect": "Deny",
      "Action": "ec2:RunInstances",
      "Resource": "*",
      "Condition": {
        "ForAnyValue:StringLike": {
          "ec2:InstanceType": [
            "p*.*",
            "g*.*",
            "m7i.*2xlarge",
            "c7gn.*"
          ]
        }
      }
    }
  ]
}
```

```prompt
⏺ This is a valid AWS Service Control Policy (SCP) with correct JSON syntax and structure.

  What This Policy Does

  This SCP denies the launch of expensive EC2 instance types across all accounts in the organization or OU where it's applied:

  - P-family instances (p*.*) - GPU instances for machine learning/HPC (e.g., p4d, p5)
  - G-family instances (g*.*) - GPU instances for graphics/ML workloads (e.g., g4dn, g5)
  - M7i large instances (m7i.*2xlarge) - General-purpose instances 2xlarge and larger
  - C7gn instances (c7gn.*) - Network-optimized compute instances

  Cost Optimization Considerations

  ✅ Effective Cost Controls:
  - P and G instance families can cost $10-40+ per hour (P5 instances cost ~$98/hour)
  - C7gn instances are premium networking instances (~$2-10/hour)
  - Prevents accidental launches that could result in massive bills
  - Good guardrail for development/test environments

  ⚠️ Potential Issues:
  1. Pattern ambiguity: m7i.*2xlarge will match:
    - ✅ m7i.2xlarge
    - ✅ m7i.12xlarge, m7i.16xlarge, m7i.24xlarge, m7i.48xlarge (likely intended)
    - But the wildcard * before 2xlarge means it needs at least one character, so it works correctly
  2. May block legitimate workloads:
    - ML/AI teams may need P or G instances
    - High-throughput applications may require C7gn                                                                          - Consider exception mechanisms (separate OU or account exemptions)                                                                                                                                                                             Security Considerations                                                                                                                                                                                                                           - Not security-focused: This is cost control, not a security measure
  - Doesn't affect running instances: Only prevents new launches
  - Could impact incident response: If you need burst compute capacity during an incident
  - Consider allowlisting specific accounts/OUs that legitimately need these instance types
  Recommendations

  1. ✅ Policy is well-structured for cost control
  2. Consider adding exception paths for ML/data science teams
  3. Monitor CloudTrail for denied RunInstances calls to identify legitimate use cases being blocked
  4. Document which teams/accounts are exempt and why
```

:::tip

The AI Agent can help you refine the policy further. Try asking:

```prompt
Should I add any other expensive instance types to this policy?
```

:::

## Create the SCP in System Initiative

Now that we've verified our policy, let's have the agent create it in System
Initiative.

```prompt
Create an AWS Service Control Policy component named "DenyExpensiveInstanceTypes" with that policy. The description should be "Prevents launch of expensive GPU and compute-optimized instance types". Use the existing AWS Credential and Region components.
```

```prompt
⏺ I'll help you create an AWS Service Control Policy component with that policy. Let me start by creating a change set and finding the necessary components.

⏺ Perfect! I found the AWS::Organizations::Policy schema and the existing AWS Credential and Region components. Let me check the schema attributes to understand what fields are required.

⏺ system-initiative - schema-attributes-list (MCP)(schemaNameOrId: "AWS::Organizations::Policy", changeSetId:
                                                  "01KE89DMMYNFBTQM1VD6G4M721")
  ⎿  {
       "status": "success",
       "data": {
     … +60 lines (ctrl+o to expand)

⏺ Now let me check the documentation for the Type and Content fields to ensure I use the correct values for a Service Control Policy.

⏺ Excellent! I've successfully created the AWS Service Control Policy component. Let me verify the configuration:

⏺ ✅ Successfully created the AWS Service Control Policy component!

  Component Details
  Name: DenyExpensiveInstanceTypes
  Schema: AWS::Organizations::Policy
  Type: SERVICE_CONTROL_POLICY
  Description: Prevents launch of expensive GPU and compute-optimized instance types

  Configuration Summary

  The SCP has been configured with:
  - ✅ Policy document blocking: p*.*, g*.*, m7i.*2xlarge, c7gn.* instances
  - ✅ AWS Credential subscription to existing credential component
  - ✅ Region subscription to existing region component
  - ✅ No qualification failures - component is ready to deploy

  Available Actions

  - Create Asset - Deploy the SCP to AWS Organizations
  - Update Asset - Modify the policy after creation
  - Refresh Asset - Sync state from AWS
  - Delete Asset - Remove the SCP
  - Import from AWS - Import an existing policy
  - Discover on AWS - Find existing policies

  Next Steps

  1. Review the configuration in the System Initiative web UI
  2. Apply the change set when ready: This will create the policy in AWS Organizations
  3. Attach to OUs/Accounts: After creation, attach this SCP to specific organizational units or accounts in AWS Organizations Console

  ⚠️ Important: This SCP will only take effect after you apply the change set AND attach it to target accounts/OUs in AWS Organizations.

  Would you like me to apply the change set now, or would you prefer to review it first?
```

In this case, notice that the agent has prompted that it's not attached to any
accounts or OUs in the AWS Organization. You can tell the agent to attach it to
the OU / account for you:

```prompt
can you attach this policy to this OU `ou-ts96-8zgxljlr`?
```

```prompt
⏺ I'll attach the SCP to the organizational unit by adding it to the TargetIds. Let me update the component:

⏺ Perfect! I've successfully attached the SCP to the organizational unit ou-ts96-8zgxljlr.

  Updated Configuration

  The DenyExpensiveInstanceTypes SCP now has:
  - Target OU: ou-ts96-8zgxljlr
  - Qualifications: ✓ All passing

  When you apply this change set, the policy will be:
  1. Created in AWS Organizations
  2. Automatically attached to the specified OU (ou-ts96-8zgxljlr)

  All accounts under this OU (and any nested OUs) will have the cost control restrictions enforced, preventing them from launching expensive P-family, G-family, large M7i, and C7gn instances.

  Ready to apply the change set when you are!
```

## Examine the New Component and Apply the Change Set

Before we apply the change set, let's look at the new component to ensure it
looks as we expect.

```prompt
Show me the created policy component.
```

```prompt
⏺ Here's the SCP policy component "DenyExpensiveInstanceTypes":
  Component Details
  - Name: DenyExpensiveInstanceTypes
  - Schema: AWS::Organizations::Policy
  - Type: SERVICE_CONTROL_POLICY
  - Description: Prevents launch of expensive GPU and compute-optimized instance types
  - Component ID: 01KE89ECYGPTP6X779W4C1PRF5
  - Change Set: add-expensive-instance-scp

  Policy Content
  {
    "Version": "2012-10-17",
    "Statement": [
    {
        "Sid": "DenyLargeFamilies",
        "Effect": "Deny",
        "Action": "ec2:RunInstances",
        "Resource": "*",
        "Condition": {
          "ForAnyValue:StringLike": {
            "ec2:InstanceType": [
              "p*.*",
              "g*.*",
              "m7i.*2xlarge",
              "c7gn.*"
            ]
          }
        }
      }
    ]
  }

  Subscriptions
    - Region: Subscribes to AWS | Region component (/domain/region)
    - AWS Credential: Subscribes to AWS | Credential component (/secrets/AWS Credential)

  Available Actions

  - Create Asset
  - Update Asset
  - Refresh Asset
  - Delete Asset

  Status

  ✅ No qualification failures - the policy is properly configured

  This policy prevents launching expensive EC2 instance types including GPU instances (p-family, g-family), large m7i instances (2xlarge+), and c7gn network-optimized instances.
```

:::tip

This is a valid SCP configuration, but feel free to ask the agent to make any
changes you would like!

:::

You can now apply the change set.

```prompt
Apply the change set.
```

:::info

You may be asked by the AI agent for permission to apply the change set here. By
default, System Initiative configures agents to confirm major operations like
apply.

:::

```prompt
⏺ I'll apply the change set now.

⏺ The change set has been applied! The "Create" action for the SCP is currently Queued. Let me check the action status to see if it has executed:

⏺ ✅ Change set applied successfully!

  The Service Control Policy has been created and attached to AWS Organizations:

  Results

  - Status: ✅ Success
  - Policy ID: p-1vyzwaio
  - Attached to OU: ou-ts96-8zgxljlr
  - Policy Name: DenyExpensiveInstanceTypes
  - Type: SERVICE_CONTROL_POLICY

  What happened:

  The SCP was created in AWS Organizations (us-east-1 region) using AWS CloudControl API. The policy is now active and enforcing cost controls on the specified organizational unit.

  Impact:

  All AWS accounts under OU ou-ts96-8zgxljlr (including any nested OUs) are now prevented from launching:
  - P-family GPU instances (p*.*)
  - G-family GPU instances (g*.*)
  - Large M7i instances (m7i.*2xlarge)
  - All C7gn network-optimized instances (c7gn.*)

  Any attempt to launch these instance types will be denied with an AWS Organizations SCP denial message, helping protect your organization from unexpected cloud costs.
```

## Test the SCP with Attemping to Create EC2 Instances

As the SCP policy has been applied to a specific OU / AWS Account, we need to
test the policy in account it was attached to. Let's verify that our SCP works
by attempting to create EC2 instances with blocked instance types. The agent
will help us create test instances and observe the SCP denials. I have already
created a credential and a region for that specific account. That credential is
called `si-apps-sandbox`.

```prompt
Create an EC2 instance with instance type "g4dn.xlarge" to test that the SCP will block its creation in the real world. Ensure the instance is attached to `si-apps-sandbox` credential and `si-apps-sandbox-region` components. You can use ami-0b6c6ebed2801a5cb which is the AMI for Ubuntu 22.04 LTS.
```

```prompt
⏺ I'll create an EC2 instance with the g4dn.xlarge instance type to test the SCP blocking. Let me set up a todo list and get started:

✶ Creating a change set for testing the SCP… (esc to interrupt · ctrl+t to hide todos · thought for 4s)
  ⎿  ☐ Create a change set for testing the SCP
     ☐ Find the EC2 Instance schema
     ☐ Find si-apps-sandbox credential and region components
     ☐ Create the g4dn.xlarge EC2 instance
     ☐ Apply the change set to test SCP blocking

 EC2 Instance: test-g4dn-instance

 Instance Characteristics:
 - Instance Type: g4dn.xlarge
   - GPU instance family (NVIDIA T4 Tensor Core GPU)
   - 4 vCPUs, 16 GiB RAM
   - 1 GPU with 16 GiB GPU memory
   - Matches the SCP deny pattern: g*.*
 - AMI: ami-0b6c6ebed2801a5cb (Ubuntu 22.04 LTS)
 - Credentials: Connected to si-apps-sandbox AWS Credential
 - Region: Connected to si-apps-sandbox-region

 Qualifications: ✓ All passing
 - Instance type is valid for the region

 Why this tests the SCP:
 The g4dn.xlarge instance type matches the g*.* pattern in the DenyExpensiveInstanceTypes SCP. When we apply this change set, AWS should reject the instance creation with an SCP denial, proving that the policy is working correctly to block expensive GPU instances in the target OU.

 Available Actions:
 - Create Asset (this will be blocked by SCP)
 - Update Asset
 - Refresh Asset
 - Delete Asset
 - Reboot/Stop/Start EC2 Instance

 Ready to apply when you are!
```

Apply the change set and observe the failure.

```prompt
Apply the change set. We should see the EC2 instance create action fail due to the SCP.
```

```prompt
⏺ I'll apply the change set now. The EC2 instance creation should fail due to the SCP blocking the g4dn.xlarge instance type:

⏺ Change set applied! Let me check the action status to see if the SCP blocked the instance creation:

⏺ ✅ SUCCESS! The SCP is working correctly!

  The EC2 instance creation was blocked by the Service Control Policy as expected!

  Test Results

  Error Message from AWS:
  "You are not authorized to perform this operation. User: arn:aws:sts::104640795146:assumed-role/AWSReservedSSO_AdministratorAccess_d661ebf4da6c4f09/paul@systeminit.com is not authorized to perform: ec2:RunInstances on resource: arn:aws:ec2:us-east-1:104640795146:instance/* with an explicit deny in a service control policy."

  Key Details:
  - Status Code: 403 (Access Denied)
  - Error Code: AccessDenied
  - Root Cause: Explicit deny in a Service Control Policy
  - Instance Type Attempted: g4dn.xlarge (matches SCP pattern g*.*)
  - User: AdministratorAccess role (even admins are blocked by SCPs!)

  What This Proves

  1. ✅ The DenyExpensiveInstanceTypes SCP is active and enforcing
  2. ✅ The policy successfully blocks G-family GPU instances (g4dn.xlarge)
  3. ✅ The OU attachment (ou-ts96-8zgxljlr) is working correctly
  4. ✅ Even users with AdministratorAccess cannot bypass SCPs
  5. ✅ Cost control is in place - no expensive GPU instances can be launched

  The SCP is functioning exactly as designed, preventing expensive instance types from being launched in the target organizational unit. This demonstrates effective cost governance at the AWS Organizations level!
```

Congratulations! Now, you have an AWS SCP that can deny the creation of
expensive EC2 instances.

## Clean Up

Remember to clean up any components!

```prompt
Delete all Instance and SCP components and apply the change set.
```

:::info

You may be asked by the AI agent for permission to delete a component. By
default, System Initiative configures agents to confirm major operations like
apply.

:::

```prompt
⏺ I'll delete the EC2 instance and SCP components. Let me create a change set for this cleanup:

✻ Deleting EC2 instance component… (esc to interrupt · ctrl+t to hide todos · thinking)
  ⎿  ☒ Create change set for cleanup
     ☐ Delete EC2 instance component
     ☐ Delete SCP component
     ☐ Apply the change set

⏺ Cleanup Results

  ✅ EC2 Instance Deleted

  The test-g4dn-instance component was successfully removed (it never actually created a resource in AWS due to the SCP blocking it).

  ⚠️ SCP Deletion Failed

  The DenyExpensiveInstanceTypes SCP (Policy ID: p-1vyzwaio) cannot be deleted because:

  Error: "The specified policy is in use and cannot be deleted."

  Reason: The SCP is still attached to organizational unit ou-ts96-8zgxljlr. AWS Organizations requires you to detach policies from all targets before deletion.

  To Complete Deletion:

  You have two options:

  1. Detach then Delete (recommended if you want to fully remove the policy):
    - First, update the SCP component to remove the TargetIds (detach from the OU)
    - Apply that change set
    - Then delete the SCP component
  2. Leave the SCP in place:
    - The SCP is still active and protecting against expensive instance launches
    - You can detach it manually later via the AWS Console if needed

  Would you like me to detach the SCP from the OU first, then delete it?
```

If you agree, then it will cleanup as expected. 
