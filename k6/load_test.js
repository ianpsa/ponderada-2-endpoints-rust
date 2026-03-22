import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

const errorRate = new Rate('errors');

export const options = {
  stages: [
    { duration: '30s', target: 10  },  // ramp up
    { duration: '1m',  target: 50  },  // carga sustentada
    { duration: '30s', target: 100 },  // spike
    { duration: '30s', target: 0   },  // ramp down
  ],
  thresholds: {
    http_req_duration: ['p(95)<500'],  // 95% das requests abaixo de 500ms
    errors: ['rate<0.01'],             // taxa de erro abaixo de 1%
  },
};

const DEVICE_IDS = ['dev-001', 'dev-002', 'dev-003', 'dev-004', 'dev-005'];

const SENSORS = [
  { sensor_type: 'temperature',     reading_nature: 'analog',   value: () => parseFloat((Math.random() * 60 - 10).toFixed(2)) },
  { sensor_type: 'humidity',        reading_nature: 'analog',   value: () => parseFloat((Math.random() * 100).toFixed(2)) },
  { sensor_type: 'presence',        reading_nature: 'discrete', value: () => Math.random() > 0.5 },
  { sensor_type: 'vibration',       reading_nature: 'analog',   value: () => parseFloat((Math.random() * 10).toFixed(3)) },
  { sensor_type: 'luminosity',      reading_nature: 'analog',   value: () => parseFloat((Math.random() * 1000).toFixed(1)) },
  { sensor_type: 'reservoir_level', reading_nature: 'analog',   value: () => parseFloat((Math.random() * 100).toFixed(2)) },
];

export default function () {
  const device  = DEVICE_IDS[Math.floor(Math.random() * DEVICE_IDS.length)];
  const sensor  = SENSORS[Math.floor(Math.random() * SENSORS.length)];

  const payload = JSON.stringify({
    device_id:      device,
    timestamp:      new Date().toISOString(),
    sensor_type:    sensor.sensor_type,
    reading_nature: sensor.reading_nature,
    value:          sensor.value(),
  });

  const res = http.post('http://localhost:8080/telemetry', payload, {
    headers: { 'Content-Type': 'application/json' },
  });

  const ok = check(res, {
    'status 202': (r) => r.status === 202,
    'latency < 200ms': (r) => r.timings.duration < 200,
  });

  errorRate.add(!ok);
  sleep(0.1);
}
