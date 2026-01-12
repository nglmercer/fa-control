import { getActiveAudioApps, getPlatform, setAppVolume } from '../index';

function main() {
    const audioapps = getActiveAudioApps();
    const platforms = getPlatform();
    
    console.log("Audio apps found:", audioapps);
    console.log("Platform:", platforms);
    
    // Test setting volume for the first app (index 0)
    const volumenValue = 0.1;
    if (audioapps.length > 0) {
        const app = audioapps[1];
        if (!app) return;
        console.log(`Setting volume for ${app.name} (PID: ${app.pid}) to ${volumenValue}`);
        const result = setAppVolume(app.pid, volumenValue);
        console.log("Result:", result, app);
        
        // Get updated list to verify
        const updatedApps = getActiveAudioApps();
        console.log("Updated apps:", updatedApps);
    } else {
        console.log("No active audio apps found");
    }
    
    return {
        audioapps,
        platforms
    };
}

main();
