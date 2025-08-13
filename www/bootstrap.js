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
let helpStartupTimer = 3000; // 3 seconds in milliseconds

function isMobileDevice() {
  return /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent) || 
         ('ontouchstart' in window) || 
         (navigator.maxTouchPoints > 0);
}

function setupHelpOverlay() {
  const helpOverlay = document.getElementById('help-overlay');
  const helpFlash = document.getElementById('help-flash');
  const touchControlsOverlay = document.getElementById('touch-controls-overlay');
  if (!helpOverlay || !helpFlash || !touchControlsOverlay) return;
  
  // Start with full help hidden and appropriate startup message visible
  helpOverlay.style.display = 'none';
  
  const isMobile = isMobileDevice();
  
  if (isMobile) {
    // On mobile, show touch control regions instead of text message
    helpFlash.style.display = 'none';
    touchControlsOverlay.style.display = 'block';
  } else {
    // On desktop, show text message
    helpFlash.style.display = 'block';
    touchControlsOverlay.style.display = 'none';
  }
  
  // Auto-fade startup message after timer
  setTimeout(() => {
    if (helpFlashVisible) {
      if (isMobile) {
        // Fade out touch controls overlay
        touchControlsOverlay.style.transition = 'opacity 1s ease-out';
        touchControlsOverlay.style.opacity = '0';
        setTimeout(() => {
          if (helpFlashVisible) {
            touchControlsOverlay.style.display = 'none';
            helpFlashVisible = false;
          }
        }, 1000);
      } else {
        // Fade out text message
        helpFlash.style.transition = 'opacity 1s ease-out';
        helpFlash.style.opacity = '0';
        setTimeout(() => {
          if (helpFlashVisible) {
            helpFlash.style.display = 'none';
            helpFlashVisible = false;
          }
        }, 1000);
      }
    }
  }, helpStartupTimer);
}

// Global functions for WASM to call
window.toggleHelp = function() {
  const helpOverlay = document.getElementById('help-overlay');
  const helpFlash = document.getElementById('help-flash');
  const touchControlsOverlay = document.getElementById('touch-controls-overlay');
  if (!helpOverlay || !helpFlash || !touchControlsOverlay) return;
  
  helpVisible = !helpVisible;
  helpOverlay.style.display = helpVisible ? 'block' : 'none';
  
  // Fade out startup message when user manually toggles help
  if (helpFlashVisible) {
    const isMobile = isMobileDevice();
    
    if (isMobile) {
      // Fade out touch controls overlay
      touchControlsOverlay.style.transition = 'opacity 0.3s ease-out';
      touchControlsOverlay.style.opacity = '0';
      setTimeout(() => {
        touchControlsOverlay.style.display = 'none';
      }, 300);
    } else {
      // Fade out text message
      helpFlash.style.transition = 'opacity 0.3s ease-out';
      helpFlash.style.opacity = '0';
      setTimeout(() => {
        helpFlash.style.display = 'none';
      }, 300);
    }
    
    helpFlashVisible = false;
  }
  
  // Disable auto-hide timer when manually toggled
  helpStartupTimer = 0;
};

window.setHelpVisible = function(visible) {
  const helpOverlay = document.getElementById('help-overlay');
  if (!helpOverlay) return;
  
  helpVisible = visible;
  helpOverlay.style.display = helpVisible ? 'block' : 'none';
  
  // Only hide startup message if help is being shown (user manually opened help)
  // Don't hide on automatic state sync when help is false
  if (visible && helpFlashVisible) {
    const helpFlash = document.getElementById('help-flash');
    const touchControlsOverlay = document.getElementById('touch-controls-overlay');
    const isMobile = isMobileDevice();
    
    if (isMobile && touchControlsOverlay) {
      // Fade out touch controls overlay
      touchControlsOverlay.style.transition = 'opacity 0.3s ease-out';
      touchControlsOverlay.style.opacity = '0';
      setTimeout(() => {
        touchControlsOverlay.style.display = 'none';
      }, 300);
    } else if (helpFlash) {
      // Fade out text message
      helpFlash.style.transition = 'opacity 0.3s ease-out';
      helpFlash.style.opacity = '0';
      setTimeout(() => {
        helpFlash.style.display = 'none';
      }, 300);
    }
    
    helpFlashVisible = false;
  }
};

