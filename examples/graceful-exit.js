/**
 * This script runs for 5 seconds and exits.
 * If it receives SIGTERM, it performs a 2-second cleanup before exiting.
 */

const RUN_TIME_MS = 3000;
const CLEANUP_TIME_MS = 2000;

console.log(`[Task] Started. I will run for ${RUN_TIME_MS / 1000} seconds or until I receive SIGTERM.`);

let isCleaningUp = false;

async function performCleanup() {
    if (isCleaningUp) return;
    isCleaningUp = true;
    console.log("[Task] Received SIGTERM. Starting cleanup...");
    console.log(`[Task] Cleanup will take ${CLEANUP_TIME_MS / 1000} seconds...`);

    await new Promise(resolve => setTimeout(resolve, CLEANUP_TIME_MS));

    console.log("[Task] Cleanup complete. Exiting.");
    process.exit(0);
}

// Listen for SIGTERM
process.on("SIGTERM", async () => {
    console.log("did get sigterm");
    await performCleanup();
});

// Main execution
const startTime = Date.now();
const interval = setInterval(() => {
    const elapsed = Date.now() - startTime;
    if (elapsed >= RUN_TIME_MS && !isCleaningUp) {
        console.log("[Task] 3 seconds elapsed. Exiting naturally.");
        clearInterval(interval);
        process.exit(0);
    } else if (!isCleaningUp) {
        console.log(`[Task] Working... (${Math.round(elapsed / 1000)}s)`);
    }
}, 1000);
