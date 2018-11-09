#! /bin/bash

MEDIA_EDITOR_WEB_BUCKET = 'docs.netology-group.services'

lftp -c "open -u ${SFTP_USER},${SFTP_PASSWORD} sftp.selcdn.ru; mirror --parallel=4 --no-empty-dirs --no-perms --exclude-glob .DS_Store --reverse --verbose -e docs/html ${MEDIA_EDITOR_WEB_BUCKET}/storage"