window.updateDebugInfo = function(position, orientation, lastKey, fps, renderWidth, renderHeight, velocity) {
  document.getElementById('debug-position').textContent = 
    `Position: (${position[0].toFixed(2)}, ${position[1].toFixed(2)}, ${position[2].toFixed(2)})`;
  document.getElementById('debug-orientation').textContent = 
    `Orientation: Yaw ${orientation[0].toFixed(1)}°, Pitch ${orientation[1].toFixed(1)}°`;
  document.getElementById('debug-velocity').textContent = 
    `Velocity: (${velocity[0].toFixed(2)}, ${velocity[1].toFixed(2)}, ${velocity[2].toFixed(2)})`;
  document.getElementById('debug-lastkey').textContent = 
    `Last Key: ${lastKey || 'None'}`;
  document.getElementById('debug-fps').textContent = 
    `FPS: ${fps.toFixed(1)}`;
  document.getElementById('debug-resolution').textContent = 
    `Resolution: ${renderWidth.toFixed(0)}x${renderHeight.toFixed(0)}`;
};

window.updateFpsCounter = function(fps, visible) {
  const fpsCounter = document.getElementById('fps-counter');
  if (fpsCounter) {
    fpsCounter.textContent = `FPS: ${fps.toFixed(1)}`;
    fpsCounter.style.display = visible ? 'block' : 'none';
  }
};

// Debug control sliders - global reference to WASM module
let wasmModule = null;

function setupDebugControls() {
  // Helper function to blur slider after interaction to restore keyboard focus
  function setupSlider(sliderId, valueId, updateCallback, formatter) {
    const slider = document.getElementById(sliderId);
    const valueDisplay = document.getElementById(valueId);
    if (slider && valueDisplay) {
      slider.addEventListener('input', (e) => {
        const value = parseFloat(e.target.value);
        valueDisplay.textContent = formatter(value);
        if (wasmModule && updateCallback) {
          updateCallback(value);
        }
      });
      
      // Blur the slider when user stops interacting to restore keyboard focus to canvas
      slider.addEventListener('change', () => {
        slider.blur();
      });
      
      // Also blur on mouse leave to ensure keyboard focus returns
      slider.addEventListener('mouseleave', () => {
        slider.blur();
      });
    }
  }

  // FOV slider
  setupSlider('fov-slider', 'fov-value', 
    (value) => wasmModule?.set_debug_fov?.(value), 
    (value) => `${value}°`);

  // Mass slider
  setupSlider('mass-slider', 'mass-value', 
    (value) => wasmModule?.set_debug_mass?.(value), 
    (value) => value.toFixed(1));

  // Spin slider
  setupSlider('spin-slider', 'spin-value', 
    (value) => wasmModule?.set_debug_spin?.(value), 
    (value) => value.toFixed(1));

  // Ray steps slider
  setupSlider('ray-steps-slider', 'ray-steps-value', 
    (value) => wasmModule?.set_debug_ray_steps?.(value), 
    (value) => value.toFixed(0));
}

async function main() {
  // Set up canvas sizing before initializing WASM
  setupCanvas();
  setupHelpOverlay();
  
  // Wait for the wasm module to be compiled and initialized
  wasmModule = await init();

  // Set up debug controls now that WASM is loaded
  setupDebugControls();

  // Ensure canvas can receive keyboard events
  const canvas = document.getElementById('wasm-canvas');
  if (canvas) {
    canvas.tabIndex = 0; // Make canvas focusable
    canvas.focus(); // Give canvas initial focus
    
    // Restore focus to canvas when clicked
    canvas.addEventListener('click', () => {
      canvas.focus();
    });
  }

  // Now that initialization is complete, it's safe to call our function
  run();
}

main().catch(console.error);
