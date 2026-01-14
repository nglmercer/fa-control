import {
  getActiveAudioApps,
  setAppVolume,
  setMicrophoneVolume,
  setMasterVolume,
  setMasterMute,
  toggleMasterMute,
  setMicrophoneMute,
  toggleMicrophoneMute,
  setAppMute,
  getMasterVolume,
  getMicrophoneVolume,
  isAppMuted
} from '../index';

export async function demonstrateSetters() {
  console.log('=== SETTERS DEMONSTRATION ===');
  
  try {
    const volumeToSet = 0.5;
    console.log({ action: 'Setting Master Volume', volumeToSet });
    setMasterVolume(volumeToSet);
    const newMasterVol = getMasterVolume();
    console.log({ newMasterVol });

    const masterMutedNow = toggleMasterMute();
    console.log({ action: 'Toggling Master Mute', masterMutedNow });
    
    console.log({ action: 'Explicitly Unmuting Master' });
    setMasterMute(false);
  } catch (error) {
    console.log({ error, context: 'setting master audio' });
  }

  try {
    const micVolumeToSet = 0.8;
    console.log({ action: '\nSetting Microphone Volume', micVolumeToSet });
    setMicrophoneVolume(micVolumeToSet);
    const newMicVol = getMicrophoneVolume();
    console.log({ newMicVol });

    const micMutedNow = toggleMicrophoneMute();
    console.log({ action: 'Toggling Microphone Mute', micMutedNow });
    
    console.log({ action: 'Explicitly Unmuting Microphone' });
    setMicrophoneMute(false);
  } catch (error) {
    console.log({ error, context: 'setting microphone audio' });
  }

  try {
    const apps = getActiveAudioApps();
    if (apps.length > 0) {
      const targetApp = apps[0];
      const { pid, name } = targetApp;
      console.log({ action: '\nModifying App Volume', targetApp: { pid, name } });
      
      const appVolumeToSet = 0.3;
      console.log({ action: 'Setting App Volume', appVolumeToSet });
      setAppVolume(pid, appVolumeToSet);
      
      console.log({ action: 'Muting App' });
      setAppMute(pid, true);
      const appMuted = isAppMuted(pid);
      console.log({ appMuted });
      
      console.log({ action: 'Unmuting App' });
      setAppMute(pid, false);
    } else {
      console.log({ status: 'No active audio apps found for testing app setters' });
    }
  } catch (error) {
    console.log({ error, context: 'setting app audio' });
  }
  console.log('=============================\n');
}
