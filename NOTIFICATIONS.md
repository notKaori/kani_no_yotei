# 🦀 Kani No Yotei - Notification System

This document explains how to set up and use the notification system for your todo list.

## Overview

The notification system uses **cron jobs** (not background daemons) following best practices:

- ✅ **Resource Efficient**: Only runs when needed
- ✅ **Reliable**: System-managed scheduling
- ✅ **Simple**: Easy to set up and manage
- ✅ **Secure**: Runs with user permissions

## Quick Setup

### 1. Build the Program
```bash
cargo build --release
```

### 2. Start Tiramisu (for desktop notifications)
```bash
# Start tiramisu in the background
tiramisu &

# Or add to your startup scripts to start automatically
```

### 3. Run the Setup Script
```bash
./setup_notifications.sh
```

The script will guide you through setting up automatic notifications and check if tiramisu is running.

## Manual Setup

### 1. Add a Cron Job
```bash
# Edit your crontab
crontab -e

# Add one of these lines (choose based on your needs):
*/5 * * * * cd /path/to/your/todos && /path/to/kny notify -m 15
*/10 * * * * cd /path/to/your/todos && /path/to/kny notify -m 15
*/15 * * * * cd /path/to/your/todos && /path/to/kny notify -m 15
```

### 2. Test Notifications
```bash
# Test manually
kny notify -v

# Test with custom timing
kny notify -m 30 -v  # Check for tasks due within 30 minutes
```

## Usage Examples

### Adding Tasks with Due Dates
```bash
# Task due at specific time today
kny add "Doctor appointment @ 14:30"

# Task due tomorrow
kny add "Call mom @ tomorrow"

# Task due on specific date
kny add "Project deadline @ 2024-12-31"

# Task due on specific date and time
kny add "Meeting @ 2024-12-25 14:30"
```

**Important**: The entire task including the `@` symbol must be in quotes. If you don't quote it properly, the shell will split it into multiple arguments:

```bash
# ❌ Wrong - creates 3 separate tasks
kny add "do laundry" @ tomorrow

# ✅ Correct - creates 1 task with due date
kny add "do laundry @ tomorrow"
```

### Notification Commands
```bash
# Check for tasks due within 15 minutes (default)
kny notify

# Check for tasks due within 30 minutes
kny notify -m 30

# Verbose output showing what's being checked
kny notify -v

# Check for tasks due within 1 hour
kny notify -m 60 -v
```

## Supported Date/Time Formats

- **Full datetime**: `2024-12-25 14:30`
- **Date only**: `2024-12-25` (defaults to 9:00 AM)
- **US format**: `01/15/2024 14:30`
- **Time only**: `14:30` (assumes today)
- **Relative dates**: `tomorrow`, `today`

## How It Works

1. **Cron Job**: Runs every 5-15 minutes (your choice)
2. **Lock File**: Prevents overlapping executions
3. **Task Check**: Scans for overdue and due-soon tasks
4. **Notifications**: Sends desktop notifications for due tasks
5. **Cleanup**: Removes lock file when done

## Troubleshooting

### No Desktop Notifications Appearing
- **Start tiramisu**: `tiramisu &`
- **Check if running**: `pgrep tiramisu`
- **Test manually**: `kny notify -v`
- **Fallback**: If desktop notifications fail, the program will show terminal notifications

### Cron Job Not Running
- Check cron service: `systemctl status cron` (Linux)
- Check cron logs: `journalctl -u cron` (Linux)
- Verify path in cron job is absolute

### Lock File Issues
- Lock files are automatically cleaned up
- If stuck, manually remove: `rm ToDo.txt.lock`

### Tiramisu Issues
- **Install tiramisu**: `nix-env -iA nixpkgs.tiramisu` (Nix) or use your package manager
- **Start tiramisu**: `tiramisu &`
- **Auto-start**: Add `tiramisu &` to your `.xinitrc` or desktop environment startup
- **Alternative**: The program will fall back to terminal notifications if desktop notifications fail

## Best Practices

### Cron Job Frequency
- **Every 5 minutes**: For time-sensitive tasks
- **Every 10 minutes**: Good balance of responsiveness and efficiency
- **Every 15 minutes**: For less urgent tasks

### Notification Timing
- **15 minutes before**: Good for most tasks
- **30 minutes before**: For important meetings
- **60 minutes before**: For travel or preparation time

### File Organization
- Keep your todo file in a consistent location
- Use absolute paths in cron jobs
- Consider using a dedicated directory for todos

## Advanced Usage

### Multiple Todo Files
```bash
# Set up notifications for different projects
crontab -e

# Work todos
*/10 * * * * cd /home/user/work && /path/to/kny notify -m 15

# Personal todos  
*/15 * * * * cd /home/user/personal && /path/to/kny notify -m 30
```

### Custom Notification Timing
```bash
# Morning check for today's tasks
0 8 * * * cd /path/to/todos && /path/to/kny notify -m 60

# Afternoon check for urgent tasks
*/5 13-17 * * * cd /path/to/todos && /path/to/kny notify -m 15
```

## Security Notes

- Cron jobs run with your user permissions
- No elevated privileges required
- Lock files prevent race conditions
- All file operations are local

## Why Cron Jobs vs Background Process?

| Feature | Cron Jobs ✅ | Background Process |
|---------|-------------|-------------------|
| Resource Usage | Low (runs only when needed) | High (always running) |
| Reliability | High (system managed) | Medium (needs monitoring) |
| Complexity | Low | High |
| Security | High (user permissions) | Medium (process management) |
| Maintenance | Low | High |

For periodic task checking, cron jobs are the industry standard and best practice.
