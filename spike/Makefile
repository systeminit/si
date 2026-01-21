MAKEPATH:=$(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))

pull: pull-deps pull-innit

pull-deps:
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

pull-innit:
	cd $(MAKEPATH) && docker compose pull innit

up:
	cd $(MAKEPATH) && docker compose up -d

down:
	cd $(MAKEPATH) && docker compose down

delete-volumes:
	-docker volume rm spike_postgres-data
