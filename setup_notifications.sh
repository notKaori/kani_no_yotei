#!/bin/bash

# Kani No Yotei - Notification Setup Script
# This script helps you set up automatic notifications for your todo list

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🦀 Kani No Yotei - Notification Setup${NC}"
echo "=================================="
echo

# Check if tiramisu is available and running
if command -v tiramisu >/dev/null 2>&1; then
    if ! pgrep tiramisu >/dev/null; then
        echo -e "${YELLOW}⚠️  Tiramisu is installed but not running${NC}"
        echo "Starting tiramisu for notifications..."
        tiramisu &
        sleep 1
        if pgrep tiramisu >/dev/null; then
            echo -e "${GREEN}✅ Tiramisu started successfully${NC}"
        else
            echo -e "${RED}❌ Failed to start tiramisu${NC}"
            echo "You may need to start it manually: tiramisu &"
        fi
    else
        echo -e "${GREEN}✅ Tiramisu is running${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  Tiramisu not found${NC}"
    echo "For desktop notifications, install tiramisu:"
    echo "  - Nix: nix-env -iA nixpkgs.tiramisu"
    echo "  - Or use your system package manager"
    echo "  - Then run: tiramisu &"
fi
echo

# Get the absolute path to the kny binary
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KNY_BINARY="$SCRIPT_DIR/target/release/kny"

# Check if release binary exists, fallback to debug
if [ ! -f "$KNY_BINARY" ]; then
    KNY_BINARY="$SCRIPT_DIR/target/debug/kny"
fi

if [ ! -f "$KNY_BINARY" ]; then
    echo -e "${RED}❌ Error: kny binary not found!${NC}"
    echo "Please run 'cargo build' or 'cargo build --release' first."
    exit 1
fi

echo -e "${GREEN}✅ Found kny binary: $KNY_BINARY${NC}"
echo

# Get current working directory for todo file
TODO_DIR="$(pwd)"
TODO_FILE="$TODO_DIR/ToDo.txt"

echo -e "${BLUE}📁 Todo file location: $TODO_FILE${NC}"
echo

# Check if todo file exists
if [ ! -f "$TODO_FILE" ]; then
    echo -e "${YELLOW}⚠️  Warning: ToDo.txt not found in current directory${NC}"
    echo "The notification system will work, but you may want to create some tasks first."
    echo
fi

# Function to add cron job
add_cron_job() {
    local interval="$1"
    local minutes_before="$2"
    
    # Create the cron command
    CRON_CMD="cd $TODO_DIR && $KNY_BINARY notify -m $minutes_before"
    
    # Add to crontab
    (crontab -l 2>/dev/null; echo "$interval $CRON_CMD") | crontab -
    
    echo -e "${GREEN}✅ Added cron job: Check every $interval for tasks due within $minutes_before minutes${NC}"
}

# Function to show current cron jobs
show_cron_jobs() {
    echo -e "${BLUE}📋 Current cron jobs for kny:${NC}"
    crontab -l 2>/dev/null | grep "kny notify" || echo "No kny notification jobs found."
    echo
}

# Function to remove all kny cron jobs
remove_cron_jobs() {
    echo -e "${YELLOW}🗑️  Removing all kny notification cron jobs...${NC}"
    crontab -l 2>/dev/null | grep -v "kny notify" | crontab -
    echo -e "${GREEN}✅ Removed all kny notification cron jobs${NC}"
    echo
}

# Main menu
while true; do
    echo -e "${BLUE}Choose an option:${NC}"
    echo "1) Add notification check every 5 minutes (15 min advance warning)"
    echo "2) Add notification check every 10 minutes (15 min advance warning)"
    echo "3) Add notification check every 15 minutes (15 min advance warning)"
    echo "4) Add custom notification schedule"
    echo "5) Show current notification jobs"
    echo "6) Remove all notification jobs"
    echo "7) Test notification system"
    echo "8) Exit"
    echo
    read -p "Enter your choice (1-8): " choice
    
    case $choice in
        1)
            add_cron_job "*/5 * * * *" "15"
            ;;
        2)
            add_cron_job "*/10 * * * *" "15"
            ;;
        3)
            add_cron_job "*/15 * * * *" "15"
            ;;
        4)
            echo
            echo -e "${BLUE}Custom Schedule Setup${NC}"
            echo "Enter cron schedule (e.g., '*/5 * * * *' for every 5 minutes):"
            read -p "Schedule: " schedule
            echo "Enter minutes before due date to send notification (default: 15):"
            read -p "Minutes before: " minutes
            minutes=${minutes:-15}
            add_cron_job "$schedule" "$minutes"
            ;;
        5)
            show_cron_jobs
            ;;
        6)
            remove_cron_jobs
            ;;
        7)
            echo -e "${BLUE}🧪 Testing notification system...${NC}"
            cd "$TODO_DIR"
            "$KNY_BINARY" notify -v
            echo
            ;;
        8)
            echo -e "${GREEN}👋 Setup complete!${NC}"
            echo
            echo -e "${BLUE}💡 Tips:${NC}"
            echo "• Use 'kny notify -v' to test notifications manually"
            echo "• Use 'kny notify -m 30' to check for tasks due within 30 minutes"
            echo "• Use 'crontab -l' to view all your cron jobs"
            echo "• Use 'crontab -e' to manually edit cron jobs"
            echo
            break
            ;;
        *)
            echo -e "${RED}❌ Invalid choice. Please enter 1-8.${NC}"
            echo
            ;;
    esac
done
