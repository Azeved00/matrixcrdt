LOGS="./logs/macro/scenario1/"
CLIENTS=13
TIME=10

# start  backend
cargo run --manifest-path ./backend/Cargo.toml --bin socket &
while ! netstat -an | grep LISTEN | grep -q 20076; do
    sleep 0.1
done

# start  frontend
for i in $(seq 1 "$CLIENTS"); do 
    npm --prefix ./frontend run macro "$((3000+$i))" &
    while ! netstat -an | grep LISTEN | grep -q "$((3000+$i))"; do
        sleep 0.1
    done
done



# start workload
# 2 clients 
python3 ./scripts/workload/main.py "$CLIENTS" "$TIME"


jobs -p | xargs kill

# put logs in place
# take every file and put it at /logs folder
mkdir --parents $LOGS
mv ./*.csv $LOGS
