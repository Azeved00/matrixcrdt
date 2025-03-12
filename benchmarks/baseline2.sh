SAMPLE=3
APPLY_N=5
LOGS="./logs"

# start  backend
cargo run --manifest-path ./backend/Cargo.toml --bin baseline &
while ! nc -z localhost 20076; do
    sleep 0.1
done

# start  frontend 1
npm --prefix ./frontend start 3000 &
while ! nc -z localhost 3000; do
    sleep 0.1
done

# start  frontend 2
npm --prefix ./frontend start 3001 &
while ! nc -z localhost 3001; do
    sleep 0.1
done


# send curl requests to frontend
for i in $(seq 1 $SAMPLE); do
    for i in $(seq 1 $APPLY_N); do
        curl --location 'localhost:3000/map' \
            -o /dev/null  -s \
            --header 'Content-Type: application/json' \
            --data "{\"key\":\"123\",\"value\":\"$i\"}"

        curl --location 'localhost:3000/save'  -o /dev/null  -s
    done

    curl --location 'localhost:3001/query'  -o /dev/null  -s
done

jobs -p | xargs kill

# put logs in place
# take every file and put it at /logs folder
mv ./*.csv ./logs/
