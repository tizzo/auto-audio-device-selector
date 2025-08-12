#!/bin/bash

# restart-service.sh - Full restart of audio-device-monitor service
# This script stops and starts the service completely

set -e

SERVICE_NAME="audio-device-monitor"
LAUNCHAGENT_LABEL="com.audiodevicemonitor.daemon"
LAUNCHAGENT_PATH="$HOME/Library/LaunchAgents/${LAUNCHAGENT_LABEL}.plist"

echo "🔄 Restarting $SERVICE_NAME service..."

# Function to check if service is running
check_service_status() {
    if launchctl list | grep -q "$LAUNCHAGENT_LABEL" 2>/dev/null; then
        return 0  # Service is loaded
    else
        return 1  # Service is not loaded
    fi
}

# Function to wait for service to stop
wait_for_stop() {
    local timeout=10
    local count=0
    
    while [ $count -lt $timeout ]; do
        if ! pgrep -f "$SERVICE_NAME" >/dev/null 2>&1; then
            return 0
        fi
        sleep 1
        count=$((count + 1))
    done
    return 1
}

# Stop the service
echo "Stopping service..."
if check_service_status; then
    if launchctl unload "$LAUNCHAGENT_PATH" 2>/dev/null; then
        echo "✅ Service unloaded via launchctl"
    else
        echo "⚠️  launchctl unload failed, trying direct process termination"
        
        # Try to kill process directly
        PID=$(pgrep -f "$SERVICE_NAME" 2>/dev/null || true)
        if [ -n "$PID" ]; then
            echo "Terminating process $PID..."
            if kill -TERM "$PID" 2>/dev/null; then
                echo "✅ Process terminated"
            else
                echo "❌ Failed to terminate process"
                exit 1
            fi
        fi
    fi
    
    # Wait for process to actually stop
    if wait_for_stop; then
        echo "✅ Service stopped successfully"
    else
        echo "⚠️  Service may still be running, proceeding anyway"
    fi
else
    echo "⚠️  Service was not loaded via launchctl"
    
    # Check if process is running directly
    PID=$(pgrep -f "$SERVICE_NAME" 2>/dev/null || true)
    if [ -n "$PID" ]; then
        echo "Found running process $PID, terminating..."
        if kill -TERM "$PID" 2>/dev/null; then
            wait_for_stop && echo "✅ Process terminated"
        else
            echo "❌ Failed to terminate process"
            exit 1
        fi
    else
        echo "✅ No running processes found"
    fi
fi

# Start the service
echo "Starting service..."
if [ -f "$LAUNCHAGENT_PATH" ]; then
    if launchctl load "$LAUNCHAGENT_PATH" 2>/dev/null; then
        echo "✅ Service loaded via launchctl"
        
        # Wait a moment for service to start
        sleep 2
        
        # Verify service is running
        if check_service_status; then
            PID=$(pgrep -f "$SERVICE_NAME" 2>/dev/null || true)
            if [ -n "$PID" ]; then
                echo "✅ Service is running with PID: $PID"
            else
                echo "⚠️  Service loaded but process not found yet"
            fi
        else
            echo "❌ Service failed to start"
            exit 1
        fi
    else
        echo "❌ Failed to load service via launchctl"
        echo "💡 Try installing the service: cargo run -- install-service"
        exit 1
    fi
else
    echo "❌ LaunchAgent plist not found at: $LAUNCHAGENT_PATH"
    echo "💡 Install the service first: cargo run -- install-service"
    echo "Or run manually: cargo run -- --daemon"
    exit 1
fi

echo "🎉 Service restart complete!"
echo "📋 Monitor logs: tail -f ~/.local/share/audio-device-monitor/logs/audio-device-monitor.log.*"
echo "📊 Check status: launchctl list | grep $LAUNCHAGENT_LABEL"