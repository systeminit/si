MAKEPATH:=$(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))

all: up

pull:
	cd $(MAKEPATH) && docker compose pull \
		grafana \
		jaeger \
		loki \
		nats \
		otelcol \
		postgres \
		prometheus \
		promtail \
		versitygw

up:
	cd $(MAKEPATH) && docker compose up -d

down:
	cd $(MAKEPATH) && docker compose down

delete-volumes:
	-docker volume rm spike_postgres-data
