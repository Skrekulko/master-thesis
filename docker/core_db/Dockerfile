FROM postgres:latest

ENV POSTGRES_DB postgres
ENV POSTGRES_USER postgres
ENV POSTGRES_PASSWORD changeme

COPY init.sql /docker-entrypoint-initdb.d/
