SESSION="Thesis"
WINDOW="test"
CLIENTS=2
OPERATIONS=250
LOGS="./logs/macro/matrix"

# Pane 0: Backend
tmux send-keys -t $SESSION:$WINDOW.0 'echo "Starting backend"' C-m
tmux send-keys -t $SESSION:$WINDOW.0 '
cargo run --manifest-path ./backend/Cargo.toml --bin matrix --features bench, debug
' C-m

# Wait for backend to start listening
tmux send-keys -t $SESSION:$WINDOW.0 '
while ! netstat -an | grep LISTEN | grep -q 20076; do sleep 0.1; done
' C-m

# Frontend clients
for i in $(seq 1 "$CLIENTS"); do
    PORT=$((3000 + i))

    # Split a new vertical pane
    tmux split-window -t $SESSION:$WINDOW -v
    tmux select-layout -t $SESSION:$WINDOW tiled

    tmux send-keys -t $SESSION:$WINDOW "
echo \"Starting frontend $i on port $PORT\"
NODE_ENV=bench npm --prefix ./frontend run macro $PORT
" C-m

    tmux send-keys -t $SESSION:$WINDOW "
while ! netstat -an | grep LISTEN | grep -q $PORT; do sleep 0.1; done
" C-m
done

# Workload script
tmux split-window -t $SESSION:$WINDOW -v
tmux select-layout -t $SESSION:$WINDOW tiled
tmux send-keys -t $SESSION:$WINDOW "
echo 'Starting workload script'
python3 ./scripts/macro/workload/main.py \"$CLIENTS\" \"$OPERATIONS\"
" C-m

# Logging pane
tmux split-window -t $SESSION:$WINDOW -v
tmux select-layout -t $SESSION:$WINDOW tiled
tmux send-keys -t $SESSION:$WINDOW "
echo 'Collecting logs'
mkdir -p $LOGS

for file in ./*.csv; do
    filename=\$(basename \"\$file\")
    ftitle=\"\${filename%.csv}\"
    mv \"\$file\" \"$LOGS/script_\${ftitle}.csv\"
done

for file in ./frontend/src/*.csv; do
    filename=\$(basename \"\$file\")
    ftitle=\"\${filename%.csv}\"
    mv \"\$file\" \"$LOGS/front_\${ftitle}.csv\"
done

for file in ./backend/*.csv; do
    filename=\$(basename \"\$file\")
    ftitle=\"\${filename%.csv}\"
    mv \"\$file\" \"$LOGS/back_\${ftitle}.csv\"
done
" C-m

# Attach to the session in window "test"
tmux select-window -t $SESSION:$WINDOW
tmux attach -t $SESSION
