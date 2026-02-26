#!/bin/sh
# Generate the plain-text password file from the env var
# We use printf to ensure NO trailing newlines
printf "%s" "$MONITOR_PW" > /etc/prometheus/metrics_pw.txt

# Substitute env vars in the prometheus config
envsubst < /etc/prometheus/prometheus.yml > /tmp/prometheus.yml \
  && mv /tmp/prometheus.yml /etc/prometheus/prometheus.yml

# Start Prometheus with the original flags
exec /bin/prometheus \
  --config.file=/etc/prometheus/prometheus.yml \
  --web.config.file=/etc/prometheus/web-config.yml