#!/bin/bash
# Quick start Conway's Game of Life as background

# Build if not exists
if [ ! -f target/release/conway ]; then
    echo "Building Conway's Game of Life..."
    cargo build --release
fi

# Kill any existing instance
pkill -f "conway.*--random" 2>/dev/null

# Start in transparent terminal (simplest method)
echo "Starting Conway's Game of Life background..."
gnome-terminal --hide-menubar --geometry=200x60+0+0 --zoom=0.5 -- bash -c "cd /home/riot/conway_v1 && ./target/release/conway --random; exec bash" &

echo "Background started! Adjust terminal transparency in preferences."
echo "Press Ctrl+C in the terminal to stop."