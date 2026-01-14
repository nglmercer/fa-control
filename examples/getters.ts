import {
  getActiveAudioApps,
  getMasterVolume,
  isMasterMuted,
  getMicrophoneVolume,
  isMicrophoneMuted,
  getAppVolume,
  isAppMuted,
  getPlatform
} from '../index';

export async function demonstrateGetters() {
  console.log('=== GETTERS DEMONSTRATION ===');
  
  const platform = getPlatform();
  console.log({ platform });

  try {
    const masterVol = getMasterVolume();
    const masterMuted = isMasterMuted();
    console.log({ masterVol, masterMuted });
  } catch (error) {
    console.log({ error, context: 'master volume info' });
  }

  try {
    const micVol = getMicrophoneVolume();
    const micMuted = isMicrophoneMuted();
    console.log({ micVol, micMuted });
  } catch (error) {
    console.log({ error, context: 'microphone info' });
  }

  try {
    const apps = getActiveAudioApps();
    console.log({ appsCount: apps.length });
    
    apps.forEach(app => {
      const { pid, name, volume, muted } = app;
      console.log({ pid, name, volume, muted });
      
      // Individual app getters verification
      const individualVol = getAppVolume(pid);
      const individualMute = isAppMuted(pid);
      console.log({ verification: { pid, individualVol, individualMute } });
    });
  } catch (error) {
    console.log({ error, context: 'active audio apps' });
  }
  console.log('=============================\n');
}
