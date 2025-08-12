// Import the initializer and the function you want to run
import init, { run } from './pkg/index.js';

function setupCanvas() {
  const canvas = document.getElementById('wasm-canvas');
  if (!canvas) return;
  
  function resizeCanvas() {
    const displayWidth = window.innerWidth;
    const displayHeight = window.innerHeight;
    
    // Set the CSS size to fill the viewport
    canvas.style.width = displayWidth + 'px';
    canvas.style.height = displayHeight + 'px';
    
    // Set canvas resolution to match viewport (with device pixel ratio consideration)
    const devicePixelRatio = window.devicePixelRatio || 1;
    const renderWidth = Math.floor(displayWidth * devicePixelRatio);
    const renderHeight = Math.floor(displayHeight * devicePixelRatio);
    
    canvas.width = renderWidth;
    canvas.height = renderHeight;
    
    console.log(`Canvas: CSS ${displayWidth}x${displayHeight}, Render ${renderWidth}x${renderHeight}, DPR ${devicePixelRatio}`);
  }
  
  // Initial resize
  resizeCanvas();
  
  // Listen for window resize events
  window.addEventListener('resize', resizeCanvas);
  
  // Also listen for orientation change on mobile
  window.addEventListener('orientationchange', () => {
    setTimeout(resizeCanvas, 100);
  });
}

// Help overlay management
let helpVisible = false; // Start with help hidden
let helpFlashVisible = true; // Start with flash message visible
let helpStartupTimer = 5000; // 5 seconds in milliseconds

function setupHelpOverlay() {
  const helpOverlay = document.getElementById('help-overlay');
  const helpFlash = document.getElementById('help-flash');
  if (!helpOverlay || !helpFlash) return;
  
  // Start with full help hidden and flash message visible
  helpOverlay.style.display = 'none';
  helpFlash.style.display = 'block';
  
  // Auto-hide flash message after startup timer
  setTimeout(() => {
    if (helpFlashVisible) {
      helpFlash.style.display = 'none';
      helpFlashVisible = false;
    }
  }, helpStartupTimer);
}

// Global functions for WASM to call
window.toggleHelp = function() {
  const helpOverlay = document.getElementById('help-overlay');
  const helpFlash = document.getElementById('help-flash');
  if (!helpOverlay || !helpFlash) return;
  
  helpVisible = !helpVisible;
  helpOverlay.style.display = helpVisible ? 'block' : 'none';
  
  // Hide flash message when user manually toggles help
  if (helpFlashVisible) {
    helpFlash.style.display = 'none';
    helpFlashVisible = false;
  }
  
  // Disable auto-hide timer when manually toggled
  helpStartupTimer = 0;
};

window.setHelpVisible = function(visible) {
  const helpOverlay = document.getElementById('help-overlay');
  const helpFlash = document.getElementById('help-flash');
  if (!helpOverlay || !helpFlash) return;
  
  helpVisible = visible;
  helpOverlay.style.display = helpVisible ? 'block' : 'none';
  
  // Hide flash message when help state is manually set
  if (helpFlashVisible) {
    helpFlash.style.display = 'none';
    helpFlashVisible = false;
  }
  
  // Disable auto-hide timer when manually set
  helpStartupTimer = 0;
};

window.updateDebugInfo = function(position, orientation, lastKey, fps, renderWidth, renderHeight) {
  document.getElementById('debug-position').textContent = 
    `Position: (${position[0].toFixed(2)}, ${position[1].toFixed(2)}, ${position[2].toFixed(2)})`;
  document.getElementById('debug-orientation').textContent = 
    `Orientation: Yaw ${orientation[0].toFixed(1)}°, Pitch ${orientation[1].toFixed(1)}°`;
  document.getElementById('debug-lastkey').textContent = 
    `Last Key: ${lastKey || 'None'}`;
  document.getElementById('debug-fps').textContent = 
    `FPS: ${fps.toFixed(1)}`;
  document.getElementById('debug-resolution').textContent = 
    `Resolution: ${renderWidth.toFixed(0)}x${renderHeight.toFixed(0)}`;
};

async function main() {
  // Set up canvas sizing before initializing WASM
  setupCanvas();
  setupHelpOverlay();
  
  // Wait for the wasm module to be compiled and initialized
  await init();

  // Now that initialization is complete, it's safe to call our function
  run();
}

main().catch(console.error);
