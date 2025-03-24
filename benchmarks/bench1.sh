SAMPLE=1000
APPLY_N=5
LOGS="./logs/bench1/"

# start  backend
cargo run --manifest-path ./backend/Cargo.toml --bin socket &
while ! netstat -an | grep LISTEN | grep -q 20076; do
    sleep 0.1
done

# start  frontend
npm --prefix ./frontend start &
while ! netstat -an | grep LISTEN | grep -q 3000; do
    sleep 0.1
done

# send curl requests to frontend
for i in $(seq 1 $SAMPLE); do
    for i in $(seq 1 $APPLY_N); do
        curl --location 'localhost:3000/map' \
            --header 'Content-Type: application/json' \
            --data "{\"key\":\"123\",\"value\":\"$i\"}"

        curl --location 'localhost:3000/save'
    done
done

jobs -p | xargs kill

# put logs in place
# take every file and put it at /logs folder
mkdir --parents $LOGS
mv ./*.csv $LOGS
