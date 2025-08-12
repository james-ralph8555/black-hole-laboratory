// Import the initializer and the function you want to run
import init, { run } from './pkg/index.js';

function setupCanvas() {
  const canvas = document.getElementById('wasm-canvas');
  if (!canvas) return;
  
  function resizeCanvas() {
    const displayWidth = window.innerWidth;
    const displayHeight = window.innerHeight;
    
    // Fixed internal resolution for consistent rendering performance
    const FIXED_WIDTH = 1280;
    const FIXED_HEIGHT = 720;
    
    // Set the CSS size to fill the viewport
    canvas.style.width = displayWidth + 'px';
    canvas.style.height = displayHeight + 'px';
    
    // Set fixed internal resolution
    canvas.width = FIXED_WIDTH;
    canvas.height = FIXED_HEIGHT;
    
    // Use CSS image-rendering for better scaling
    canvas.style.imageRendering = 'auto';
    
    console.log(`Canvas: CSS ${displayWidth}x${displayHeight}, Fixed render ${FIXED_WIDTH}x${FIXED_HEIGHT}`);
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
let helpVisible = true;
let helpStartupTimer = 5000; // 5 seconds in milliseconds

function setupHelpOverlay() {
  const helpOverlay = document.getElementById('help-overlay');
  if (!helpOverlay) return;
  
  // Show help initially
  helpOverlay.style.display = 'block';
  
  // Auto-hide after startup timer
  setTimeout(() => {
    if (helpStartupTimer > 0) {
      helpOverlay.style.display = 'none';
      helpVisible = false;
    }
  }, helpStartupTimer);
}

// Global functions for WASM to call
window.toggleHelp = function() {
  const helpOverlay = document.getElementById('help-overlay');
  if (!helpOverlay) return;
  
  helpVisible = !helpVisible;
  helpOverlay.style.display = helpVisible ? 'block' : 'none';
  
  // Disable auto-hide when manually toggled
  helpStartupTimer = 0;
};

window.setHelpVisible = function(visible) {
  const helpOverlay = document.getElementById('help-overlay');
  if (!helpOverlay) return;
  
  helpVisible = visible;
  helpOverlay.style.display = helpVisible ? 'block' : 'none';
  
  // Disable auto-hide when manually set
  helpStartupTimer = 0;
};

window.updateDebugInfo = function(position, orientation, lastKey, fps) {
  document.getElementById('debug-position').textContent = 
    `Position: (${position[0].toFixed(2)}, ${position[1].toFixed(2)}, ${position[2].toFixed(2)})`;
  document.getElementById('debug-orientation').textContent = 
    `Orientation: Yaw ${orientation[0].toFixed(1)}°, Pitch ${orientation[1].toFixed(1)}°`;
  document.getElementById('debug-lastkey').textContent = 
    `Last Key: ${lastKey || 'None'}`;
  document.getElementById('debug-fps').textContent = 
    `FPS: ${fps.toFixed(1)}`;
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
