{{- define "probes" }}
readinessProbe:
  tcpSocket:
    port: app
  initialDelaySeconds: 5
  periodSeconds: 10
livenessProbe:
  tcpSocket:
    port: app
  initialDelaySeconds: 15
  periodSeconds: 20
{{- end }}
