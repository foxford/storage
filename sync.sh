#! /bin/bash

export DOC_WEB_BUCKET='docs-netology-group.services'
lftp -c "open -u ${SFTP_USER},${SFTP_PASSWORD} sftp.selcdn.ru; mirror --parallel=4 --no-empty-dirs --no-perms --exclude-glob .DS_Store --reverse --verbose -e docs/book ${DOC_WEB_BUCKET}/storage"