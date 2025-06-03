LOGS="./logs/macro/authdag"
CLIENTS=2
OPERATIONS=250

echo "Starting backend"
cargo run --manifest-path ./backend/Cargo.toml --bin socket --features bench &
while ! netstat -an | grep LISTEN | grep -q 20076; do
    sleep 0.1
done

echo "Starting frontend"
for i in $(seq 1 "$CLIENTS"); do 
    NODE_ENV="bench" npm --prefix ./frontend run macro "$((3000+$i))" &
    while ! netstat -an | grep LISTEN | grep -q "$((3000+$i))"; do
        sleep 0.1
    done
done

echo "Starting workload scripts"
python3 ./scripts/macro/workload/main.py "$CLIENTS" "$OPERATIONS"

jobs -p | xargs kill

echo "Collecting logs"
mkdir --parents $LOGS

for file in ./*.csv; do
    filename=$(basename "$file")
    ftitle="${filename%.csv}"
    mv "$file" "$LOGS/script_${ftitle}.csv"
done

for file in ./frontend/src/*.csv; do
    filename=$(basename "$file")
    ftitle="${filename%.csv}"
  mv "$file" "$LOGS/front_${ftitle}.csv"
done

for file in ./backend/*.csv; do
    filename=$(basename "$file")
    ftitle="${filename%.csv}"
  mv "$file" "$LOGS/back_${ftitle}.csv"
done
