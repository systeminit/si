#!/bin/bash

set -euo pipefail

HEALTH_ENDPOINT="${HEALTH_ENDPOINT:-http://127.0.0.1:8080/health}"
SLEEP_INTERVAL="${SLEEP_INTERVAL:-30}"
AWS_REGION="${AWS_REGION:-us-east-1}"

check_health_and_report() {
    local service_name="$1"
    local instance_id="$2"
    local value
    
    if curl -f -s --max-time 10 "$HEALTH_ENDPOINT" > /dev/null; then
        value=1
        echo "$(date -Iseconds): Health check PASSED for $service_name (instance: $instance_id)"
    else
        value=0
        echo "$(date -Iseconds): Health check FAILED for $service_name (instance: $instance_id)"
    fi
    
    aws cloudwatch put-metric-data \
        --region "$AWS_REGION" \
        --namespace "SI/ServiceHealth" \
        --metric-data "MetricName=Health,Value=$value,Unit=Count,Dimensions=[{Name=Service,Value=$service_name},{Name=Instance,Value=$instance_id}]"
    
    if [ $? -eq 0 ]; then
        echo "$(date -Iseconds): Metric sent successfully (Value: $value)"
    else
        echo "$(date -Iseconds): Failed to send metric to CloudWatch"
    fi
}

main() {
    local service_name="${SI_SERVICE:-unknown}"
    local instance_id="${SI_INSTANCE_ID:-unknown}"
    
    echo "$(date -Iseconds): Starting health monitor for service: $service_name"
    echo "$(date -Iseconds): Instance ID: $instance_id"
    echo "$(date -Iseconds): Health endpoint: $HEALTH_ENDPOINT"
    echo "$(date -Iseconds): Check interval: ${SLEEP_INTERVAL}s"
    echo "$(date -Iseconds): AWS Region: $AWS_REGION"
    
    while true; do
        check_health_and_report "$service_name" "$instance_id"
        sleep "$SLEEP_INTERVAL"
    done
}

main "$@"