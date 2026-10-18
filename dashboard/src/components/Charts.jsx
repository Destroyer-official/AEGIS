import React from 'react';
import {
  LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip,
  ResponsiveContainer, BarChart, Bar
} from 'recharts';

const mockTimeData = [
  { time: '00:00', hosts: 1200, ports: 4500 },
  { time: '04:00', hosts: 1150, ports: 4200 },
  { time: '08:00', hosts: 1300, ports: 4800 },
  { time: '12:00', hosts: 1450, ports: 5200 },
  { time: '16:00', hosts: 1380, ports: 5000 },
  { time: '20:00', hosts: 1250, ports: 4600 },
];

const mockSeverityData = [
  { severity: 'Critical', count: 42 },
  { severity: 'High', count: 156 },
  { severity: 'Medium', count: 389 },
  { severity: 'Low', count: 1024 },
];

export function HostTimelineChart({ data = mockTimeData }) {
  return (
    <div className="chart-card">
      <h3>Host Discovery Timeline</h3>
      <ResponsiveContainer width="100%" height={300}>
        <LineChart data={data}>
          <CartesianGrid strokeDasharray="3 3" />
          <XAxis dataKey="time" />
          <YAxis />
          <Tooltip />
          <Line type="monotone" dataKey="hosts" stroke="#8884d8" />
          <Line type="monotone" dataKey="ports" stroke="#82ca9d" />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}

export function VulnerabilityChart({ data = mockSeverityData }) {
  return (
    <div className="chart-card">
      <h3>CVE Severity Distribution</h3>
      <ResponsiveContainer width="100%" height={300}>
        <BarChart data={data}>
          <CartesianGrid strokeDasharray="3 3" />
          <XAxis dataKey="severity" />
          <YAxis />
          <Tooltip />
          <Bar dataKey="count" fill="#ff7300" />
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
}
