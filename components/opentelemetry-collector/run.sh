#!/bin/bash
DIR="$(dirname "${BASH_SOURCE[0]}")"  # get the directory name
DIR="$(realpath "${DIR}")"    # resolve its full path if need be
cat ${DIR}/otel-local-config.yaml | sed "s/USERNAME/${USER}/g" > ${DIR}/otel-local-config-user.yaml
docker run -d -p 55678:55678 -p 55679:55679 -p 5778:5778 -p 14250:14250 -p 14268:14268 -p 6831:6831/udp -p 6832:6832/udp --name otelcol -v $(pwd)/otel-local-config-user.yaml:/etc/otel-local-config.yaml otel/opentelemetry-collector-contrib --config /etc/otel-local-config.yaml --log-level DEBUG 
