FROM dpage/pgadmin4:latest

ENV PGADMIN_DEFAULT_EMAIL pgadmin4@pgadmin.org
ENV PGADMIN_DEFAULT_PASSWORD changeme
ENV PGADMIN_CONFIG_SERVER_MODE False
ENV PGADMIN_CONFIG_MASTER_PASSWORD_REQUIRED False

WORKDIR /pgadmin4

COPY ./servers.json .
COPY ./pgpass .

USER root
RUN chown pgadmin:root /pgadmin4/pgpass
