FROM quay.io/keycloak/keycloak:latest as builder

# Disable health and metrics support
ENV KC_HEALTH_ENABLED=false
ENV KC_METRICS_ENABLED=false

# Configure the database vendor
ENV KC_DB=postgres


# Configure Keycloak env vars
ENV KEYCLOAK_ADMIN=kcadmin
ENV KEYCLOAK_ADMIN_PASSWORD=kcadmin
ENV KC_DB_USERNAME=postgres
ENV KC_DB_PASSWORD=changeme
ENV KC_DB_URL_HOST=sso_db

# Default commands
CMD ["start-dev"]

ENTRYPOINT ["/opt/keycloak/bin/kc.sh"]
