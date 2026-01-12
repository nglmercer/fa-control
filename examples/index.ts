import { getActiveAudioApps, getPlatform, setAppVolume } from '../index';
async function main() {
    const audioapps = getActiveAudioApps();
    const platforms = getPlatform();
    
    //console.log("Audio apps found:", audioapps);
    
    // Test setting volume for the first app (index 0)
    const volumenValue = 0.5;
    if (audioapps.length > 0) {
        const app = audioapps[1];
        console.log(`Setting volume for ${app.name} (PID: ${app.pid}) to ${volumenValue/1}`);
        const result = setAppVolume(app.pid, volumenValue);
        console.log("Result:", result,app);
        
        // Get updated list to verify
        const updatedApps = getActiveAudioApps();
        console.log("Updated apps:", updatedApps);
    }
    
    return {
        audioapps,
        platforms
    }
}
main().then(function(r)
    {
        console.log(r)
    }
)
