const puppeteer = require('puppeteer');
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

// Configuration
const castFile = process.argv[2];
const outputGif = process.argv[3];
const frameRate = process.argv[4] || 10;
const speed = process.argv[5] || 1.0;

if (!castFile || !outputGif) {
  console.error('Usage: node enterprise_gif_generator.js <cast_file> <output_gif> [frameRate] [speed]');
  process.exit(1);
}

// Create a temporary HTML file with asciinema-player
const tempHtml = path.join(__dirname, 'temp_player.html');
const html = `
<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <title>SentientOS Demo</title>
  <link rel="stylesheet" type="text/css" href="https://cdn.jsdelivr.net/npm/asciinema-player@3.0.1/dist/bundle/asciinema-player.css">
  <style>
    body { margin: 0; padding: 0; background: #121212; }
    .player-container {
      width: 1024px;
      height: 600px;
      margin: 0;
      padding: 0;
      display: flex;
      align-items: center;
      justify-content: center;
    }
  </style>
</head>
<body>
  <div class="player-container">
    <asciinema-player
      src="${path.resolve(castFile)}"
      cols="100"
      rows="30"
      preload="true"
      autoplay="true"
      font-size="medium"
      theme="monokai"
      speed="${speed}"
      idle-time-limit="1"
    ></asciinema-player>
  </div>
  <script src="https://cdn.jsdelivr.net/npm/asciinema-player@3.0.1/dist/bundle/asciinema-player.js"></script>
</body>
</html>
`;

fs.writeFileSync(tempHtml, html);

// Function to ensure directory exists
function ensureDirectoryExists(dir) {
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
}

// Directory for temporary screenshots
const screenshotsDir = path.join(__dirname, 'temp_screenshots');
ensureDirectoryExists(screenshotsDir);

// Clean existing screenshots
fs.readdirSync(screenshotsDir)
  .filter(file => file.endsWith('.png'))
  .forEach(file => fs.unlinkSync(path.join(screenshotsDir, file)));

(async () => {
  console.log('Launching browser to record asciinema playback...');
  const browser = await puppeteer.launch({
    headless: 'new',
    args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-web-security']
  });
  
  const page = await browser.newPage();
  await page.setViewport({ width: 1024, height: 600 });
  
  // Load the HTML file
  const fileUrl = 'file://' + path.resolve(tempHtml);
  await page.goto(fileUrl, { waitUntil: 'networkidle2' });
  
  // Wait for player to load
  await page.waitForSelector('asciinema-player', { timeout: 5000 });
  console.log('Player loaded, beginning capture...');
  
  // Get recording duration from the player
  const duration = await page.evaluate(() => {
    const player = document.querySelector('asciinema-player');
    return new Promise(resolve => {
      // Give player time to initialize
      setTimeout(() => {
        if (player && player.getDuration) {
          resolve(player.getDuration());
        } else {
          resolve(120); // Default to 2 minutes if can't detect
        }
      }, 2000);
    });
  });
  
  console.log(`Recording duration: ${duration}s`);
  
  // Calculate total frames based on frameRate
  const totalFrames = Math.ceil(duration * frameRate);
  const frameInterval = 1000 / frameRate;
  
  // Take screenshots at regular intervals
  let currentTime = 0;
  let frameCount = 0;
  
  while (frameCount < totalFrames) {
    const framePath = path.join(screenshotsDir, `frame_${String(frameCount).padStart(5, '0')}.png`);
    
    // Set player current time
    await page.evaluate((time) => {
      const player = document.querySelector('asciinema-player');
      if (player && player.setCurrentTime) {
        player.setCurrentTime(time);
      }
    }, currentTime);
    
    // Wait a bit for rendering
    await new Promise(r => setTimeout(r, 100));
    
    // Take screenshot
    await page.screenshot({ path: framePath });
    console.log(`Captured frame ${frameCount+1}/${totalFrames} at time ${currentTime.toFixed(2)}s`);
    
    // Increment
    frameCount++;
    currentTime += 1 / frameRate;
  }
  
  await browser.close();
  
  // Convert screenshots to GIF using ImageMagick
  console.log('Converting frames to GIF...');
  try {
    execSync(`convert -delay ${100/frameRate} -loop 0 ${path.join(screenshotsDir, 'frame_*.png')} ${outputGif}`);
    console.log(`GIF created successfully: ${outputGif}`);
    
    // Optimize the GIF
    console.log('Optimizing GIF...');
    execSync(`gifsicle -O3 ${outputGif} -o ${outputGif}`);
    console.log('GIF optimization complete');
  } catch (error) {
    console.error('Error creating GIF:', error.message);
    console.log('Please ensure ImageMagick and gifsicle are installed.');
  }
  
  // Clean up
  console.log('Cleaning up temporary files...');
  fs.unlinkSync(tempHtml);
  fs.readdirSync(screenshotsDir)
    .forEach(file => fs.unlinkSync(path.join(screenshotsDir, file)));
  fs.rmdirSync(screenshotsDir);
  
  console.log('Enterprise-grade GIF generation complete!');
})();
