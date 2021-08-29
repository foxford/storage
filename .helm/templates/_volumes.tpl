{{- define "volumes" }}
- name: config
  configMap:
    name: {{ .Chart.Name }}-config
- name: internal
  secret:
    secretName: secrets-internal
{{- range $tenant := .Values.app.tenants }}
- name: {{ $tenant.name | lower }}
  secret:
    secretName: secrets-{{ $tenant.name | lower }}
{{- end }}
{{- end }}

{{- define "volumeMounts" }}
- name: config
  mountPath: /app/App.toml
  subPath: App.toml
- name: internal
  mountPath: {{ printf "/app/%s" (pluck .Values.werf.env .Values.app.authz.key | first | default .Values.app.authz.key._default) }}
  subPath: private_key
- name: internal
  mountPath: {{ printf "/app/%s" (pluck .Values.werf.env .Values.app.authn.key | first | default .Values.app.authn.key._default) }}
  subPath: public_key
{{- range $tenant := .Values.app.tenants }}
- name: {{ $tenant.name | lower }}
  mountPath: {{ printf "/app/%s" (pluck $.Values.werf.env $tenant.authn.key | first | default $tenant.authn.key._default) }}
  subPath: public_key
{{- end }}
{{- end }}
