#!/bin/bash

set -uo pipefail

HEALTH_ENDPOINT="${HEALTH_ENDPOINT:-http://127.0.0.1:8080/health}"
SLEEP_INTERVAL="${SLEEP_INTERVAL:-30}"
AWS_REGION="${AWS_REGION:-us-east-1}"
STARTUP_GRACE_PERIOD="${STARTUP_GRACE_PERIOD:-300}"
UNHEALTHY_THRESHOLD="${UNHEALTHY_THRESHOLD:-10}"

# Global state variables
CONSECUTIVE_FAILURES=0
STARTUP_TIME=$(date +%s)
GRACE_PERIOD_ACTIVE=true
LAST_REPORTED_STATUS=""

check_health_and_report() {
    local service_name="$1"
    local instance_id="$2"
    local current_time=$(date +%s)
    local health_check_passed=false
    local should_report=false
    local report_status=""
    
    echo "$(date -Iseconds): Starting health check function..."
    
    # Perform health check
    echo "$(date -Iseconds): Attempting health check to $HEALTH_ENDPOINT"
    local curl_output
    if curl_output=$(curl -f -s --max-time 10 "$HEALTH_ENDPOINT" 2>&1); then
        health_check_passed=true
        CONSECUTIVE_FAILURES=0
        echo "$(date -Iseconds): Health check PASSED for $service_name (instance: $instance_id)"
    else
        health_check_passed=false
        CONSECUTIVE_FAILURES=$((CONSECUTIVE_FAILURES + 1))
        echo "$(date -Iseconds): Health check FAILED for $service_name (instance: $instance_id) - Failure count: $CONSECUTIVE_FAILURES"
        echo "$(date -Iseconds): Curl error: $curl_output"
    fi
    
    # Check if we're still in startup grace period
    if [ "$GRACE_PERIOD_ACTIVE" = true ]; then
        local elapsed=$((current_time - STARTUP_TIME))
        if [ $elapsed -ge $STARTUP_GRACE_PERIOD ]; then
            GRACE_PERIOD_ACTIVE=false
            echo "$(date -Iseconds): Startup grace period of ${STARTUP_GRACE_PERIOD}s has ended"
        fi
    fi
    
    # Determine if we should report and what status
    if [ "$health_check_passed" = true ]; then
        # Single success = healthy (always report healthy immediately)
        should_report=true
        report_status="Healthy"
        if [ "$GRACE_PERIOD_ACTIVE" = true ]; then
            echo "$(date -Iseconds): Early healthy signal detected during grace period"
            GRACE_PERIOD_ACTIVE=false
        fi
    else
        # Health check failed
        if [ "$GRACE_PERIOD_ACTIVE" = true ]; then
            # Still in grace period - don't report unhealthy
            should_report=false
            echo "$(date -Iseconds): Still in startup grace period - not reporting unhealthy status"
        elif [ $CONSECUTIVE_FAILURES -ge $UNHEALTHY_THRESHOLD ]; then
            # Grace period over and threshold reached
            should_report=true
            report_status="Unhealthy"
            echo "$(date -Iseconds): Unhealthy threshold of $UNHEALTHY_THRESHOLD consecutive failures reached"
        else
            # Grace period over but threshold not reached yet
            should_report=false
            echo "$(date -Iseconds): Grace period over but only $CONSECUTIVE_FAILURES/$UNHEALTHY_THRESHOLD failures - not reporting unhealthy yet"
        fi
    fi
    
    # Always send CloudWatch metric for every health check
    local metric_value
    if [ "$health_check_passed" = true ]; then
        metric_value=1
    else
        metric_value=0
    fi
    
    aws cloudwatch put-metric-data \
        --region "$AWS_REGION" \
        --namespace "SI/InstanceLifecycle" \
        --metric-data "MetricName=HealthStatus,Value=$metric_value,Unit=None,Dimensions=[{Name=Service,Value=$service_name},{Name=Instance,Value=$instance_id}]"
    
    echo "$(date -Iseconds): CloudWatch health status metric sent (Value: $metric_value)"
    
    # Report to ASG when we should report (healthy always, unhealthy only after threshold)
    if [ "$should_report" = true ]; then
        aws autoscaling set-instance-health \
            --region "$AWS_REGION" \
            --instance-id "$instance_id" \
            --health-status "$report_status"
        
        if [ $? -eq 0 ]; then
            echo "$(date -Iseconds): ASG health status set to $report_status"
            LAST_REPORTED_STATUS="$report_status"
            
            # Send additional metric when marking node as unhealthy (killed)
            if [ "$report_status" = "Unhealthy" ]; then
                aws cloudwatch put-metric-data \
                    --region "$AWS_REGION" \
                    --namespace "SI/InstanceLifecycle" \
                    --metric-data "MetricName=HealthStatus,Value=3,Unit=None,Dimensions=[{Name=Service,Value=$service_name},{Name=Instance,Value=$instance_id}]"
                
                echo "$(date -Iseconds): CloudWatch node killed metric sent (Value: 3)"
            fi
        else
            echo "$(date -Iseconds): Failed to set ASG health status to $report_status"
        fi
    elif [ "$should_report" = true ]; then
        echo "$(date -Iseconds): Health status unchanged ($report_status) - not reporting to ASG"
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
    echo "$(date -Iseconds): Startup grace period: ${STARTUP_GRACE_PERIOD}s"
    echo "$(date -Iseconds): Unhealthy threshold: $UNHEALTHY_THRESHOLD consecutive failures"
    
    # Send "started" metric on service startup
    echo "$(date -Iseconds): Sending startup metric to CloudWatch..."
    if aws cloudwatch put-metric-data \
        --region "$AWS_REGION" \
        --namespace "SI/InstanceLifecycle" \
        --metric-data "MetricName=HealthStatus,Value=2,Unit=None,Dimensions=[{Name=Service,Value=$service_name},{Name=Instance,Value=$instance_id}]" 2>&1; then
        echo "$(date -Iseconds): CloudWatch startup metric sent successfully (Value: 2 - started)"
    else
        echo "$(date -Iseconds): Failed to send CloudWatch startup metric - continuing anyway"
    fi
    
    echo "$(date -Iseconds): Starting health check loop..."
    while true; do
        check_health_and_report "$service_name" "$instance_id"
        sleep "$SLEEP_INTERVAL"
    done
}

main "$@"