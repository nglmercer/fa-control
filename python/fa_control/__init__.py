"""
fa-control - Cross-platform audio control library for Python

This module provides functionality to control audio devices and application volumes
on Windows and Linux systems.

Example usage:
    >>> import fa_control
    >>> fa_control.get_master_volume()
    0.5
    >>> fa_control.set_master_volume(0.75)
    >>> fa_control.is_master_muted()
    False
    >>> fa_control.get_active_audio_apps()
    [AppInfo(pid=1234, name='Spotify', volume=0.80, muted=False)]
"""

from fa_control._internal import (
    get_master_volume,
    set_master_volume,
    is_master_muted,
    toggle_master_mute,
    set_master_mute,
    get_app_volume,
    set_app_volume,
    is_app_muted,
    set_app_mute,
    get_active_audio_apps,
    get_microphone_volume,
    set_microphone_volume,
    is_microphone_muted,
    toggle_microphone_mute,
    set_microphone_mute,
    get_platform,
    AppInfo,
)

__all__ = [
    "get_master_volume",
    "set_master_volume",
    "is_master_muted",
    "toggle_master_mute",
    "set_master_mute",
    "get_app_volume",
    "set_app_volume",
    "is_app_muted",
    "set_app_mute",
    "get_active_audio_apps",
    "get_microphone_volume",
    "set_microphone_volume",
    "is_microphone_muted",
    "toggle_microphone_mute",
    "set_microphone_mute",
    "get_platform",
    "AppInfo",
]

__version__ = "0.1.3"
