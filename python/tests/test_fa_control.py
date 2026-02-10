import pytest
import fa_control


class TestPlatform:
    """Test platform detection"""
    def test_get_platform(self):
        platform = fa_control.get_platform()
        assert platform in ["windows", "linux", "unsupported"]


class TestMasterVolume:
    """Test master volume control"""
    def test_get_master_volume(self):
        volume = fa_control.get_master_volume()
        assert 0.0 <= volume <= 1.0

    def test_set_master_volume(self):
        # Get current volume
        original = fa_control.get_master_volume()
        
        # Set to a valid value
        fa_control.set_master_volume(0.5)
        assert fa_control.get_master_volume() == pytest.approx(0.5, rel=1e-2)
        
        # Restore original
        fa_control.set_master_volume(original)

    def test_set_master_volume_invalid(self):
        with pytest.raises(ValueError, match="Volume must be between 0.0 and 1.0"):
            fa_control.set_master_volume(1.5)
        
        with pytest.raises(ValueError, match="Volume must be between 0.0 and 1.0"):
            fa_control.set_master_volume(-0.1)

    def test_is_master_muted(self):
        muted = fa_control.is_master_muted()
        assert isinstance(muted, bool)

    def test_toggle_master_mute(self):
        original = fa_control.is_master_muted()
        new_state = fa_control.toggle_master_mute()
        assert isinstance(new_state, bool)
        assert new_state != original
        
        # Restore original state
        fa_control.set_master_mute(original)

    def test_set_master_mute(self):
        original = fa_control.is_master_muted()
        fa_control.set_master_mute(not original)
        assert fa_control.is_master_muted() == (not original)
        fa_control.set_master_mute(original)


class TestMicrophone:
    """Test microphone control"""
    def test_get_microphone_volume(self):
        volume = fa_control.get_microphone_volume()
        assert 0.0 <= volume <= 1.0

    def test_set_microphone_volume(self):
        original = fa_control.get_microphone_volume()
        
        fa_control.set_microphone_volume(0.7)
        assert fa_control.get_microphone_volume() == pytest.approx(0.7, rel=1e-2)
        
        # Restore original
        fa_control.set_microphone_volume(original)

    def test_set_microphone_volume_invalid(self):
        with pytest.raises(ValueError, match="Volume must be between 0.0 and 1.0"):
            fa_control.set_microphone_volume(2.0)

    def test_is_microphone_muted(self):
        muted = fa_control.is_microphone_muted()
        assert isinstance(muted, bool)

    def test_toggle_microphone_mute(self):
        original = fa_control.is_microphone_muted()
        new_state = fa_control.toggle_microphone_mute()
        assert isinstance(new_state, bool)
        
        # Restore original state
        fa_control.set_microphone_mute(original)

    def test_set_microphone_mute(self):
        original = fa_control.is_microphone_muted()
        fa_control.set_microphone_mute(not original)
        assert fa_control.is_microphone_muted() == (not original)
        fa_control.set_microphone_mute(original)


class TestAppVolume:
    """Test application-specific volume control"""
    def test_get_active_audio_apps(self):
        apps = fa_control.get_active_audio_apps()
        assert isinstance(apps, list)
        
        for app in apps:
            assert isinstance(app, fa_control.AppInfo)
            assert isinstance(app.pid, int)
            assert isinstance(app.name, str)
            assert 0.0 <= app.volume <= 1.0
            assert isinstance(app.muted, bool)

    def test_get_app_volume(self):
        apps = fa_control.get_active_audio_apps()
        if not apps:
            pytest.skip("No active audio applications")
        
        first_app = apps[0]
        volume = fa_control.get_app_volume(first_app.pid)
        assert 0.0 <= volume <= 1.0

    def test_set_app_volume(self):
        apps = fa_control.get_active_audio_apps()
        if not apps:
            pytest.skip("No active audio applications")
        
        first_app = apps[0]
        original = fa_control.get_app_volume(first_app.pid)
        
        fa_control.set_app_volume(first_app.pid, 0.4)
        assert fa_control.get_app_volume(first_app.pid) == pytest.approx(0.4, rel=1e-2)
        
        # Restore original
        fa_control.set_app_volume(first_app.pid, original)

    def test_set_app_volume_invalid(self):
        with pytest.raises(ValueError, match="Volume must be between 0.0 and 1.0"):
            fa_control.set_app_volume(9999, 1.5)

    def test_is_app_muted(self):
        apps = fa_control.get_active_audio_apps()
        if not apps:
            pytest.skip("No active audio applications")
        
        first_app = apps[0]
        muted = fa_control.is_app_muted(first_app.pid)
        assert isinstance(muted, bool)

    def test_set_app_mute(self):
        apps = fa_control.get_active_audio_apps()
        if not apps:
            pytest.skip("No active audio applications")
        
        first_app = apps[0]
        original = fa_control.is_app_muted(first_app.pid)
        
        fa_control.set_app_mute(first_app.pid, not original)
        assert fa_control.is_app_muted(first_app.pid) == (not original)
        
        fa_control.set_app_mute(first_app.pid, original)


class TestAppInfo:
    """Test AppInfo class"""
    def test_appinfo_creation(self):
        app = fa_control.AppInfo(
            pid=1234,
            name="Test App",
            volume=0.5,
            muted=False
        )
        assert app.pid == 1234
        assert app.name == "Test App"
        assert app.volume == pytest.approx(0.5, rel=1e-2)
        assert app.muted is False

    def test_appinfo_repr(self):
        app = fa_control.AppInfo(
            pid=5678,
            name="MyApp",
            volume=0.75,
            muted=True
        )
        repr_str = repr(app)
        assert "pid=5678" in repr_str
        assert "MyApp" in repr_str
        assert "0.75" in repr_str

    def test_appinfo_properties(self):
        app = fa_control.AppInfo(
            pid=9999,
            name="TestApp",
            volume=0.25,
            muted=True
        )
        assert app.pid == 9999
        assert app.name == "TestApp"
        assert app.volume == pytest.approx(0.25, rel=1e-2)
        assert app.muted is True
