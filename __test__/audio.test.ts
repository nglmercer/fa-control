import { describe, test, expect, beforeAll } from 'bun:test';
import * as faControl from '../index';

describe('fa-control', () => {
  let platform: string;

  beforeAll(() => {
    platform = faControl.getPlatform();
    console.log(`Testing on platform: ${platform}`);
  });

  describe('Platform Detection', () => {
    test('should return a valid platform string', () => {
      expect(platform).toBeOneOf(['windows', 'linux', 'unsupported']);
    });
  });

  describe('Per-Application Volume Control', () => {
    test('should get list of active audio apps', () => {
      const apps = faControl.getActiveAudioApps();
      expect(Array.isArray(apps)).toBe(true);
      
      if (apps.length > 0) {
        const app = apps[0];
        expect(app).toHaveProperty('pid');
        expect(app).toHaveProperty('name');
        expect(app).toHaveProperty('volume');
        expect(app).toHaveProperty('muted');
        expect(typeof app.pid).toBe('number');
        expect(typeof app.name).toBe('string');
        expect(typeof app.volume).toBe('number');
        expect(typeof app.muted).toBe('boolean');
        expect(app.volume).toBeGreaterThanOrEqual(0);
        expect(app.volume).toBeLessThanOrEqual(1);
      }
    });

    test('should handle invalid PID for app volume', () => {
      expect(() => faControl.getAppVolume(99999999)).toThrow();
    });

    test('should handle invalid PID for app mute', () => {
      expect(() => faControl.isAppMuted(99999999)).toThrow();
    });

    test('should reject invalid volume values', () => {
      const apps = faControl.getActiveAudioApps();
      if (apps.length > 0) {
        const pid = apps[0].pid;
        
        expect(() => faControl.setAppVolume(pid, -0.1)).toThrow();
        expect(() => faControl.setAppVolume(pid, 1.1)).toThrow();
        expect(() => faControl.setAppVolume(pid, 2.0)).toThrow();
      }
    });

    test('should accept valid volume values', () => {
      const apps = faControl.getActiveAudioApps();
      if (apps.length > 0) {
        const pid = apps[0].pid;
        
        expect(() => faControl.setAppVolume(pid, 0.0)).not.toThrow();
        expect(() => faControl.setAppVolume(pid, 0.5)).not.toThrow();
        expect(() => faControl.setAppVolume(pid, 1.0)).not.toThrow();
      }
    });

    test('should get and set app volume', () => {
      const apps = faControl.getActiveAudioApps();
      if (apps.length > 0) {
        const pid = apps[0].pid;
        const originalVolume = faControl.getAppVolume(pid);
        expect(typeof originalVolume).toBe('number');
        
        // Set volume to 0.5
        faControl.setAppVolume(pid, 0.5);
        const newVolume = faControl.getAppVolume(pid);
        expect(newVolume).toBeCloseTo(0.5, 1);
        
        // Restore original volume
        faControl.setAppVolume(pid, originalVolume);
      }
    });

    test('should get and set app mute state', () => {
      const apps = faControl.getActiveAudioApps();
      if (apps.length > 0) {
        const pid = apps[0].pid;
        const originalMuted = faControl.isAppMuted(pid);
        expect(typeof originalMuted).toBe('boolean');
        
        // Toggle mute
        faControl.setAppMute(pid, true);
        expect(faControl.isAppMuted(pid)).toBe(true);
        
        // Restore
        faControl.setAppMute(pid, false);
        expect(faControl.isAppMuted(pid)).toBe(false);
      }
    });
  });

  describe('Master Volume Control', () => {
    test('should get master volume', () => {
      const volume = faControl.getMasterVolume();
      expect(typeof volume).toBe('number');
      expect(volume).toBeGreaterThanOrEqual(0);
      expect(volume).toBeLessThanOrEqual(1);
    });

    test('should set master volume', () => {
      const originalVolume = faControl.getMasterVolume();
      
      faControl.setMasterVolume(0.5);
      expect(faControl.getMasterVolume()).toBeCloseTo(0.5, 1);
      
      faControl.setMasterVolume(originalVolume);
    });

    test('should reject invalid master volume values', () => {
      expect(() => faControl.setMasterVolume(-0.1)).toThrow();
      expect(() => faControl.setMasterVolume(1.1)).toThrow();
    });

    test('should get master mute state', () => {
      const muted = faControl.isMasterMuted();
      expect(typeof muted).toBe('boolean');
    });

    test('should set master mute state', () => {
      const originalMuted = faControl.isMasterMuted();
      
      faControl.setMasterMute(true);
      expect(faControl.isMasterMuted()).toBe(true);
      
      faControl.setMasterMute(false);
      expect(faControl.isMasterMuted()).toBe(false);
      
      // Restore
      faControl.setMasterMute(originalMuted);
    });

    test('should toggle master mute', () => {
      const originalMuted = faControl.isMasterMuted();
      const newMuted = faControl.toggleMasterMute();
      expect(newMuted).toBe(!originalMuted);
      
      // Toggle back
      faControl.toggleMasterMute();
    });
  });

  describe('Performance', () => {
    test('getActiveAudioApps should complete within reasonable time', () => {
      const start = Date.now();
      const apps = faControl.getActiveAudioApps();
      const duration = Date.now() - start;
      
      expect(Array.isArray(apps)).toBe(true);
      expect(duration).toBeLessThan(5000); // Should complete in less than 5 seconds
    });
  });
});
