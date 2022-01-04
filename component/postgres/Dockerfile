ARG BASE_VERSION
FROM postgres:${BASE_VERSION}

COPY component/postgres/multiple-database-support.sh /docker-entrypoint-initdb.d/
