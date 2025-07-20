// SentientOS IoT Application - Frontend Logic
document.addEventListener('DOMContentLoaded', () => {
    // Connect to Socket.IO server
    const socket = io();
    let selectedDeviceId = '';
    let readingsChart = null;
    
    // Chart configuration
    const chartColors = {
        temperature: 'rgb(255, 99, 132)',
        humidity: 'rgb(54, 162, 235)',
        pressure: 'rgb(255, 205, 86)',
        airQuality: 'rgb(75, 192, 192)'
    };
    
    // Initialize readings chart
    initChart();
    
    // Socket events
    socket.on('connect', () => {
        addLogEntry('[System] Connected to SentientOS IoT server');
    });
    
    socket.on('init_devices', (devices) => {
        populateDeviceList(devices);
        updateDeviceSelect(devices);
    });
    
    socket.on('init_readings', (readings) => {
        updateChartWithReadings(readings);
        updateCurrentValues(readings[readings.length - 1]);
    });
    
    socket.on('new_reading', (reading) => {
        addNewReading(reading);
        addLogEntry(`[Data] New reading received from device ${reading.deviceId.substring(0, 8)}`);
    });
    
    socket.on('device_update', (device) => {
        updateDevice(device);
        addLogEntry(`[Device] Device ${device.name} status updated`);
    });
    
    socket.on('device_response', (response) => {
        showCommandFeedback(response.status === 'success' ? 'success' : 'danger', 
            `Command sent to device ${response.deviceId.substring(0, 8)} - ${response.status}`);
        addLogEntry(`[Command] Device command response: ${response.status}`);
    });
    
    // UI Event Listeners
    document.getElementById('save-device-btn').addEventListener('click', () => {
        const name = document.getElementById('device-name').value;
        const type = document.getElementById('device-type').value;
        
        if (!name) {
            alert('Please enter a device name');
            return;
        }
        
        fetch('/api/devices', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ name, type })
        })
        .then(response => response.json())
        .then(device => {
            addLogEntry(`[ZK] Device ${device.name} added with zero-knowledge verification`);
            const modal = bootstrap.Modal.getInstance(document.getElementById('addDeviceModal'));
            modal.hide();
        })
        .catch(error => {
            console.error('Error adding device:', error);
            alert('Error adding device');
        });
    });
    
    document.getElementById('send-command-btn').addEventListener('click', () => {
        const deviceId = document.getElementById('control-device-select').value;
        const action = document.getElementById('control-action-select').value;
        
        if (!deviceId || !action) {
            alert('Please select a device and action');
            return;
        }
        
        socket.emit('control_device', { deviceId, action });
        addLogEntry(`[Command] Sent "${action}" command to device`);
        showCommandFeedback('info', 'Command sent, awaiting response...');
    });
    
    // Setup simulated data generation
    simulateIoTData();
    
    // Functions
    function initChart() {
        const ctx = document.getElementById('readings-chart').getContext('2d');
        readingsChart = new Chart(ctx, {
            type: 'line',
            data: {
                labels: [],
                datasets: [
                    {
                        label: 'Temperature',
                        backgroundColor: chartColors.temperature,
                        borderColor: chartColors.temperature,
                        data: [],
                        fill: false
                    },
                    {
                        label: 'Humidity',
                        backgroundColor: chartColors.humidity,
                        borderColor: chartColors.humidity,
                        data: [],
                        fill: false
                    }
                ]
            },
            options: {
                responsive: true,
                scales: {
                    x: {
                        display: true,
                        title: {
                            display: true,
                            text: 'Time'
                        }
                    },
                    y: {
                        display: true,
                        title: {
                            display: true,
                            text: 'Value'
                        }
                    }
                }
            }
        });
    }
    
    function populateDeviceList(devices) {
        const deviceList = document.getElementById('device-list');
        deviceList.innerHTML = '';
        
        if (devices.length === 0) {
            deviceList.innerHTML = '<div class="list-group-item">No devices connected</div>';
            return;
        }
        
        devices.forEach(device => {
            const deviceItem = document.createElement('div');
            deviceItem.className = 'list-group-item list-group-item-action device-card';
            deviceItem.dataset.deviceId = device.id;
            deviceItem.innerHTML = `
                <div class="d-flex justify-content-between">
                    <h6>${device.name}</h6>
                    <span class="device-${device.status === 'online' ? 'online' : 'offline'}">
                        ${device.status === 'online' ? '⚫ Online' : '⚫ Offline'}
                    </span>
                </div>
                <div class="small text-muted">${device.type}</div>
                <div class="small">ID: ${device.id.substring(0, 8)}...</div>
            `;
            
            deviceItem.addEventListener('click', () => {
                // Select device and update UI
                document.querySelectorAll('.device-card').forEach(card => {
                    card.classList.remove('active');
                });
                deviceItem.classList.add('active');
                selectedDeviceId = device.id;
                document.getElementById('control-device-select').value = device.id;
            });
            
            deviceList.appendChild(deviceItem);
        });
    }
    
    function updateDeviceSelect(devices) {
        const deviceSelect = document.getElementById('control-device-select');
        deviceSelect.innerHTML = '<option value="">-- Select Device --</option>';
        
        devices.forEach(device => {
            const option = document.createElement('option');
            option.value = device.id;
            option.text = `${device.name} (${device.type})`;
            deviceSelect.appendChild(option);
        });
    }
    
    function updateDevice(device) {
        const deviceItem = document.querySelector(`.device-card[data-device-id="${device.id}"]`);
        if (deviceItem) {
            const statusSpan = deviceItem.querySelector('span');
            statusSpan.className = `device-${device.status === 'online' ? 'online' : 'offline'}`;
            statusSpan.textContent = device.status === 'online' ? '⚫ Online' : '⚫ Offline';
        } else {
            // If device doesn't exist yet, refresh the whole list
            fetch('/api/devices')
                .then(response => response.json())
                .then(data => {
                    populateDeviceList(data.devices);
                    updateDeviceSelect(data.devices);
                });
        }
    }
    
    function addNewReading(reading) {
        // Update chart
        const timestamp = new Date(reading.timestamp).toLocaleTimeString();
        
        if (readingsChart.data.labels.length > 20) {
            readingsChart.data.labels.shift();
            readingsChart.data.datasets.forEach(dataset => {
                dataset.data.shift();
            });
        }
        
        readingsChart.data.labels.push(timestamp);
        
        // This assumes readings have proper type information
        // In a real app, we'd map reading.type to the right dataset
        if (reading.type === 'temperature') {
            readingsChart.data.datasets[0].data.push(reading.value);
        } else if (reading.type === 'humidity') {
            readingsChart.data.datasets[1].data.push(reading.value);
        }
        
        readingsChart.update();
        
        // Update current values
        updateCurrentValues(reading);
    }
    
    function updateChartWithReadings(readings) {
        // Group readings by type
        const temperatureReadings = readings.filter(r => r.type === 'temperature');
        const humidityReadings = readings.filter(r => r.type === 'humidity');
        
        readingsChart.data.labels = temperatureReadings.map(r => 
            new Date(r.timestamp).toLocaleTimeString());
        
        readingsChart.data.datasets[0].data = temperatureReadings.map(r => r.value);
        readingsChart.data.datasets[1].data = humidityReadings.map(r => r.value);
        
        readingsChart.update();
    }
    
    function updateCurrentValues(reading) {
        // In a real app, we'd use actual sensor types
        // For simulation, we'll update based on a timer
        document.getElementById('last-updated').textContent = new Date().toLocaleTimeString();
    }
    
    function addLogEntry(message) {
        const logs = document.getElementById('system-logs');
        const entry = document.createElement('p');
        entry.className = 'log-entry';
        entry.textContent = message;
        logs.appendChild(entry);
        
        // Auto-scroll to bottom
        logs.scrollTop = logs.scrollHeight;
        
        // Limit log entries
        if (logs.children.length > 100) {
            logs.removeChild(logs.children[0]);
        }
    }
    
    function showCommandFeedback(type, message) {
        const feedback = document.getElementById('command-feedback');
        feedback.className = `mt-2 alert alert-${type}`;
        feedback.textContent = message;
        feedback.classList.remove('d-none');
        
        // Auto-hide after 5 seconds
        setTimeout(() => {
            feedback.classList.add('d-none');
        }, 5000);
    }
    
    function simulateIoTData() {
        // Simulated data generation for demo purposes
        function generateReading() {
            const temp = 20 + Math.random() * 10;
            const humidity = 30 + Math.random() * 40;
            const pressure = 980 + Math.random() * 50;
            const airQuality = Math.floor(Math.random() * 100);
            
            document.getElementById('temperature-value').textContent = `${temp.toFixed(1)}°C`;
            document.getElementById('humidity-value').textContent = `${humidity.toFixed(1)}%`;
            document.getElementById('pressure-value').textContent = `${pressure.toFixed(0)} hPa`;
            document.getElementById('air-value').textContent = airQuality;
            
            // Add ZK verified badge to the updated values
            addZkBadge('temperature-value');
            addZkBadge('humidity-value');
            addZkBadge('pressure-value');
            addZkBadge('air-value');
            
            document.getElementById('last-updated').textContent = new Date().toLocaleTimeString();
            addLogEntry('[ZK] Sensor readings verified with zero-knowledge proof');
        }
        
        // Update readings every 3-5 seconds
        function scheduleUpdate() {
            const delay = 3000 + Math.random() * 2000;
            setTimeout(() => {
                generateReading();
                scheduleUpdate();
            }, delay);
        }
        
        scheduleUpdate();
    }
    
    function addZkBadge(elementId) {
        const element = document.getElementById(elementId);
        let badge = element.querySelector('.zk-verified');
        
        if (!badge) {
            badge = document.createElement('span');
            badge.className = 'zk-verified';
            badge.innerHTML = '✓ ZK';
            element.appendChild(badge);
        }
    }
});
