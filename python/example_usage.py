#!/usr/bin/env python3
"""
Example usage of fa-control Python bindings

This demonstrates how to use the fa-control library to control audio
on Windows and Linux systems.
"""

import fa_control

# Get current platform
platform = fa_control.get_platform()
print(f"Running on: {platform}")

# Master volume control
print("\n=== Master Volume Control ===")
current_volume = fa_control.get_master_volume()
print(f"Current master volume: {current_volume:.2%}")

# Set master volume to 50%
fa_control.set_master_volume(0.5)
print("Set master volume to 50%")

# Check mute state
is_muted = fa_control.is_master_muted()
print(f"Master muted: {is_muted}")

# Toggle mute
new_mute_state = fa_control.toggle_master_mute()
print(f"Toggled mute to: {new_mute_state}")

# Microphone control
print("\n=== Microphone Control ===")
mic_volume = fa_control.get_microphone_volume()
print(f"Current microphone volume: {mic_volume:.2%}")

# Set microphone volume to 75%
fa_control.set_microphone_volume(0.75)
print("Set microphone volume to 75%")

mic_muted = fa_control.is_microphone_muted()
print(f"Microphone muted: {mic_muted}")

# Application-specific volume control
print("\n=== Application Volume Control ===")
apps = fa_control.get_active_audio_apps()
print(f"Found {len(apps)} active audio applications:")

for app in apps:
    print(f"  PID: {app.pid:6d} | Name: {app.name:30s} | Volume: {app.volume:6.1%} | Muted: {app.muted}")

# Example: Set volume for first application if any exist
if apps:
    first_app = apps[0]
    print(f"\nSetting volume for '{first_app.name}' (PID: {first_app.pid}) to 30%")
    fa_control.set_app_volume(first_app.pid, 0.3)

    # Mute the application
    print(f"Muting '{first_app.name}'")
    fa_control.set_app_mute(first_app.pid, True)

    # Check new state
    new_volume = fa_control.get_app_volume(first_app.pid)
    new_mute = fa_control.is_app_muted(first_app.pid)
    print(f"New state - Volume: {new_volume:.2%}, Muted: {new_mute}")