#!/bin/bash

set -euo pipefail

HEALTH_ENDPOINT="${HEALTH_ENDPOINT:-http://127.0.0.1:8080/health}"
SLEEP_INTERVAL="${SLEEP_INTERVAL:-30}"
AWS_REGION="${AWS_REGION:-us-east-1}"

check_health_and_report() {
    local service_name="$1"
    local instance_id="$2"
    local health_status
    
    if curl -f -s --max-time 10 "$HEALTH_ENDPOINT" > /dev/null; then
        health_status="Healthy"
        echo "$(date -Iseconds): Health check PASSED for $service_name (instance: $instance_id)"
    else
        health_status="Unhealthy"
        echo "$(date -Iseconds): Health check FAILED for $service_name (instance: $instance_id)"
    fi
    
    # Report to Auto Scaling Group for scaling decisions
    if [ "$health_status" = "Unhealthy" ]; then
        aws autoscaling set-instance-health \
            --region "$AWS_REGION" \
            --instance-id "$instance_id" \
            --health-status "$health_status" \
            --should-respect-grace-period false
    else
        aws autoscaling set-instance-health \
            --region "$AWS_REGION" \
            --instance-id "$instance_id" \
            --health-status "$health_status"
    fi
    
    if [ $? -eq 0 ]; then
        echo "$(date -Iseconds): ASG health status set to $health_status"
    else
        echo "$(date -Iseconds): Failed to set ASG health status"
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