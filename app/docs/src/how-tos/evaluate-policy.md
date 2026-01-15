---
outline:
  level: [2, 3, 4]
---

# How to Evaluate Policy

This how-to assumes:

- Basic [familiarity with System Initiative](../tutorials/setup)
- The [SI CLI installed and authenticated](../reference/si-cli)
- The [AI agent](../reference/ai-agent) initialized to your workspace
- An `ANTHROPIC_API_KEY` environment variable set with your [Claude API key](https://platform.claude.com/settings/keys)
- At least one AWS VPC created
  - You can follow the [AWS VPC via AI Agent](./aws-vpc-ai-agent) "how-to" to create AWS VPCs quickly
  - You can also use whatever infrastructure you'd like, but you will have to modify the example policy documents accordingly

It will teach you how to write and evaluate policy against your System Initiative infrastructure using the `si policy evaluate` command.

:::info

Policy evaluation is the third layer of [policy enforcement in System Initiative](../explanation/policy-layers). Unlike native cloud provider policies (preventive) and component qualifications (detective), control document evaluation provides compliance verification through AI-powered assessment of your infrastructure against written compliance requirements.

:::

## Understanding Policy Evaluation

The `si policy evaluate` command uses the [Claude Agent SDK](https://platform.claude.com/docs/en/agent-sdk/overview) to:

1. Extract policy structure from markdown documents
2. Collect infrastructure data from System Initiative
3. Evaluate components and resources against policy requirements
4. Generate detailed markdown reports with findings
5. Upload results to System Initiative (with the ability to opt-out)

This approach allows you to write policies in natural language and get comprehensive compliance reports without writing custom code.

## Create a Policy Document

Let's create a compliance policy that ensures all AWS VPCs use private IP address ranges. Create a file called `vpc-private-ip-policy.md` with the following content:

```markdown
# All VPCs must use private IP ranges

## Policy

All VPCs must be configured with private IP address ranges according to RFC 1918:
- 10.0.0.0/8
- 172.16.0.0/12
- 192.168.0.0/16

Public IP address ranges in VPC CIDR blocks pose security risks and can lead to routing conflicts with internet traffic.

### Exceptions

Test VPCs may use non-standard ranges for development purposes when explicitly documented and approved.

## Source Data

### System Initiative

~~~yaml
all-vpcs: "schema:\"AWS::EC2::VPC\""
~~~

## Output Tags

~~~yaml
tags:
  - networking
  - security
  - compliance
~~~
```

The policy document has three main sections:

- **Policy**: Describes the compliance requirement in natural language
- **Source Data**: YAML queries to collect infrastructure data from System Initiative
- **Output Tags**: Categorization tags for the policy report

## Evaluate the Policy

Run the policy evaluation command:

```bash
si policy evaluate vpc-private-ip-policy.md --name "VPC Private IP Compliance"
```

You should see output similar to:

```shell
✨ info    si              Starting policy evaluation for: "vpc-private-ip-policy.md"
✨ info    si              Stage 1: Extracting policy structure...
I'll parse the compliance policy document and extract the structured information according to the specifications.

Let me analyze the document:

1. **Policy Title**: "All VPCs must use private IP ranges" (from the # heading)
2. **Policy Text**: The full policy description including the exceptions section
3. **Source Data Queries**: From the YAML block under "System Initiative"
   - `all-vpcs: "schema:\"AWS::EC2::VPC\""`
4. **Output Tags**: From the YAML tags array
   - `["networking", "security", "compliance"]`

Now I'll create the JSON output and write it to the specified file:
Perfect! I've successfully parsed the compliance policy document and extracted the structured information. The JSON file ha

**`/Users/toddhoward/scripts/ai-agent/prod/2026-01-15T19:07:52Z/vpc-private-ip-policy-extracted.json`**

The extracted information includes:
- **Policy Title**: "All VPCs must use private IP ranges"
- **Policy Text**: Full policy description with RFC 1918 IP ranges and the exceptions for test VPCs
- **Source Data Queries**: Single query named "all-vpcs" targeting AWS EC2 VPC resources
- **Output Tags**: Three tags - networking, security, and compliance
✨ info    si              Policy extraction complete
✨ info    si              Policy extracted: "All VPCs must use private IP ranges"
✨ info    si              Stage 2: Collecting source data...
✨ info    si              Collecting data for: "all-vpcs"
✨ info    si              Source data collection complete
✨ info    si              Stage 3: Evaluating policy compliance...
✨ info    si              Evaluating 3 component(s)
I'll evaluate the infrastructure against the compliance policy. Let me start by reading the source data file.
Now I'll analyze the VPCs against the compliance policy. Let me evaluate each component:

**Policy Requirements:**
- VPCs must use RFC 1918 private IP address ranges:
  - 10.0.0.0/8
  - 172.16.0.0/12
  - 192.168.0.0/16
- Exception: Test VPCs may use non-standard ranges when explicitly documented and approved

**Components Found:**
1. **VPC-1**: CidrBlock = "10.1.0.0/16" ✓ (within 10.0.0.0/8)
2. **VPC-2**: CidrBlock = "10.2.0.0/16" ✓ (within 10.0.0.0/8)
3. **VPC-3**: CidrBlock = "10.3.0.0/16" ✓ (within 10.0.0.0/8)

All VPCs are using IP ranges within the 10.0.0.0/8 RFC 1918 private address space, so all components pass the policy.

Now I'll write the evaluation results to the output file:
## Evaluation Complete ✓

I've successfully evaluated the infrastructure against the VPC private IP address policy. Here are the findings:

**Result: Pass**

All 3 VPCs in the infrastructure comply with the RFC 1918 private IP address range requirements:

- **VPC-1**: 10.1.0.0/16 ✓
- **VPC-2**: 10.2.0.0/16 ✓
- **VPC-3**: 10.3.0.0/16 ✓

All VPCs are using CIDR blocks within the approved 10.0.0.0/8 range, which is one of the three RFC 1918 private address ranges specified in the policy. No components are using public IP address ranges that would pose security risks or routing conflicts.

The evaluation results have been written to:
`/Users/toddhoward/scripts/ai-agent/prod/2026-01-15T19:07:52Z/vpc-private-ip-policy-evaluation.json`
✨ info    si              Policy evaluation complete
✨ info    si              Result: "Pass"
✨ info    si              Failing components: 0
✨ info    si              Stage 4: Generating report...
✨ info    si              Report generation complete
✨ info    si              Output: "/Users/toddhoward/scripts/ai-agent/prod/2026-01-15T19:07:52Z/report.md"
✨ info    si              Files organized in: "/Users/toddhoward/scripts/ai-agent/prod/2026-01-15T19:07:52Z"
✨ info    si              Stage 5: Uploading policy evaluation results...
✨ info    si              Policy uploaded: "01KF1GYFXVQQJDXGNSZQR7ZRFK"
✨ info    si              Policy Evaluation Complete
✨ info    si              Result: PASS
✨ info    si              Report: "/Users/toddhoward/scripts/ai-agent/prod/2026-01-15T19:07:52Z/report.md"
```

## Review the Evaluation Report

The command creates a timestamped folder containing several files:

- `vpc-private-ip-compliance-extracted.json` - Extracted policy structure
- `vpc-private-ip-compliance-source-data.json` - Infrastructure data from System Initiative
- `vpc-private-ip-compliance-evaluation.json` - Evaluation results
- `report.md` - Human-readable markdown report

Open the `report.md` file to see detailed findings:

```markdown
# All VPCs must use private IP ranges

**Date**: 2026-01-15T19:08:34Z

## Policy

All VPCs must be configured with private IP address ranges according to RFC 1918:
- 10.0.0.0/8
- 172.16.0.0/12
- 192.168.0.0/16

Public IP address ranges in VPC CIDR blocks pose security risks and can lead to routing conflicts with internet traffic.

### Exceptions

Test VPCs may use non-standard ranges for development purposes when explicitly documented and approved.

## Tags

- networking
- security
- compliance

## Test Results

**Result**: Pass

All 3 VPCs in the infrastructure comply with RFC 1918 private IP address range requirements. All VPCs use CIDR blocks within the 10.0.0.0/8 range (VPC-1: 10.1.0.0/16, VPC-2: 10.2.0.0/16, VPC-3: 10.3.0.0/16), which is one of the approved private IP address ranges according to RFC 1918.

## Source Data

### System Initiative

#### all-vpcs

| Component | CIDR Block | VPC Name | AWS VPC ID | Resource Status |
|--|--|--|--|--|
|AWS::EC2::VPC VPC-1|10.1.0.0/16|VPC-1|vpc-062d28d3423842758|ok|
|AWS::EC2::VPC VPC-2|10.2.0.0/16|VPC-2|vpc-0fc5bba7331d49524|ok|
|AWS::EC2::VPC VPC-3|10.3.0.0/16|VPC-3|vpc-0061661ac6d27617d|ok|

**Column Selection Reasoning**: The CIDR Block is the primary attribute for evaluating RFC 1918 compliance, as the policy specifically requires private IP address ranges (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16). The VPC Name and AWS VPC ID are essential for identifying non-compliant resources. Resource Status confirms the VPC is actively deployed and subject to the policy.
```

## Evaluate Multiple Policies

You can organize multiple policy documents in a directory and evaluate them all at once.
Let's start by moving our existing policy into a new directory.

```bash
mkdir compliance-policies
mv vpc-private-ip-policy.md compliance-policies/
```

Now, create a new policy called `vpc-cidr.md`.
This policy will check all AWS VPC components' CIDR block configuration.

```markdown
# All VPCs in the 10 CIDR block

## Policy

All the VPCs must be in the 10 CIDR block, and not have overlapping addresses.

### Exceptions

The default VPC (sandbox) is allowed to be the 172.31 CIDR block.

## Source Data

### System Initiative

~~~yaml
all-vpc: "schema:\"AWS::EC2::VPC\""
~~~

## Output Tags

~~~yaml
tags:
  - networking
~~~
```

Evaluate all policies in the directory:

```bash
si policy evaluate compliance-policies/ --all
```

When using `--all`:
- Policy names are derived from filenames (e.g., `vpc-private-ip-policy.md` → `vpc-private-ip-policy`)
- Each policy is evaluated independently
- If one policy fails, others continue processing
- A summary shows results for all evaluated policies

You should see output similar to:

```shell
✨ info    si              Found 2 policy file(s) to evaluate
✨ info    si              === Evaluating policy: "vpc-private-ip-policy" ===
✨ info    si              Starting policy evaluation for: "compliance-policies//vpc-private-ip-policy.md"
✨ info    si              Stage 1: Extracting policy structure...
I'll parse the compliance policy document and extract the structured information as requested.

Let me analyze the document:

1. **Policy Title**: "All VPCs must use private IP ranges"
2. **Policy Text**: Includes the main policy description and the exceptions section
3. **Source Data Queries**: Contains one query `all-vpcs` with value `schema:"AWS::EC2::VPC"`
4. **Output Tags**: Contains three tags: networking, security, compliance

Now I'll write the extracted JSON to the specified file:
Perfect! I've successfully parsed the compliance policy document and extracted all the structured information into a JSON f

**Summary of extracted data:**
- **Policy Title**: "All VPCs must use private IP ranges"
- **Policy Text**: Complete description including RFC 1918 private IP ranges and the exception for test VPCs
- **Source Data Queries**: One query named "all-vpcs" that searches for AWS EC2 VPC schemas
- **Output Tags**: Three tags (networking, security, compliance)

The JSON has been written to: `/Users/toddhoward/scripts/ai-agent/prod/2026-01-15T19:12:40Z/vpc-private-ip-policy-extracted.json`
✨ info    si              Policy extraction complete
✨ info    si              Policy extracted: "All VPCs must use private IP ranges"
✨ info    si              Stage 2: Collecting source data...
✨ info    si              Collecting data for: "all-vpcs"
✨ info    si              Source data collection complete
✨ info    si              Stage 3: Evaluating policy compliance...
✨ info    si              Evaluating 3 component(s)
I'll evaluate the infrastructure against the compliance policy. Let me start by reading the source data file.
Now I'll analyze the VPC components against the compliance policy. Let me evaluate each VPC's CIDR block:

**Policy Requirements:**
- VPCs must use RFC 1918 private IP ranges:
  - 10.0.0.0/8
  - 172.16.0.0/12
  - 192.168.0.0/16

**Components Found:**
1. **VPC-1**: CIDR Block `10.1.0.0/16` - ✅ Within 10.0.0.0/8 range (RFC 1918 compliant)
2. **VPC-2**: CIDR Block `10.2.0.0/16` - ✅ Within 10.0.0.0/8 range (RFC 1918 compliant)
3. **VPC-3**: CIDR Block `10.3.0.0/16` - ✅ Within 10.0.0.0/8 range (RFC 1918 compliant)

All VPCs are using private IP address ranges according to RFC 1918. Now I'll write the evaluation results:
## Evaluation Complete ✅

I've successfully evaluated the infrastructure against the VPC private IP address compliance policy.

**Results Summary:**
- **Overall Result**: **Pass**
- **VPCs Evaluated**: 3
- **Failing Components**: 0

**Details:**
All three VPCs in the infrastructure are compliant with RFC 1918 private IP address requirements:

1. **VPC-1**: Uses `10.1.0.0/16` (within 10.0.0.0/8 range) ✅
2. **VPC-2**: Uses `10.2.0.0/16` (within 10.0.0.0/8 range) ✅
3. **VPC-3**: Uses `10.3.0.0/16` (within 10.0.0.0/8 range) ✅

All CIDR blocks fall within the approved RFC 1918 private IP address space, meaning there are no security risks from public

The evaluation results have been written to:
`/Users/toddhoward/scripts/ai-agent/prod/2026-01-15T19:12:40Z/vpc-private-ip-policy-evaluation.json`
✨ info    si              Policy evaluation complete
✨ info    si              Result: "Pass"
✨ info    si              Failing components: 0
✨ info    si              Stage 4: Generating report...
✨ info    si              Report generation complete
✨ info    si              Output: "/Users/toddhoward/scripts/ai-agent/prod/2026-01-15T19:12:40Z/report.md"
✨ info    si              Files organized in: "/Users/toddhoward/scripts/ai-agent/prod/2026-01-15T19:12:40Z"
✨ info    si              Stage 5: Uploading policy evaluation results...
✨ info    si              Policy uploaded: "01KF1H7ACVCME2F5MMEJE9P0P5"
✨ info    si              Policy Evaluation Complete
✨ info    si              Result: PASS
✨ info    si              Report: "/Users/toddhoward/scripts/ai-agent/prod/2026-01-15T19:12:40Z/report.md"
✨ info    si              === Evaluating policy: "vpc-cidr" ===
✨ info    si              Starting policy evaluation for: "compliance-policies//vpc-cidr.md"
✨ info    si              Stage 1: Extracting policy structure...
I'll parse the policy document and extract the structured information, then write it to the specified file.

Let me analyze the document:

1. **Policy Title**: "All VPCs in the 10 CIDR block"
2. **Policy Text**: The full policy description including the exception
3. **Source Data Queries**: The YAML query under System Initiative
4. **Output Tags**: The tags array

Now I'll create the JSON output and write it to the file:
Perfect! I've successfully parsed the compliance policy document and extracted the structured information. The JSON file ha

**Summary of extracted information:**

- **Policy Title**: "All VPCs in the 10 CIDR block"
- **Policy Text**: Includes the main policy requirement about VPCs being in the 10 CIDR block without overlapping addresses
- **Source Data Queries**: One query named "all-vpc" with the value `schema:"AWS::EC2::VPC"`
- **Output Tags**: One tag: "networking"
✨ info    si              Policy extraction complete
✨ info    si              Policy extracted: "All VPCs in the 10 CIDR block"
✨ info    si              Stage 2: Collecting source data...
✨ info    si              Collecting data for: "all-vpc"
✨ info    si              Source data collection complete
✨ info    si              Stage 3: Evaluating policy compliance...
✨ info    si              Evaluating 3 component(s)
I'll evaluate the infrastructure components against the compliance policy. Let me start by reading the source data file.
Now let me analyze the VPC components against the compliance policy:

**Policy Requirements:**
1. All VPCs must be in the 10.x.x.x CIDR block
2. VPCs must not have overlapping addresses
3. Exception: The default VPC (sandbox) is allowed to be in the 172.31.x.x CIDR block

**Analysis of Components:**

1. **VPC-2** (01KF1ESVB7G07WCK2KX8SW5TZE): CIDR = 10.2.0.0/16 ✓ Pass
2. **VPC-3** (01KF1EYXSK35JN00ANG2TPWA0G): CIDR = 10.3.0.0/16 ✓ Pass
3. **VPC-1** (01KF1EKPPNNPS22W856JAZ59HT): CIDR = 10.1.0.0/16 ✓ Pass

All three VPCs are in the 10.x.x.x CIDR block as required. They use different /16 subnets (10.1.0.0/16, 10.2.0.0/16, 10.3.0.0/16), so there are no overlapping addresses. None of them are named "sandbox" or appear to be default VPCs.

**Result: Pass** - All components comply with the policy.

Now let me write the evaluation results to the JSON file:
## Evaluation Complete ✓

I've successfully evaluated the infrastructure against the compliance policy and written the results to the JSON file.

**Summary:**
- **Result:** Pass
- **Components Evaluated:** 3 VPCs
- **Failing Components:** 0

**Details:**
All three VPCs comply with the policy requirements:
- **VPC-1**: 10.1.0.0/16 ✓
- **VPC-2**: 10.2.0.0/16 ✓
- **VPC-3**: 10.3.0.0/16 ✓

Each VPC is in the required 10.x.x.x CIDR block, and they all use non-overlapping /16 subnets. No VPCs were found that needed the sandbox exception (172.31.x.x CIDR block).

The evaluation results have been written to: `/Users/toddhoward/scripts/ai-agent/prod/2026-01-15T19:13:24Z/vpc-cidr-evaluation.json`
✨ info    si              Policy evaluation complete
✨ info    si              Result: "Pass"
✨ info    si              Failing components: 0
✨ info    si              Stage 4: Generating report...
✨ info    si              Report generation complete
✨ info    si              Output: "/Users/toddhoward/scripts/ai-agent/prod/2026-01-15T19:13:24Z/report.md"
✨ info    si              Files organized in: "/Users/toddhoward/scripts/ai-agent/prod/2026-01-15T19:13:24Z"
✨ info    si              Stage 5: Uploading policy evaluation results...
✨ info    si              Policy uploaded: "01KF1H8GASX7MS857HH8SB06C1"
✨ info    si              Policy Evaluation Complete
✨ info    si              Result: PASS
✨ info    si              Report: "/Users/toddhoward/scripts/ai-agent/prod/2026-01-15T19:13:24Z/report.md"
✨ info    si              === Completed evaluating all policies ===
```

There will be a report generated for each policy evaluated that you can review, just like before.

:::info

We are working on the ability to fetch reports so that you do not have to review reports with local files.

:::

Congratulations! You now have two policies that you can evaluate against AWS VPCs in System Initiative.

## Next Steps

Now that you understand policy evaluation:

1. Create policies for your organization's compliance requirements
2. Set up automated policy evaluation in CI/CD pipelines
3. Use policy reports to demonstrate compliance during audits
4. Combine with [component qualifications](../reference/qualification) and [native cloud policies](./aws-scp-policy) for comprehensive governance across [policy layers](../explanation/policy-layers)
