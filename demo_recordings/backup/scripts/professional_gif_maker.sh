#!/bin/bash

# Enterprise-grade GIF maker for SentientOS demos
# Uses a combination of tools to produce high-quality GIFs

CAST_FILE="$1"
OUTPUT_FILE="$2"

if [ -z "$CAST_FILE" ] || [ -z "$OUTPUT_FILE" ]; then
  echo "Usage: $0 input.cast output.gif"
  exit 1
fi

echo "====================================================="
echo "  SentientOS Enterprise GIF Generator"
echo "====================================================="
echo ""
echo "Processing: $CAST_FILE"
echo "Output: $OUTPUT_FILE"
echo ""

# Create a temporary directory
TEMP_DIR=$(mktemp -d)
HTML_FILE="$TEMP_DIR/player.html"

# Create an optimized HTML player for the recording
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
      background: #1a1b26;
      display: flex;
      justify-content: center;
      align-items: center;
      min-height: 100vh;
    }
    .player-container {
      width: 960px;
      border-radius: 6px;
      overflow: hidden;
      box-shadow: 0 5px 30px rgba(0,0,0,0.3);
    }
    .header {
      background: #292e42;
      color: white;
      padding: 12px 15px;
      font-family: sans-serif;
      font-weight: bold;
      border-bottom: 1px solid #414868;
      display: flex;
      align-items: center;
    }
    .logo {
      width: 24px;
      height: 24px;
      background: #7aa2f7;
      border-radius: 50%;
      margin-right: 10px;
    }
    .window-controls {
      display: flex;
      margin-right: 15px;
    }
    .control {
      width: 12px;
      height: 12px;
      border-radius: 50%;
      margin-right: 8px;
    }
    .red { background: #f7768e; }
    .yellow { background: #e0af68; }
    .green { background: #9ece6a; }
    .title { flex-grow: 1; }
  </style>
</head>
<body>
  <div class="player-container">
    <div class="header">
      <div class="window-controls">
        <div class="control red"></div>
        <div class="control yellow"></div>
        <div class="control green"></div>
      </div>
      <div class="logo"></div>
      <div class="title">SentientOS IoT Application Demo</div>
    </div>
    <asciinema-player
      src="$(realpath "$CAST_FILE")"
      cols="100"
      rows="30"
      preload
      autoplay
      font-size="medium"
      theme="monokai"
      speed="1.5"
      idle-time-limit="1"
    ></asciinema-player>
  </div>
  <script src="https://cdn.jsdelivr.net/npm/asciinema-player@3.0.1/dist/bundle/asciinema-player.js"></script>
</body>
</html>
EOL

echo "Created optimized HTML player"

# Check if we have puppeteer for headless capture
if ! npm list -g puppeteer > /dev/null 2>&1; then
  echo "Installing puppeteer for headless capture..."
  npm install -g puppeteer
fi

# Create a simple capture script
CAPTURE_SCRIPT="$TEMP_DIR/capture.js"

cat > "$CAPTURE_SCRIPT" << EOL
const puppeteer = require('puppeteer');
const fs = require('fs');
const path = require('path');

(async () => {
  try {
    console.log('Launching browser...');
    const browser = await puppeteer.launch({
      headless: 'new',
      args: ['--no-sandbox', '--disable-setuid-sandbox']
    });
    
    const page = await browser.newPage();
    await page.setViewport({ width: 1000, height: 700 });
    
    console.log('Loading player...');
    await page.goto('file://$HTML_FILE', { waitUntil: 'networkidle2' });
    
    // Wait for player to initialize
    await page.waitForSelector('asciinema-player');
    console.log('Player loaded, waiting for playback to complete...');
    
    // Get the recording duration
    const duration = await page.evaluate(() => {
      const player = document.querySelector('asciinema-player');
      if (player && player.getDuration) {
        return player.getDuration();
      }
      return 120; // Default to 2 minutes
    });
    
    console.log(\`Recording duration: \${duration}s\`);
    
    // Create output directory
    const outputDir = '$TEMP_DIR/frames';
    if (!fs.existsSync(outputDir)) {
      fs.mkdirSync(outputDir, { recursive: true });
    }
    
    // Take screenshots at regular intervals
    const fps = 10;
    const frameCount = Math.ceil(duration * fps);
    const frameInterval = 1000 / fps;
    
    console.log(\`Capturing \${frameCount} frames at \${fps} fps...\`);
    
    for (let i = 0; i < frameCount; i++) {
      const time = i / fps;
      
      // Set playback position
      await page.evaluate((t) => {
        const player = document.querySelector('asciinema-player');
        if (player && player.setCurrentTime) {
          player.setCurrentTime(t);
        }
      }, time);
      
      // Wait a bit for rendering
      await new Promise(r => setTimeout(r, 50));
      
      // Take screenshot
      const frameFile = path.join(outputDir, \`frame_\${String(i).padStart(5, '0')}.png\`);
      await page.screenshot({ path: frameFile });
      
      if (i % 10 === 0) {
        console.log(\`Captured frame \${i+1}/\${frameCount}\`);
      }
    }
    
    console.log('Capture complete!');
    await browser.close();
    process.exit(0);
  } catch (err) {
    console.error('Error:', err);
    process.exit(1);
  }
})();
EOL

echo "Starting headless capture process..."
node "$CAPTURE_SCRIPT"

# Create GIF from frames using ImageMagick
if command -v convert > /dev/null; then
  echo "Creating GIF from frames..."
  convert -delay 10 -loop 0 "$TEMP_DIR/frames/frame_*.png" "$OUTPUT_FILE"
  
  # Optimize if gifsicle is available
  if command -v gifsicle > /dev/null; then
    echo "Optimizing GIF..."
    gifsicle -O3 "$OUTPUT_FILE" -o "${OUTPUT_FILE}.opt" && mv "${OUTPUT_FILE}.opt" "$OUTPUT_FILE"
  fi
  
  echo "GIF created successfully: $OUTPUT_FILE"
else
  echo "Error: ImageMagick not found. Cannot create GIF."
  echo "Please install ImageMagick and try again."
  exit 1
fi

# Clean up
echo "Cleaning temporary files..."
rm -rf "$TEMP_DIR"

echo ""
echo "Enterprise GIF generation complete!"
echo "Output: $OUTPUT_FILE"
