#!/usr/bin/env python3

import json
import os
import sys
import shutil
from PIL import Image, ImageDraw, ImageFont
import subprocess
import tempfile

def create_terminal_image(text, width, height, font_size=18, bg_color=(25, 25, 35), fg_color=(220, 220, 220)):
    """Create a terminal-like image with text."""
    img = Image.new('RGB', (width, height), bg_color)
    draw = ImageDraw.Draw(img)
    
    try:
        # Try to load a monospace font
        font = ImageFont.truetype("DejaVuSansMono.ttf", font_size)
    except IOError:
        # Fall back to default
        font = ImageFont.load_default()
    
    # Calculate line height based on font
    line_height = font_size + 2
    
    # Split text into lines and draw each line
    y_position = 10
    lines = text.split('\n')
    for line in lines:
        draw.text((10, y_position), line, font=font, fill=fg_color)
        y_position += line_height
    
    return img

def asciinema_to_gif(cast_file, output_gif, fps=5, width=1000, height=600):
    """Convert an asciinema .cast file to a GIF."""
    # Read the cast file
    with open(cast_file, 'r') as f:
        # Parse header (first line)
        header = json.loads(f.readline())
        
        # Set terminal dimensions
        terminal_width = header.get('width', 80) * 8  # Approximate character width
        terminal_height = header.get('height', 24) * 16  # Approximate character height
        
        if terminal_width > width:
            terminal_width = width
        if terminal_height > height:
            terminal_height = height
        
        # Initialize terminal state
        terminal_content = ""
        frames = []
        frame_duration = 1000 // fps  # in milliseconds
        
        # Create temp directory for frames
        with tempfile.TemporaryDirectory() as tmpdirname:
            frame_count = 0
            last_time = 0
            
            # Read events
            for line in f:
                event = json.loads(line)
                time_diff = event[0] - last_time
                last_time = event[0]
                
                # Only process output events (type 'o')
                if event[1] == 'o':
                    output = event[2]
                    
                    # Simple terminal emulation (just append output)
                    terminal_content += output
                    
                    # Limit content to last 30 lines
                    lines = terminal_content.split('\n')
                    if len(lines) > 30:
                        terminal_content = '\n'.join(lines[-30:])
                    
                    # Create frames with slower playback
                    # Add repeated frames for slower motion
                    img = create_terminal_image(terminal_content, terminal_width, terminal_height)
                    frame_path = os.path.join(tmpdirname, f"frame_{frame_count:05d}.png")
                    img.save(frame_path)
                    
                    # Adjust speed based on time difference
                    repeat_frames = max(1, min(5, int(time_diff * 3)))
                    for _ in range(repeat_frames):
                        frames.append((frame_path, frame_duration * 3))
                    
                    frame_count += 1
            
            # Ensure we have at least one frame
            if frame_count == 0:
                img = create_terminal_image(terminal_content, terminal_width, terminal_height)
                frame_path = os.path.join(tmpdirname, "frame_00000.png")
                img.save(frame_path)
                frames.append((frame_path, frame_duration))
            
            # Create GIF using the saved frames
            if frames:
                images = []
                for frame_path, _ in frames:
                    images.append(Image.open(frame_path))
                
                # Save as GIF
                images[0].save(
                    output_gif,
                    save_all=True,
                    append_images=images[1:],
                    optimize=False,
                    duration=[duration for _, duration in frames],
                    loop=0
                )
                print(f"GIF created at {output_gif}")
            else:
                print("No frames were created.")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print(f"Usage: {sys.argv[0]} input.cast output.gif")
        sys.exit(1)
    
    cast_file = sys.argv[1]
    output_gif = sys.argv[2]
    
    asciinema_to_gif(cast_file, output_gif)
