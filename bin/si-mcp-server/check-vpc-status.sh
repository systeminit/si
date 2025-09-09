#!/bin/bash

# VPC Status Monitor Script
# Monitors the status of the main-vpc created in System Initiative

VPC_NAME="main-vpc"
REGION="us-east-1"
CHECK_INTERVAL=10  # seconds between checks

echo "Monitoring VPC status for '$VPC_NAME' in region $REGION"
echo "Press Ctrl+C to stop"
echo "----------------------------------------"

while true; do
    # Get current timestamp
    TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
    
    # Find VPC by name tag
    VPC_INFO=$(aws ec2 describe-vpcs \
        --region "$REGION" \
        --filters "Name=tag:Name,Values=$VPC_NAME" \
        --query 'Vpcs[0].[VpcId,State,CidrBlock]' \
        --output text 2>/dev/null)
    
    if [ "$VPC_INFO" = "None	None	None" ] || [ -z "$VPC_INFO" ]; then
        echo "[$TIMESTAMP] VPC '$VPC_NAME' not found"
    else
        VPC_ID=$(echo "$VPC_INFO" | cut -f1)
        VPC_STATE=$(echo "$VPC_INFO" | cut -f2)
        VPC_CIDR=$(echo "$VPC_INFO" | cut -f3)
        
        echo "[$TIMESTAMP] VPC: $VPC_ID | State: $VPC_STATE | CIDR: $VPC_CIDR"
        
        # Show additional details if VPC exists
        if [ "$VPC_STATE" = "available" ]; then
            # Check for subnets
            SUBNET_COUNT=$(aws ec2 describe-subnets \
                --region "$REGION" \
                --filters "Name=vpc-id,Values=$VPC_ID" \
                --query 'length(Subnets)' \
                --output text 2>/dev/null)
            
            # Check for internet gateway
            IGW_INFO=$(aws ec2 describe-internet-gateways \
                --region "$REGION" \
                --filters "Name=attachment.vpc-id,Values=$VPC_ID" \
                --query 'InternetGateways[0].InternetGatewayId' \
                --output text 2>/dev/null)
            
            if [ "$IGW_INFO" != "None" ] && [ -n "$IGW_INFO" ]; then
                IGW_STATUS="attached ($IGW_INFO)"
            else
                IGW_STATUS="not attached"
            fi
            
            echo "         └── Subnets: $SUBNET_COUNT | Internet Gateway: $IGW_STATUS"
        fi
    fi
    
    sleep "$CHECK_INTERVAL"
done