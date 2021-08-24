{{- define "app_envs" }}
- name: AWS_ACCESS_KEY_ID
  value: {{ pluck .Values.werf.env .Values.app.aws.ng.access_key_id | first | default .Values.app.aws.ng.access_key_id._default | quote }}
- name: AWS_SECRET_ACCESS_KEY
  value: {{ pluck .Values.werf.env .Values.app.aws.ng.secret_access_key | first | default .Values.app.aws.ng.secret_access_key._default | quote }}
- name: AWS_ENDPOINT
  value: {{ pluck .Values.werf.env .Values.app.aws.ng.endpoint | first | default .Values.app.aws.ng.endpoint._default | quote }}
- name: AWS_REGION
  value: {{ pluck .Values.werf.env .Values.app.aws.ng.region | first | default .Values.app.aws.ng.region._default | quote }}
- name: YANDEX_AWS_ACCESS_KEY_ID
  value: {{ pluck .Values.werf.env .Values.app.aws.ng.access_key_id | first | default .Values.app.aws.ng.access_key_id._default | quote }}
- name: YANDEX_AWS_SECRET_ACCESS_KEY
  value: {{ pluck .Values.werf.env .Values.app.aws.ng.secret_access_key | first | default .Values.app.aws.ng.secret_access_key._default | quote }}
- name: YANDEX_AWS_ENDPOINT
  value: {{ pluck .Values.werf.env .Values.app.aws.ng.endpoint | first | default .Values.app.aws.ng.endpoint._default | quote }}
- name: YANDEX_AWS_REGION
  value: {{ pluck .Values.werf.env .Values.app.aws.ng.region | first | default .Values.app.aws.ng.region._default | quote }}
- name: YANDEX_B2G_AWS_ACCESS_KEY_ID
  value: {{ pluck .Values.werf.env .Values.app.aws.b2g.access_key_id | first | default .Values.app.aws.b2g.access_key_id._default | quote }}
- name: YANDEX_B2G_AWS_SECRET_ACCESS_KEY
  value: {{ pluck .Values.werf.env .Values.app.aws.b2g.secret_access_key | first | default .Values.app.aws.b2g.secret_access_key._default | quote }}
- name: YANDEX_B2G_AWS_ENDPOINT
  value: {{ pluck .Values.werf.env .Values.app.aws.b2g.endpoint | first | default .Values.app.aws.b2g.endpoint._default | quote }}
- name: YANDEX_B2G_AWS_REGION
  value: {{ pluck .Values.werf.env .Values.app.aws.b2g.region | first | default .Values.app.aws.b2g.region._default | quote }}
- name: RUST_LOG
  value: {{ pluck .Values.werf.env .Values.app.rust_log | first | default .Values.app.rust_log._default | quote }}
- name: CACHE_ENABLED
  value: {{ pluck .Values.werf.env .Values.app.cache.enabled | first | default .Values.app.cache.enabled._default | quote }}
- name: CACHE_POOL_SIZE
  value: {{ pluck .Values.werf.env .Values.app.cache.pool_size | first | default .Values.app.cache.pool_size._default | quote }}
- name: CACHE_POOL_IDLE_SIZE
  value: {{ pluck .Values.werf.env .Values.app.cache.pool_idle_size | first | default .Values.app.cache.pool_idle_size._default | quote }}
- name: CACHE_POOL_TIMEOUT
  value: {{ pluck .Values.werf.env .Values.app.cache.pool_timeout | first | default .Values.app.cache.pool_timeout._default | quote }}
- name: CACHE_EXPIRATION_TIME
  value: {{ pluck .Values.werf.env .Values.app.cache.expiration_time | first | default .Values.app.cache.expiration_time._default | quote }}
- name: CACHE_URL
  value: {{ pluck .Values.werf.env .Values.app.cache.url | first | default .Values.app.cache.url._default | quote }}
{{- end }}
