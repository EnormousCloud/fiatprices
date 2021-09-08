#!/usr/bin/env bash

export SSH_HOST=root@enormous.cloud
export DB=fiatprices

set -ex

[[ "$1" == "db:start" ]] && {
    docker run -d --name postgres -p 5432:5432 \
        -e POSTGRES_PASSWORD=password \
        -e POSTGRES_DB=$DB \
        postgres
}

[[ "$1" == "db:shell" ]] && {
    docker exec -it postgres psql -U postgres -d $DB
}

[[ "$1" == "db:save" ]] && {
    docker exec -t postgres pg_dumpall -c -U postgres -h 127.0.0.1 -l fiatprices  | tee >(gzip --stdout > fiatprices.sql.gz)
}

[[ "$1" == "db:download" ]] && {
    rm -f $DB.sql.gz || true
    ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no \
        $SSH_HOST /opt/fiatprices/backup.sh
    scp -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no \
        $SSH_HOST:/opt/fiatprices/$DB.sql.gz \
        ./$DB.sql.gz
    cat $DB.sql.gz | gzip -d - | docker exec -i postgres psql -U postgres
}

[[ "$1" == "db:upload" ]] && {
    scp -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no \
        ./$DB.sql.gz \
        $SSH_HOST:/opt/$DB.sql.gz
}


[[ "$1" == "publish" ]] && {
    docker build -t fiatprices .
    docker save fiatprices | bzip2 | ssh $SSH_HOST 'bunzip2 | docker load'
    ssh $SSH_HOST 'cd /opt/fiatprices; docker-compose up -d'
}
