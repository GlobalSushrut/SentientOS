// SentientOS Advanced IoT Application
// Uses SentientOS' MatrixBox for containerization and ZK Store for secure data

const express = require('express');
const http = require('http');
const socketIo = require('socket.io');
const path = require('path');
const { v4: uuidv4 } = require('uuid');
const fs = require('fs');

// Simulate SentientOS API access
const sentientOS = {
  matrixbox: {
    createContainer: (name) => console.log(`Creating MatrixBox container: ${name}`),
    runInContainer: (name, fn) => {
      console.log(`Running in MatrixBox container: ${name}`);
      return fn();
    }
  },
  store: {
    saveData: (key, data) => {
      console.log(`Storing data with ZK verification: ${key}`);
      return true;
    },
    getData: (key) => {
      console.log(`Retrieving data with ZK verification: ${key}`);
      return { timestamp: Date.now(), value: Math.random() * 100 };
    }
  },
  gossip: {
    broadcastToNetwork: (data) => {
      console.log(`Broadcasting data via gossip protocol: ${JSON.stringify(data)}`);
      return true;
    },
    listenForUpdates: (callback) => {
      console.log('Listening for gossip protocol updates');
      // Simulate incoming updates every 5 seconds
      setInterval(() => {
        const update = {
          id: uuidv4(),
          device: `iot-sensor-${Math.floor(Math.random() * 10)}`,
          reading: Math.random() * 100,
          timestamp: Date.now()
        };
        callback(update);
      }, 5000);
    }
  },
  intent: {
    recordAction: (action, metadata) => {
      console.log(`Recording developer intent: ${action}`);
      return true;
    }
  },
  zk: {
    verify: (data) => {
      console.log(`Verifying data with ZK proof: ${JSON.stringify(data)}`);
      return true;
    }
  }
};

// Initialize Express app
const app = express();
const server = http.createServer(app);
const io = socketIo(server);

// Middleware
app.use(express.static(path.join(__dirname, 'public')));
app.use(express.json());

// In-memory data store (in a real app this would use SentientOS store)
const devices = [];
const readings = [];

// Record our intent to start the IoT application
sentientOS.intent.recordAction('start_iot_application', { timestamp: Date.now() });

// Create secure container for IoT data processing
sentientOS.matrixbox.createContainer('iot-processing');

// Routes
app.get('/api/devices', (req, res) => {
  sentientOS.matrixbox.runInContainer('iot-processing', () => {
    res.json({ devices });
  });
});

app.post('/api/devices', (req, res) => {
  const newDevice = {
    id: uuidv4(),
    name: req.body.name || `Device-${devices.length + 1}`,
    type: req.body.type || 'generic',
    status: 'online',
    lastSeen: Date.now()
  };
  
  // Store with ZK verification
  sentientOS.zk.verify(newDevice);
  sentientOS.store.saveData(`device:${newDevice.id}`, newDevice);
  
  devices.push(newDevice);
  
  // Broadcast new device via gossip protocol
  sentientOS.gossip.broadcastToNetwork({
    action: 'new_device',
    device: newDevice
  });
  
  res.status(201).json(newDevice);
});

app.get('/api/readings', (req, res) => {
  sentientOS.matrixbox.runInContainer('iot-processing', () => {
    res.json({ readings: readings.slice(-100) }); // Last 100 readings
  });
});

app.post('/api/readings', (req, res) => {
  const newReading = {
    id: uuidv4(),
    deviceId: req.body.deviceId,
    value: req.body.value,
    unit: req.body.unit || '',
    timestamp: Date.now()
  };
  
  // Verify and store the reading
  sentientOS.zk.verify(newReading);
  sentientOS.store.saveData(`reading:${newReading.id}`, newReading);
  
  readings.push(newReading);
  
  // Broadcast via gossip protocol
  sentientOS.gossip.broadcastToNetwork({
    action: 'new_reading',
    reading: newReading
  });
  
  io.emit('new_reading', newReading);
  res.status(201).json(newReading);
});

// Socket.IO connection
io.on('connection', (socket) => {
  console.log('Client connected');
  
  // Send initial data
  socket.emit('init_devices', devices);
  socket.emit('init_readings', readings.slice(-100));
  
  // Handle device control
  socket.on('control_device', (data) => {
    console.log(`Controlling device: ${JSON.stringify(data)}`);
    
    // Record the intent
    sentientOS.intent.recordAction('control_device', data);
    
    // Broadcast via gossip protocol
    sentientOS.gossip.broadcastToNetwork({
      action: 'device_control',
      control: data
    });
    
    // Simulate device response
    setTimeout(() => {
      socket.emit('device_response', {
        deviceId: data.deviceId,
        status: 'success',
        timestamp: Date.now()
      });
    }, 500);
  });
  
  socket.on('disconnect', () => {
    console.log('Client disconnected');
  });
});

// Listen for device updates via gossip protocol
sentientOS.gossip.listenForUpdates((update) => {
  console.log('Received update via gossip protocol:', update);
  
  // Process update
  if (update.device) {
    const deviceIndex = devices.findIndex(d => d.id === update.device.id);
    if (deviceIndex >= 0) {
      devices[deviceIndex] = update.device;
    } else {
      devices.push(update.device);
    }
    io.emit('device_update', update.device);
  }
  
  if (update.reading) {
    readings.push(update.reading);
    io.emit('new_reading', update.reading);
  }
});

// Start the server
const PORT = process.env.PORT || 3001;
server.listen(PORT, () => {
  console.log(`SentientOS IoT application running on port ${PORT}`);
  console.log(`Using SentientOS features: MatrixBox, ZK-Store, Gossip Protocol`);
});
