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
    docker save fiatprices | bzip2 | ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no $SSH_HOST 'bunzip2 | docker load'
    ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no $SSH_HOST 'cd /opt/fiatprices; docker-compose up -d'
}

[[ "$1" == "today" ]] && {
    export DT=$(date -I)
    export DTW="'"$DT"'"
    export MARKET=${MARKET:-ethereum}
    export REVDATE=$(date -I | cut -d- -f3)-$(date -I | cut -d- -f2)-$(date -I | cut -d- -f1)
    export DATA=$(curl -s "https://api.coingecko.com/api/v3/coins/$MARKET/history?date=$REVDATE" | \
	    jq -r '.market_data.current_price | [.eur, .usd, .rub, .cny, .cad, .jpy, .gbp] | @csv')

    export SQL_INSERT="INSERT INTO price_$MARKET (ts, eur, usd, rub, cny, cad, jpy, gbp) VALUES ($DTW, $DATA) ON CONFLICT DO NOTHING;"
    export SQL_CHECK="SELECT ts, eur, usd FROM price_$MARKET ORDER BY "ts" DESC LIMIT 5;"

    echo $SQL_INSERT | ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no $SSH_HOST \
       'docker exec -i postgres psql -U postgres -h 127.0.0.1 -d fiatprices'
    echo $SQL_CHECK | ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no $SSH_HOST \
       'docker exec -i postgres psql -U postgres -h 127.0.0.1 -d fiatprices'

}
