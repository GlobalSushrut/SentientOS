#!/usr/bin/env python3

import json
import sys
import os
import subprocess
import tempfile
import shutil
from pathlib import Path

def convert_asciinema_to_termtosvg(cast_file_path, output_svg_path):
    """Convert asciinema cast file to termtosvg format."""
    # Create a temporary directory for processing
    with tempfile.TemporaryDirectory() as temp_dir:
        # Read the cast file
        with open(cast_file_path, 'r') as f:
            # Read and parse header
            header = json.loads(f.readline())
            width = header.get('width', 80)
            height = header.get('height', 24)
            
            # Create termtosvg compatible recording
            recording_path = os.path.join(temp_dir, 'recording')
            
            # Generate the SVG with termtosvg
            cmd = [
                'termtosvg', 
                output_svg_path, 
                '--template', 'window_frame', 
                '--screen-geometry', f'{width}x{height}',
                '--loop-delay', '2000',  # 2 second pause between loops
                '--min-frame-duration', '50',  # slow down playback for better readability
                '--max-frame-duration', '1000'  # cap long idle periods
            ]
            
            print(f"Converting asciinema cast to SVG using: {' '.join(cmd)}")
            
            # Run the conversion process
            termtosvg_process = subprocess.Popen(
                cmd,
                stdin=subprocess.PIPE,
                text=True
            )
            
            # Write header to simulate termtosvg format
            termtosvg_process.stdin.write(f"{{'version': 2, 'width': {width}, 'height': {height}}}\n")
            
            # Process events
            for line in f:
                event = json.loads(line)
                # Only process output events (type 'o')
                if event[1] == 'o':
                    # Format as termtosvg events
                    time_stamp = event[0]
                    output = event[2]
                    
                    # Write to termtosvg process
                    termtosvg_process.stdin.write(f"[{time_stamp}, 'o', {json.dumps(output)}]\n")
            
            # Close stdin to signal end of input
            termtosvg_process.stdin.close()
            termtosvg_process.wait()
            
            print(f"Conversion complete. SVG saved to: {output_svg_path}")

if __name__ == '__main__':
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} input.cast output.svg")
        sys.exit(1)
    
    cast_file = sys.argv[1]
    output_svg = sys.argv[2]
    
    if not os.path.exists(cast_file):
        print(f"Error: Cast file '{cast_file}' not found.")
        sys.exit(1)
    
    convert_asciinema_to_termtosvg(cast_file, output_svg)
