for i in $(seq 1 2); do
    for i in $(seq 1 3); do
        curl --location 'localhost:3000/map' \
            --header 'Content-Type: application/json' \
            --data "{\"key\":\"123\",\"value\":\"$i\"}"

        curl --location 'localhost:3000/map' \
            --header 'Content-Type: application/json' \
            --data "{\"key\":\"123\",\"value\":\"$i\"}"
        echo ""

        curl --location 'localhost:3000/map'
        echo ""

        curl --location 'localhost:3000/save'  -o /dev/null  -s
        echo ""
    done

    echo "querying"
    curl --location 'localhost:3000/query'  -o /dev/null  -s
done
