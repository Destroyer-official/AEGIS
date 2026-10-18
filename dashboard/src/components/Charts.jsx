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

function DataProvenanceBadge({ isLive, source, lastUpdated }) {
  if (isLive) {
    return (
      <div style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: '6px',
        padding: '3px 8px',
        borderRadius: '4px',
        fontSize: '11px',
        fontWeight: '600',
        background: 'rgba(34, 197, 94, 0.15)',
        color: '#4ade80',
        border: '1px solid rgba(34, 197, 94, 0.3)',
        marginBottom: '10px'
      }}>
        <span style={{ width: '6px', height: '6px', borderRadius: '50%', background: '#4ade80' }}></span>
        <span>LIVE TELEMETRY: {source || 'Active Probe Cluster'}</span>
        {lastUpdated && <span style={{ opacity: 0.8 }}>({lastUpdated})</span>}
      </div>
    );
  }

  return (
    <div style={{
      display: 'inline-flex',
      alignItems: 'center',
      gap: '6px',
      padding: '3px 8px',
      borderRadius: '4px',
      fontSize: '11px',
      fontWeight: '600',
      background: 'rgba(234, 179, 8, 0.12)',
      color: '#facc15',
      border: '1px solid rgba(234, 179, 8, 0.25)',
      marginBottom: '10px'
    }}>
      <span>⚠️ BASELINE DEMO DATA</span>
      <span style={{ opacity: 0.85, fontWeight: 'normal' }}>
        — Illustrative mock; connect live probe stream for real-time discoveries
      </span>
    </div>
  );
}

export function HostTimelineChart({ data = mockTimeData, isLive = false, source, lastUpdated }) {
  const isMock = !isLive && (data === mockTimeData || data?.length === 0);

  return (
    <div className="chart-card">
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', flexWrap: 'wrap', gap: '8px' }}>
        <h3>Host Discovery Timeline</h3>
        <DataProvenanceBadge isLive={!isMock} source={source} lastUpdated={lastUpdated} />
      </div>
      <ResponsiveContainer width="100%" height={300}>
        <LineChart data={data && data.length > 0 ? data : mockTimeData}>
          <CartesianGrid strokeDasharray="3 3" stroke="#2a2a3a" />
          <XAxis dataKey="time" stroke="#888899" />
          <YAxis stroke="#888899" />
          <Tooltip contentStyle={{ background: '#1a1a2e', borderColor: '#3a3a4a' }} />
          <Line type="monotone" dataKey="hosts" stroke="#8884d8" name="Active Hosts" />
          <Line type="monotone" dataKey="ports" stroke="#82ca9d" name="Open Ports" />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}

export function VulnerabilityChart({ data = mockSeverityData, isLive = false, source, lastUpdated }) {
  const isMock = !isLive && (data === mockSeverityData || data?.length === 0);

  return (
    <div className="chart-card">
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', flexWrap: 'wrap', gap: '8px' }}>
        <h3>CVE Severity Distribution</h3>
        <DataProvenanceBadge isLive={!isMock} source={source} lastUpdated={lastUpdated} />
      </div>
      <ResponsiveContainer width="100%" height={300}>
        <BarChart data={data && data.length > 0 ? data : mockSeverityData}>
          <CartesianGrid strokeDasharray="3 3" stroke="#2a2a3a" />
          <XAxis dataKey="severity" stroke="#888899" />
          <YAxis stroke="#888899" />
          <Tooltip contentStyle={{ background: '#1a1a2e', borderColor: '#3a3a4a' }} />
          <Bar dataKey="count" fill="#ff7300" name="Findings Count" />
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
}
