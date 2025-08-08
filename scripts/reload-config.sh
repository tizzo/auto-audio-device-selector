#!/bin/bash

# reload-config.sh - Hot reload configuration for audio-device-monitor service
# This script sends SIGHUP to the service to reload configuration without restarting

set -e

SERVICE_NAME="audio-device-monitor"
LAUNCHAGENT_LABEL="com.audiodevicemonitor.daemon"

echo "🔄 Reloading configuration for $SERVICE_NAME..."

# Try to find the process ID
PID=$(pgrep -f "$SERVICE_NAME" 2>/dev/null || true)

if [ -n "$PID" ]; then
    echo "Found service running with PID: $PID"
    echo "Sending SIGHUP signal to reload configuration..."
    
    if kill -HUP "$PID" 2>/dev/null; then
        echo "✅ Configuration reload signal sent successfully"
        echo "📋 Check logs to verify reload: tail -f ~/.local/share/audio-device-monitor/logs/audio-device-monitor.log.*"
    else
        echo "❌ Failed to send reload signal to PID $PID"
        exit 1
    fi
else
    echo "⚠️  Service not found running as process"
    
    # Try launchctl approach for system service
    echo "Attempting to reload via launchctl..."
    if launchctl list | grep -q "$LAUNCHAGENT_LABEL" 2>/dev/null; then
        if launchctl kill -HUP "gui/$(id -u)/$LAUNCHAGENT_LABEL" 2>/dev/null; then
            echo "✅ Configuration reload via launchctl successful"
        else
            echo "❌ Failed to reload via launchctl"
            echo "💡 Try running: ./scripts/restart-service.sh"
            exit 1
        fi
    else
        echo "❌ Service not found via launchctl either"
        echo "💡 Service may not be running. Try: cargo run -- --daemon"
        echo "Or install as service: cargo run -- install-service"
        exit 1
    fi
fi

echo "🎉 Configuration reload complete!"