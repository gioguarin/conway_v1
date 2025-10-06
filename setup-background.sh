#!/bin/bash
# Conway's Game of Life - Animated Background Setup Script

echo "Conway's Game of Life - Background Setup"
echo "========================================="
echo ""
echo "Choose setup method:"
echo "1) Transparent Terminal (Simple)"
echo "2) Desktop Background with xwinwrap (Advanced)"
echo "3) Both options"
echo ""
read -p "Enter choice [1-3]: " choice

# Build optimized version
echo "Building optimized version..."
cargo build --release

# Create launch scripts directory
mkdir -p ~/.local/bin

# Method 1: Transparent Terminal
if [ "$choice" = "1" ] || [ "$choice" = "3" ]; then
    echo "Setting up transparent terminal method..."

    cat > ~/.local/bin/conway-transparent.sh << 'EOF'
#!/bin/bash
# Kill any existing instances
pkill -f "conway.*--random" 2>/dev/null

# Get screen dimensions
SCREEN_WIDTH=$(xdpyinfo | awk '/dimensions:/ {print $2}' | cut -d'x' -f1)
SCREEN_HEIGHT=$(xdpyinfo | awk '/dimensions:/ {print $2}' | cut -d'x' -f2)

# Launch with transparent terminal
cd /home/riot/conway_v1
gnome-terminal --window-with-profile=ConwayBG --geometry="${SCREEN_WIDTH}x${SCREEN_HEIGHT}+0+0" -- ./target/release/conway --random &
EOF

    chmod +x ~/.local/bin/conway-transparent.sh

    # Create GNOME Terminal profile
    echo "Creating terminal profile..."
    dconf write /org/gnome/terminal/legacy/profiles:/:conway-bg/visible-name "'Conway Background'"
    dconf write /org/gnome/terminal/legacy/profiles:/:conway-bg/background-transparency-percent "30"
    dconf write /org/gnome/terminal/legacy/profiles:/:conway-bg/use-transparent-background "true"
    dconf write /org/gnome/terminal/legacy/profiles:/:conway-bg/scrollbar-policy "'never'"
fi

# Method 2: xwinwrap
if [ "$choice" = "2" ] || [ "$choice" = "3" ]; then
    echo "Setting up xwinwrap method..."

    # Check if xwinwrap is installed
    if ! command -v xwinwrap &> /dev/null; then
        echo "Installing xwinwrap..."
        sudo apt update
        sudo apt install -y xwinwrap
    fi

    cat > ~/.local/bin/conway-wallpaper.sh << 'EOF'
#!/bin/bash
# Kill any existing xwinwrap instances
killall xwinwrap 2>/dev/null

# Launch as wallpaper
cd /home/riot/conway_v1
xwinwrap -fs -ov -nf -b -sh rectangle -- alacritty --config-file /home/riot/conway_v1/alacritty-bg.yml -e ./target/release/conway --random &
EOF

    chmod +x ~/.local/bin/conway-wallpaper.sh
fi

# Create Alacritty config for background use
cat > /home/riot/conway_v1/alacritty-bg.yml << 'EOF'
window:
  opacity: 0.5
  decorations: none
  startup_mode: Fullscreen
  padding:
    x: 0
    y: 0

font:
  size: 4.0
  normal:
    family: monospace

colors:
  primary:
    background: '#000000'
    foreground: '#00ff00'

  normal:
    black:   '#000000'
    red:     '#ff0000'
    green:   '#00ff00'
    yellow:  '#ffff00'
    blue:    '#0000ff'
    magenta: '#ff00ff'
    cyan:    '#00ffff'
    white:   '#ffffff'

cursor:
  style:
    shape: Block
    blinking: Never
EOF

# Create desktop entry for easy access
cat > ~/.local/share/applications/conway-background.desktop << 'EOF'
[Desktop Entry]
Version=1.0
Type=Application
Name=Conway's Life Background
Comment=Run Conway's Game of Life as animated background
Icon=applications-games
Exec=/home/riot/.local/bin/conway-transparent.sh
Categories=Game;Utility;
Terminal=false
StartupNotify=false
EOF

# Create autostart entry
mkdir -p ~/.config/autostart
cat > ~/.config/autostart/conway-background.desktop << 'EOF'
[Desktop Entry]
Type=Application
Name=Conway Background
Comment=Start Conway's Game of Life background on login
Exec=/home/riot/.local/bin/conway-transparent.sh
Hidden=false
NoDisplay=false
X-GNOME-Autostart-enabled=false
X-GNOME-Autostart-delay=5
EOF

echo ""
echo "Setup complete!"
echo ""
echo "You can now:"
echo "  1. Run 'conway-transparent.sh' for transparent terminal background"
echo "  2. Run 'conway-wallpaper.sh' for xwinwrap wallpaper (if installed)"
echo "  3. Find 'Conway's Life Background' in your application menu"
echo "  4. Enable auto-start by editing ~/.config/autostart/conway-background.desktop"
echo "     (change X-GNOME-Autostart-enabled to true)"
echo ""
echo "Tips:"
echo "  - Adjust opacity in alacritty-bg.yml (0.0-1.0)"
echo "  - Change colors in the config files"
echo "  - Modify font size for different cell densities"