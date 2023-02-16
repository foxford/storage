{{/*
Expand the name of the chart.
*/}}
{{- define "storage.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "storage.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "storage.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}} # TODO
{{- define "storage.labels" -}}
helm.sh/chart: {{ include "storage.chart" . }}
app.kubernetes.io/version: {{ .Values.app.image.tag | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{ include "storage.selectorLabels" . }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "storage.selectorLabels" -}}
app.kubernetes.io/name: {{ include "storage.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Tenant Service Audience
*/}}
{{- define "storage.tenantServiceAudience" -}}
{{- $tenant := . -}}
{{- list "svc" $tenant | join "." -}}
{{- end -}}

{{/*
Tenant User Audience
*/}}
{{- define "storage.tenantUserAudience" -}}
{{- $tenant := . -}}
{{- list "usr" $tenant | join "." -}}
{{- end -}}

{{/*
Tenant Object Audience
*/}}
{{- define "storage.tenantObjectAudience" -}}
{{- $namespace := index . 0 -}}
{{- $tenant := index . 1 -}}
{{- $env := regexSplit "-" $namespace -1 | first -}}
{{- $devEnv := ""}}
{{- if ne $env "p" }}
{{- $devEnv = regexReplaceAll "(s)(\\d\\d)" $env "staging${2}" }}
{{- $devEnv = regexReplaceAll "(t)(\\d\\d)" $devEnv "testing${2}" }}
{{- end }}
{{- list $devEnv $tenant | compact | join "." }}
{{- end }}

{{/*
Namespace in ingress path.
converts as follows:
- testing01 -> t01
- staging01-storage-ng -> s01/storage-ng
- production-storage-ng -> storage-ng
*/}}
{{- define "storage.ingressPathNamespace" -}}
{{- $ns_head := regexSplit "-" .Release.Namespace -1 | first }}
{{- $ns_tail := regexSplit "-" .Release.Namespace -1 | rest | join "-" }}
{{- if has $ns_head (list "production" "p") }}
{{- regexReplaceAll "(.*)-ng(.*)" $ns_tail "${1}-foxford${2}" }}
{{- else }}
{{- $v := list (regexReplaceAll "(.)[^\\d]*(.+)" $ns_head "${1}${2}") $ns_tail | compact | join "/" }}
{{- regexReplaceAll "(.*)-ng(.*)" $v "${1}-foxford${2}" }}
{{- end }}
{{- end }}

{{/*
Ingress path.
*/}}
{{- define "storage.ingressPath" -}}
{{- $shortns := regexSplit "-" .Release.Namespace -1 | first }}
{{- list "" (include "storage.ingressPathNamespace" .) (include "storage.name" .) | join "/" }}
{{- end }}

{{/*
Create volumeMount name from audience and secret name
*/}}
{{- define "storage.volumeMountName" -}}
{{- $audience := index . 0 -}}
{{- $secret := index . 1 -}}
{{- printf "%s-%s-secret" $audience $secret | replace "." "-" | trunc 63 | trimSuffix "-" }}
{{- end }}
