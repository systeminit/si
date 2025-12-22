---
outline:
  level: [2, 3, 4]
---

# How to Create a Qualification for a Schema

This how-to assumes:

- The [AI agent](../reference/ai-agent) initialized to your workspace
- Basic familiarity with System Initiative

It will teach you how to create a [qualification](../reference/qualification)
that validates that all [AWS S3 Buckets](https://aws.amazon.com/s3/) have
encryption enabled.

## Create the Qualificiation

The easiest way to author a qualification in System Initiative is via our AI
Agent.

```prompt [Create a best practice VPC]
Create me qualification on an AWS S3 Bucket that ensures that the bucket is encrypted
```

```prompt
⏺ I'll help you create a qualification function for AWS S3 Buckets to ensure encryption is enabled. Let me break this down into steps.

⏺ Perfect! I've successfully created a qualification function for AWS S3 Buckets that ensures encryption is enabled.

  What Was Created

  Change Set: "Add S3 Bucket Encryption Qualification"
  Function: "Bucket Encryption Enabled"

  How It Works

  The qualification function checks:

  1. BucketEncryption exists - Verifies the bucket has encryption configuration
  2. ServerSideEncryptionConfiguration is populated - Ensures at least one encryption rule is defined
  3. SSEAlgorithm is specified - Confirms an encryption algorithm (AES256 or aws:kms) is set

  Qualification Results

  - Success: Bucket has proper server-side encryption configured
  - Failure: Returns specific error messages for:
    - Missing encryption configuration
    - Empty encryption rules
    - No SSE algorithm specified

  Next Steps

  To apply this qualification to your AWS S3 Buckets:

  1. Review the change set: https://app.systeminit.com/n/01KD37BN83B3E4750C1EYR55A8/01KD37F12F5J85MG8GXCTNDBN9/h?map=1
  2. Check for any qualification failures on the schema
  3. Apply the change set to make the qualification active

  Once applied, any S3 Bucket component that doesn't have encryption enabled will show a qualification failure, helping you maintain security compliance across your infrastructure.
```

## Review the qualification code

It's always a good thing to review the code that the AI Agent has generated and
ask the agent to ensure that it fits the original brief.

```prompt
Show me the code for the function you created and dies this code fit the original brief to ensure that S3 buckets have encryption enabled?
```

```prompt
⏺ Here's the qualification function code I created for checking S3 bucket encryption:

  async function main(component) {
    if (!component.domain) {
      return {
        result: "failure",
        message: "Component domain is not defined"
      };
    }

    const bucketEncryption = component.domain.BucketEncryption;

    if (!bucketEncryption) {
      return {
        result: "failure",
        message: "S3 bucket does not have encryption enabled. Configure BucketEncryption to secure your data at rest."
      };
    }

    const serverSideEncryptionConfig = bucketEncryption.ServerSideEncryptionConfiguration;

    if (!serverSideEncryptionConfig || !Array.isArray(serverSideEncryptionConfig) || serverSideEncryptionConfig.length === 0) {
      return {
        result: "failure",
        message: "S3 bucket encryption configuration is empty. Add ServerSideEncryptionConfiguration with at least one encryption rule."
      };
    }

    let hasValidEncryption = false;
    for (let i = 0; i < serverSideEncryptionConfig.length; i++) {
      const config = serverSideEncryptionConfig[i];
      if (config.ServerSideEncryptionByDefault && config.ServerSideEncryptionByDefault.SSEAlgorithm) {
        hasValidEncryption = true;
        break;
      }
    }

    if (!hasValidEncryption) {
      return {
        result: "failure",
        message: "S3 bucket encryption configuration does not specify an SSE algorithm. Set SSEAlgorithm to either AES256 or aws:kms."
      };
    }

    return {
      result: "success",
      message: "S3 bucket has server-side encryption properly configured."
    };
  }

  The function performs validation in layers:
  1. Checks if the domain exists on the component
  2. Verifies BucketEncryption is configured
  3. Ensures ServerSideEncryptionConfiguration array exists and has entries
  4. Validates at least one config has an SSEAlgorithm set
  5. Returns success only when encryption is properly configured

⏺ Looking at the code objectively, it mostly meets the brief but has a potential gap:

  What It Does Well ✓

  - Checks if BucketEncryption configuration exists
  - Verifies ServerSideEncryptionConfiguration array is present and not empty
  - Confirms at least one encryption rule has an SSEAlgorithm specified

  Potential Issue ⚠️

  The code checks if SSEAlgorithm exists, but doesn't validate it has a valid value. For example:
  - An empty string ("") would pass
  - An invalid algorithm name would pass
  - Only valid values should be "AES256" or "aws:kms" (or "aws:kms:dsse" for dual-layer encryption)

  Would you like me to update the qualification function with this improved validation?
```

If you accept the changes, you are ready to apply the change set.

## Apply your Change Set

```prompt [Apply the Change Set]
> Apply the change set.
```

:::info

You may be asked by the AI agent for permission to apply the change set here. By
default, System Initiative configures agents to confirm major operations like
apply.

:::

```prompt
⏺ Perfect! The change set has been successfully applied.

  What Happened

  The "Bucket Encryption Enabled" qualification function is now active on the AWS::S3::Bucket schema in HEAD.

  What This Means

  From now on:
  - All AWS S3 Bucket components will be validated by this qualification
  - Buckets without encryption will show a qualification failure
  - Buckets with invalid encryption algorithms will also fail
  - Only buckets with valid encryption (AES256, aws:kms, or aws:kms:dsse) will pass

  Next Steps

  The qualification will automatically run on:
  - Any existing S3 Bucket components in HEAD
  - New S3 Bucket components you create
  - S3 Buckets modified in future change sets

  You can now create or check S3 Bucket components to see the qualification in action. Any bucket without proper encryption will immediately show a failure with a clear message about what needs to be fixed.
```

## Testing your qualification

```prompt
Open a change set and create an S3 bucket that has no encryption enabled and give me any qualification status
```

```prompt
⏺ Perfect! The qualification is working exactly as expected. Here's what I created:

  Change Set Created

  Name: "Test S3 Bucket Encryption Qualification"

  S3 Bucket Component

  Name: test-unencrypted-bucket
  Bucket Name: test-unencrypted-bucket-demo
  Encryption: None (intentionally omitted)

  Qualification Status: ❌ FAILURE

  Function: Bucket Encryption Enabled
  Result: failure
  Message: "S3 bucket does not have encryption enabled. Configure BucketEncryption to secure your data at rest."

  What This Demonstrates

  The qualification function is actively protecting your infrastructure by:
  1. Detecting the missing encryption configuration
  2. Blocking the bucket from being deployed without encryption
  3. Providing clear guidance on what needs to be fixed
```

Now ask the agent to update the component to ensure that the qualification
passes:

```prompt
update the component to set encryption using the default AWS encryption
```

```prompt
⏺ Excellent! The bucket has been updated with encryption and the qualification now passes.

  What Changed

  Encryption Configuration Added:
  /domain/BucketEncryption/ServerSideEncryptionConfiguration/0/ServerSideEncryptionByDefault/SSEAlgorithm: "AES256"

  This configures the bucket to use AES256 encryption (AWS managed keys), which is the default AWS encryption method.

  Qualification Status: ✅ SUCCESS

  Function: Bucket Encryption Enabled
  Result: success
  Message: "S3 bucket has server-side encryption properly configured."

  Before vs After

  Before:
  - ❌ No encryption configured
  - ❌ Qualification failed
  - ❌ Bucket blocked from deployment

  After:
  - ✅ AES256 encryption enabled
  - ✅ Qualification passed
  - ✅ Bucket ready to deploy

  Current Component Attributes

  - Bucket Name: test-unencrypted-bucket-demo
  - Encryption: AES256 (AWS managed server-side encryption)
  - Status: Ready to create

  The qualification is working perfectly - it caught the security issue and now confirms the bucket is properly secured with encryption!
```

The qualification is working as expected!

## Clean Up Testing Resources

```prompt [Clean up test bucket]
Abandon the change set that was used for testing the bucket qualification
```

:::info

You will be asked by the AI agent for permission to abandon the change set here.
By default, System Initiative configures agents to confirm major operations like
abandon.

:::

```prompt
⏺ Done! The change set "Test S3 Bucket Encryption Qualification" has been abandoned.

  What Happened

  - ✅ Change set abandoned successfully
  - 🗑️ Test S3 bucket component discarded
  - 🔒 Qualification function remains active in HEAD

  Current State

  The "Bucket Encryption Enabled" qualification is still active and will continue to validate all S3 buckets going forward. The test change set and bucket component have been cleaned up and won't affect your workspace.

  Your qualification is ready to protect any future S3 bucket components from being deployed without proper encryption!
```
