#!/bin/bash

# Enterprise-grade Asciinema to GIF converter
# Creates beautiful, professional GIFs from asciinema recordings

INPUT_FILE="$1"
OUTPUT_FILE="$2"
TEMP_DIR="temp_gif_frames"
FPS=12
DELAY=$(echo "scale=2; 100/$FPS" | bc)

if [ -z "$INPUT_FILE" ] || [ -z "$OUTPUT_FILE" ]; then
  echo "Usage: $0 input.cast output.gif"
  exit 1
fi

echo "========================================"
echo "  Enterprise GIF Converter for SentientOS"
echo "========================================"
echo ""
echo "Converting: $INPUT_FILE"
echo "Output: $OUTPUT_FILE"
echo "Frame rate: $FPS fps"
echo ""

# Create temporary directory for frames
mkdir -p "$TEMP_DIR"

# Extract metadata from cast file
WIDTH=$(head -n 1 "$INPUT_FILE" | jq '.width')
HEIGHT=$(head -n 1 "$INPUT_FILE" | jq '.height')

# Default to reasonable values if not specified
WIDTH=${WIDTH:-80}
HEIGHT=${HEIGHT:-24}

# Convert to pixel dimensions with some padding
PIX_WIDTH=$(( WIDTH * 10 + 40 ))
PIX_HEIGHT=$(( HEIGHT * 20 + 60 ))

echo "Terminal dimensions: ${WIDTH}x${HEIGHT} (${PIX_WIDTH}x${PIX_HEIGHT} pixels)"
echo ""

# Create HTML player
HTML_FILE="$TEMP_DIR/player.html"

cat > "$HTML_FILE" << EOL
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>SentientOS Demo</title>
  <link rel="stylesheet" type="text/css" href="https://cdn.jsdelivr.net/npm/asciinema-player@3.0.1/dist/bundle/asciinema-player.css">
  <style>
    body {
      margin: 0;
      padding: 0;
      background: #121212;
    }
    .terminal {
      padding: 10px;
    }
    .player {
      width: ${PIX_WIDTH}px;
      height: ${PIX_HEIGHT}px;
      margin: 0 auto;
      overflow: hidden;
    }
  </style>
</head>
<body>
  <div class="player">
    <asciinema-player
      src="$(realpath "$INPUT_FILE")"
      cols="${WIDTH}"
      rows="${HEIGHT}"
      preload="true"
      autoplay="true"
      font-size="medium"
      theme="monokai"
      speed="1.0"
      idle-time-limit="1.0"
    ></asciinema-player>
  </div>
  <script src="https://cdn.jsdelivr.net/npm/asciinema-player@3.0.1/dist/bundle/asciinema-player.js"></script>
</body>
</html>
EOL

echo "Created HTML player for rendering"
echo "Generating frames with Chrome/Chromium..."

# Try to find Chrome or Chromium executable
if command -v google-chrome > /dev/null; then
  CHROME="google-chrome"
elif command -v chromium-browser > /dev/null; then
  CHROME="chromium-browser"
elif command -v chromium > /dev/null; then
  CHROME="chromium"
else
  echo "Error: Chrome or Chromium browser not found"
  exit 1
fi

echo "Using browser: $CHROME"

# Create a simple Node.js script for controlling Chrome
NODE_SCRIPT="$TEMP_DIR/capture.js"

cat > "$NODE_SCRIPT" << EOL
const puppeteer = require('puppeteer-core');
const fs = require('fs');
const path = require('path');

(async () => {
  // Get cast file duration
  const castContent = fs.readFileSync('$INPUT_FILE', 'utf8');
  const lines = castContent.split('\n').filter(line => line.trim());
  
  // Calculate duration from last timestamp
  const lastEvent = JSON.parse(lines[lines.length - 1]);
  const duration = lastEvent[0] + 3; // Add 3 seconds buffer
  
  console.log('Estimated recording duration:', duration.toFixed(2), 'seconds');
  
  // Launch browser with path to Chrome/Chromium
  const browser = await puppeteer.launch({
    executablePath: '$CHROME',
    headless: 'new',
    args: ['--no-sandbox', '--disable-web-security']
  });
  
  const page = await browser.newPage();
  await page.setViewport({ width: $PIX_WIDTH, height: $PIX_HEIGHT });
  
  // Load the player HTML
  await page.goto('file://${PWD}/$HTML_FILE', { waitUntil: 'networkidle2' });
  console.log('Player loaded, beginning capture...');
  
  // Wait for player to initialize
  await page.waitForSelector('asciinema-player', { timeout: 5000 });
  await new Promise(r => setTimeout(r, 2000));
  
  // Calculate frames
  const fps = $FPS;
  const totalFrames = Math.ceil(duration * fps);
  const frameInterval = 1000 / fps;
  
  console.log('Capturing', totalFrames, 'frames at', fps, 'fps');
  
  // Function to capture a frame
  const captureFrame = async (frameNum, time) => {
    // Set player current time
    await page.evaluate((t) => {
      const player = document.querySelector('asciinema-player');
      if (player && player.setCurrentTime) {
        player.setCurrentTime(t);
      }
    }, time);
    
    // Wait a bit for rendering
    await new Promise(r => setTimeout(r, 50));
    
    // Take screenshot
    const framePath = path.join('$TEMP_DIR', 'frame_' + String(frameNum).padStart(5, '0') + '.png');
    await page.screenshot({ path: framePath });
    console.log('Captured frame', frameNum + 1, '/', totalFrames, 'at time', time.toFixed(2) + 's');
  };
  
  // Capture frames
  for (let i = 0; i < totalFrames; i++) {
    const time = i / fps;
    await captureFrame(i, time);
  }
  
  await browser.close();
  console.log('Capture complete');
})().catch(err => {
  console.error('Error:', err);
  process.exit(1);
});
EOL

# Install puppeteer-core if not already installed
if ! npm list puppeteer-core > /dev/null 2>&1; then
  echo "Installing puppeteer-core..."
  npm install puppeteer-core
fi

# Run the capture script
echo "Capturing frames with Node.js and Puppeteer..."
node "$NODE_SCRIPT"

# Check if ImageMagick is available for GIF creation
if ! command -v convert > /dev/null; then
  echo "Error: ImageMagick not found. Cannot create GIF."
  echo "Please install ImageMagick and try again."
  exit 1
fi

# Create GIF from frames
echo "Creating GIF from captured frames..."
convert -delay $DELAY -loop 0 "$TEMP_DIR/frame_*.png" "$OUTPUT_FILE"

# Optimize GIF if gifsicle is available
if command -v gifsicle > /dev/null; then
  echo "Optimizing GIF with gifsicle..."
  gifsicle -O3 "$OUTPUT_FILE" -o "$OUTPUT_FILE.opt" && mv "$OUTPUT_FILE.opt" "$OUTPUT_FILE"
fi

echo ""
echo "GIF creation complete!"
echo "Output saved to: $OUTPUT_FILE"
echo ""
echo "Cleaning up temporary files..."

# Clean up
rm -rf "$TEMP_DIR"

echo "Done!"
