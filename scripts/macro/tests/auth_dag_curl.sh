LOGS="./logs/macro/scenario1/"
CLIENTS=2
TIME=10
SESSION="Thesis"
WINDOW="tests"

tmux new-window -t "$SESSION" -n "$WINDOW"
tmux send-keys -t "$SESSION:$WINDOW" "nix develop .#run" C-m
tmux switch-client -t "$SESSION:$WINDOW"

tmux send-keys -t "$SESSION:$WINDOW" "cargo run --manifest-path ./backend/Cargo.toml --features debug --bin socket" C-m
while ! netstat -an | grep LISTEN | grep -q 20076; do sleep 0.1; done

# Create new panes for each frontend client
for i in $(seq 1 "$CLIENTS"); do
    tmux split-window -t "$SESSION:$WINDOW" -v
    tmux select-layout -t "$SESSION:$WINDOW" tiled
    tmux send-keys -t "$SESSION:$WINDOW" "NODE_ENV=\"dev\" npm --prefix ./frontend run macro $((3000+$i))" C-m
    while ! netstat -an | grep LISTEN | grep -q $((3000+$i)); do sleep 0.1; done
done

# New pane for Python workload
tmux split-window -t "$SESSION:$WINDOW" -v
tmux select-layout -t "$SESSION:$WINDOW" tiled
tmux send-keys -t "$SESSION:$WINDOW" "python3 ./scripts/workload/initial_state.py $CLIENTS" C-m

curl --location 'localhost:3001/pharmacy/1/prescriptions'
