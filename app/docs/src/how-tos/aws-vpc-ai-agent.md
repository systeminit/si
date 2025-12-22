---
outline:
  level: [2, 3, 4]
---

# How to Create and Manage an AWS VPC with the AI Agent

This how-to assumes:

- A [System Initiative workspace with AWS Credentials](../tutorials/setup)
- The [AI agent](../reference/ai-agent) initialized to your workspace
- Basic familiarity with System Initiative
- Basic familiarity with [AWS VPC](https://docs.aws.amazon.com/vpc/)

It will teach you how to create a best practice AWS VPC and manage it with
System Initiative.

## Create the VPC

```prompt [Create a best practice VPC]
Create a best practice VPC with public and private subnets in 3 availability zones.
```

::: tip

When using Claude Code for prompts with many steps, Claude will make a TODO to track progress. Use ctrl+t to see a nice view of it!

```prompt
⏺ I'll help you create a best practice VPC with public and private subnets across 3 availability zones. Let me plan and execute this task.

* Creating Internet Gateway and attaching to VPC… (esc to interrupt · 1m 5s · ↓ 3.7k tokens)
  ⎿  ☒ Check for existing change sets and create a new one if needed
     ☒ Verify AWS credentials and region components exist
     ☒ Create VPC with CIDR block
     ☐ Create Internet Gateway and attach to VPC
     ☐ Create 3 public subnets (one per AZ)
     ☐ Create 3 private subnets (one per AZ)
     ☐ Create 3 Elastic IPs for NAT Gateways
     ☐ Create 3 NAT Gateways (one per public subnet)
     ☐ Create public route table with internet gateway route
     ☐ Associate public subnets with public route table
     ☐ Create 3 private route tables with NAT gateway routes
     ☐ Associate private subnets with their respective route tables
     ☐ Verify all components and check for qualification failures
```

:::

```prompt
⏺ I'll help you create a best practice VPC with public and private subnets across 3 availability zones. Let me plan and execute this task.

⏺ Great! I found an existing AWS Credential and a Region (us-east-1) in the change set. Now I'll create the VPC infrastructure.

⏺ Perfect! I've successfully created a best practice VPC with public and private subnets across 3 availability zones in the change set "best-practice-vpc-setup". Here's what was created:

  Infrastructure Summary

  VPC Configuration:
  - VPC CIDR: 10.0.0.0/16
  - DNS hostnames and DNS support enabled
  - Region: us-east-1

  Public Subnets (Internet-accessible):
  - public-subnet-1a: 10.0.0.0/20 in us-east-1a (4,096 IPs)
  - public-subnet-1b: 10.0.16.0/20 in us-east-1b (4,096 IPs)
  - public-subnet-1c: 10.0.32.0/20 in us-east-1c (4,096 IPs)
  - All with auto-assign public IP enabled

  Private Subnets (NAT Gateway access):
  - private-subnet-1a: 10.0.128.0/20 in us-east-1a (4,096 IPs)
  - private-subnet-1b: 10.0.144.0/20 in us-east-1b (4,096 IPs)
  - private-subnet-1c: 10.0.160.0/20 in us-east-1c (4,096 IPs)

  High Availability Components:
  - 3 NAT Gateways (one per AZ for HA)
  - 3 Elastic IPs (one per NAT Gateway)
  - Internet Gateway for public internet access

  Routing:
  - 1 public route table (shared across all public subnets) → Internet Gateway
  - 3 private route tables (one per AZ) → respective NAT Gateway

  Best Practices Implemented

  ✓ Multi-AZ deployment for high availability
  ✓ Separate public and private subnets
  ✓ NAT Gateway per AZ (no single point of failure)
  ✓ Proper CIDR block allocation with room for expansion
  ✓ DNS resolution enabled for service discovery

  All components have been validated with no qualification failures. You can view your infrastructure here:

  View in System Initiative: https://app.systeminit.com/n/01KCME9G1NG1MPRCV0GZV9B8HB/01KCT6JH8WC8WHBB7PD5EZ5G3V/h?map=1

  When you're ready to deploy this infrastructure to AWS, apply the change set and the resources will be created in your AWS account.
```

## Review the proposed VPC

Review the new VPC your AI Agent has modeled by clicking on the link it provided. You should expect 31 network infrastructure components, including your AWS Credential, a Region, VPC, 6 subnets and NAT gateways. You can use the map view for an overview of the relationships between them. Click on individual components such as VPCs and subnets, to review their properties.

The AI Agent itself can help with the review as well, by answering powerful questions about the
proposed VPC.

```prompt
Now perform a review of the infrastructure paying attention to cost and security.
```

```prompt
⏺ I'll perform a comprehensive review of the VPC infrastructure focusing on cost optimization and security best practices.

⏺ Infrastructure Review: Cost & Security Analysis

  💰 COST ANALYSIS

  High-Cost Components

  ❗ CRITICAL: NAT Gateway Costs (Highest Impact)
  - Current Setup: 3 NAT Gateways (one per AZ)
  - Monthly Cost: ~$97.20/month per NAT Gateway × 3 = ~$291.60/month
  - Data Processing: $0.045 per GB processed × 3 gateways
  - Annual Impact: ~$3,500/year just for NAT Gateways

  Cost Optimization Options:

  1. Single NAT Gateway (Cost-Optimized)
    - Reduce to 1 NAT Gateway in a single AZ
    - Savings: ~$194.40/month (~$2,332/year)
    - Trade-off: Loss of high availability - if that AZ fails, private subnets lose internet access
    - Best for: Development/testing environments, non-critical workloads
  2. Dual NAT Gateway (Balanced)
    - Use 2 NAT Gateways across 2 AZs
    - Savings: ~$97.20/month (~$1,166/year)
    - Trade-off: Reduced redundancy but still better than single
    - Best for: Production workloads with moderate availability requirements
  3. NAT Instance Alternative
    - Replace NAT Gateways with EC2-based NAT instances
    - Savings: Potentially 60-70% depending on instance size (t3.micro ~$7/month)
    - Trade-offs: Manual management, lower bandwidth, maintenance overhead
    - Best for: Very cost-sensitive, low-bandwidth scenarios
  4. VPC Endpoints for AWS Services
    - Add Gateway Endpoints (S3, DynamoDB) - FREE
    - Add Interface Endpoints for other services (~$7.20/month each)
    - Reduces data transfer through NAT Gateway
    - Best for: Heavy AWS service usage from private subnets

  Other Cost Considerations

  Free Resources (✓ No ongoing costs):
  - VPC, Subnets, Route Tables
  - Internet Gateway
  - Subnet/Route Table Associations
  - Elastic IPs (while attached to NAT Gateways)

  Potential Hidden Costs:
  - Data transfer OUT from NAT Gateway: $0.045/GB (can add up quickly)
  - Cross-AZ data transfer: $0.01/GB between availability zones
  - VPC Flow Logs (if enabled): CloudWatch Logs storage costs

  🔒 SECURITY ANALYSIS

  Critical Security Gaps

  1. ❌ No Network ACLs (NACLs) Configured
  - Currently using default NACL (allows all traffic)
  - Risk: No subnet-level firewall protection
  - Impact: Cannot implement defense-in-depth at network layer
  - Recommendation: Create custom NACLs with explicit allow/deny rules

  2. ❌ No Security Groups Defined
  - No instance-level firewall rules configured
  - Risk: When EC2 instances are launched, they'll need proper security groups
  - Recommendation: Pre-create baseline security groups:
    - Public-facing (web tier): Allow 80/443 from internet
    - Application tier: Allow specific ports from public tier only
    - Database tier: Allow database ports from app tier only
    - Management/Bastion: Restricted SSH/RDP access

  3. ❌ No VPC Flow Logs Enabled
  - No network traffic monitoring or forensics capability
  - Risk: Cannot detect unusual traffic patterns, investigate security incidents, or troubleshoot connectivity
  - Compliance: Many frameworks (PCI-DSS, SOC 2) require network logging
  - Recommendation: Enable VPC Flow Logs to CloudWatch or S3

  4. ❌ No Private Connectivity (VPN/Direct Connect)
  - No secure connection from corporate network to private subnets
  - Risk: Must use public internet + bastion hosts for management
  - Recommendation: Consider AWS VPN or Direct Connect for sensitive workloads

  5. ❌ Missing Network Segmentation
  - No additional subnet tiers (database, management, etc.)
  - Risk: All private resources in same subnet tier
  - Recommendation: Consider dedicated subnets for:
    - Database tier (more restrictive security groups)
    - Management/bastion hosts (tightly controlled access)

  Security Strengths (✓)

  1. ✓ Multi-AZ Architecture
  - Provides resilience against AZ-level failures
  - Good for availability and disaster recovery

  2. ✓ Public/Private Subnet Separation
  - Proper network tier segregation
  - Private subnets not directly exposed to internet

  3. ✓ NAT Gateway per AZ
  - No single point of failure for outbound connectivity
  - Each private subnet has independent internet path

  4. ✓ DNS Resolution Enabled
  - Supports internal service discovery
  - Required for VPC endpoints and AWS services

  Additional Security Considerations

  Missing Components:
  - VPC Endpoint for S3/DynamoDB: Free gateway endpoints to avoid NAT Gateway for AWS API calls
  - AWS Network Firewall: Stateful firewall inspection (if required for compliance)
  - Route 53 Resolver DNS Firewall: DNS-based threat protection
  - GuardDuty VPC Flow Log Analysis: Automated threat detection
  - Resource Tagging Strategy: For cost allocation and governance
  - AWS Config Rules: Automated compliance checking

  📊 RISK MATRIX

  | Issue              | Cost Impact           | Security Impact | Priority |
  |--------------------|-----------------------|-----------------|----------|
  | 3 NAT Gateways     | 🔴 High (~$3,500/yr)  | 🟢 Low          | HIGH     |
  | No NACLs           | 🟢 None               | 🟡 Medium       | MEDIUM   |
  | No Security Groups | 🟢 None               | 🔴 High         | HIGH     |
  | No VPC Flow Logs   | 🟡 Low (~$50/mo)      | 🟡 Medium       | MEDIUM   |
  | No VPC Endpoints   | 🟡 Medium (data xfer) | 🟢 Low          | MEDIUM   |

  🎯 RECOMMENDATIONS BY PRIORITY

  Immediate Actions (Before Deployment):

  1. Decide on NAT Gateway strategy based on environment:
    - Production: Keep 3 (current setup)
    - Staging: Reduce to 2
    - Development: Reduce to 1
  2. Create baseline Security Groups before launching any instances
  3. Enable VPC Flow Logs for security monitoring

  Short-term Improvements (Week 1-2):

  4. Implement Network ACLs for defense-in-depth
  5. Add VPC Endpoints for S3 and DynamoDB (free, reduces NAT costs)
  6. Implement tagging strategy for all resources

  Long-term Enhancements (Month 1-3):

  7. Set up AWS Config rules for compliance monitoring
  8. Evaluate AWS Network Firewall if compliance requires inspection
  9. Consider Transit Gateway if connecting multiple VPCs

  Would you like me to help implement any of these recommendations? I can:
  - Reduce NAT Gateways to optimize costs
  - Create Security Groups and Network ACLs
  - Add VPC Flow Logs
  - Configure VPC Endpoints for S3/DynamoDB
```

Another great question to ask is "Can you explain the choices you made to me?"

:::tip

Now is also a good time to iterate and make changes before the infrastructure is created!

```prompt
Reduce down to 2 NAT gateways for cost savings.
```

:::

## Apply your Change Set

```prompt [Apply the Change Set]
> Apply the change set.
```

:::info

You may be asked by the AI agent for permission to apply the change set here. By default, System Initiative
configures agents to confirm major operations like apply.

:::

```prompt
⏺ Perfect! The change set has been successfully applied and AWS resource creation is now in progress.

  Deployment Status

  Currently Executing:
  - ✓ VPC (main-vpc) - Dispatched
  - ✓ Internet Gateway (main-igw) - Dispatched
  - ✓ Elastic IPs (eip-nat-1a, eip-nat-1b, eip-nat-1c) - Dispatched

  Queued (waiting for dependencies):
  - 6 Subnets (3 public, 3 private)
  - 4 Route Tables (1 public, 3 private)
  - 3 NAT Gateways
  - 9 Associations and Routes

  The resources are being created in the correct dependency order:
  1. VPC and foundational resources (creating now)
  2. Subnets (after VPC)
  3. NAT Gateways (after subnets + EIPs)
  4. Route tables and routes
  5. Route table associations

  Let me check for any action failures after a moment.

...

⏺ Great progress! The deployment is proceeding smoothly with no failures.

  Current Status Update

  ✓ Successfully Created & Refreshing:
  - VPC (main-vpc)
  - Internet Gateway (main-igw)
  - All 3 Elastic IPs
  - IGW Attachment
  - All 6 Subnets (3 public + 3 private) - currently refreshing

  ⚙️ Currently Creating:
  - Route Tables (public and private) - in progress

  ⏳ Queued (waiting for dependencies):
  - 3 NAT Gateways (these typically take 2-5 minutes to create)
  - Routes and route table associations

  No Failures Detected - All actions are progressing normally.

  The NAT Gateways are the longest-running resources to create. Once they're complete, the remaining routes and associations will be configured quickly.

  Would you like me to:
  1. Continue monitoring until all resources are complete?
  2. Check for any specific resource details?
  3. Proceed with the security improvements we discussed (Security Groups, NACLs, VPC Flow Logs)?

  The infrastructure is deploying successfully! Note that complete deployment may take 5-10 minutes due to NAT Gateway creation time.
```

On the right hand side of the System Initiative UI, you will now see actions running, creating the VPC, subnets, gateways and routes in AWS.

## Explore your resources

You can review the completed AWS resources by clicking into your Components and viewing the `Resource` sub-panel for each of your new resources.

## Clean Up

```prompt [Clean up VPC]
Delete all VPC components and apply the change set.
```

```prompt
⏺ I've successfully completed the deletion process. All 29 VPC-related components have been deleted and the
   change set has been applied. The deletion actions are now being executed in AWS.

  I can see that 15 VPC components are still showing in HEAD, which is expected - they're marked for
  deletion and the deletion actions are currently running or queued. These include:
  - main-vpc
  - main-igw
  - igw-attachment
  - 3 public subnets (1a, 1b, 1c)
  - 1 private subnet (1b)
  - 3 Elastic IPs (nat-1a, nat-1b, nat-1c)
  - 3 NAT Gateways (1a, 1b, 1c)

  The remaining components (route tables, routes, and route table associations) have already been removed,
  likely as dependencies.

  The AWS Credential and us-east-1 Region components remain intact as expected.

⏺ You can monitor the deletion progress in the System Initiative web interface:
  https://app.systeminit.com/n/01KCME9G1NG1MPRCV0GZV9B8HB/01KCME9K73PV8NNXC5CBC1Z4V8/h

  The deletion actions will execute in the proper order based on AWS dependencies. NAT Gateways will be
  deleted first, followed by subnets, then the Internet Gateway attachment, Internet Gateway, and finally
  the VPC itself.
```

All your new resources should be deleted from your AWS account.
