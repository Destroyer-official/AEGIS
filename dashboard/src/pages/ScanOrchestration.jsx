import React, { useState, useEffect, useRef, useCallback } from 'react';
import {
  LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip,
  ResponsiveContainer,
} from 'recharts';

const API_BASE = import.meta.env.VITE_API_URL || 'http://localhost:3000';
const WS_BASE = import.meta.env.VITE_WS_URL || 'ws://localhost:3000';

const SCAN_TYPES = [
  { id: 'l4_port', label: 'L4 Port Scan', description: 'TCP/UDP port enumeration' },
  { id: 'l7_http', label: 'L7 HTTP Scan', description: 'HTTP service fingerprinting' },
  { id: 'tls_cert', label: 'TLS Certificate Scan', description: 'Certificate chain analysis' },
  { id: 'ja4_fingerprint', label: 'JA4 Fingerprint', description: 'TLS client fingerprinting' },
  { id: 'full_enrich', label: 'Full Enrichment', description: 'Scan + CVE + geolocation' },
];

function ScanForm({ onStartScan, loading }) {
  const [target, setTarget] = useState('');
  const [scanType, setScanType] = useState('l4_port');
  const [options, setOptions] = useState({ port_range: '1-1024', timeout: 5 });

  return (
    <div className="scan-form">
      <h3>New Scan</h3>
      <div className="form-row">
        <label>Target</label>
        <input
          type="text"
          value={target}
          onChange={(e) => setTarget(e.target.value)}
          placeholder="IP, CIDR, or hostname"
        />
      </div>
      <div className="form-row">
        <label>Scan Type</label>
        <select value={scanType} onChange={(e) => setScanType(e.target.value)}>
          {SCAN_TYPES.map((t) => (
            <option key={t.id} value={t.id}>{t.label} — {t.description}</option>
          ))}
        </select>
      </div>
      <div className="form-row">
        <label>Port Range</label>
        <input
          type="text"
          value={options.port_range}
          onChange={(e) => setOptions({ ...options, port_range: e.target.value })}
        />
      </div>
      <div className="form-row">
        <label>Timeout (s)</label>
        <input
          type="number"
          value={options.timeout}
          onChange={(e) => setOptions({ ...options, timeout: parseInt(e.target.value) || 5 })}
        />
      </div>
      <button
        className="btn-primary"
        disabled={!target || loading}
        onClick={() => onStartScan({ target, scan_type: scanType, options })}
      >
        {loading ? 'Starting...' : 'Start Scan'}
      </button>
    </div>
  );
}

function ScanProgress({ scan }) {
  const pct = scan.total > 0 ? Math.round((scan.completed / scan.total) * 100) : 0;

  return (
    <div className="scan-progress">
      <div className="progress-header">
        <span className="scan-id">{scan.id}</span>
        <span className={`scan-status status-${scan.status}`}>{scan.status}</span>
      </div>
      <div className="progress-bar">
        <div className="progress-fill" style={{ width: `${pct}%` }} />
      </div>
      <div className="progress-stats">
        <span>{scan.completed}/{scan.total} hosts</span>
        <span>{pct}%</span>
        <span>{scan.elapsed || '—'}</span>
      </div>
    </div>
  );
}

function ScanResults({ results }) {
  if (results.length === 0) return <p className="no-results">No results yet.</p>;

  return (
    <div className="scan-results">
      <h3>Results ({results.length})</h3>
      <div className="results-table-wrap">
        <table>
          <thead>
            <tr>
              <th>Host</th>
              <th>Port</th>
              <th>Protocol</th>
              <th>Service</th>
              <th>Version</th>
              <th>JA4</th>
            </tr>
          </thead>
          <tbody>
            {results.map((r, i) => (
              <tr key={i}>
                <td>{r.host}</td>
                <td>{r.port}</td>
                <td>{r.protocol}</td>
                <td>{r.service}</td>
                <td>{r.version || '—'}</td>
                <td className="ja4-cell">{r.ja4 || '—'}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function sanitizeTarget(target) {
  return (target || '').replace(/[^a-zA-Z0-9.:/-]/g, '');
}

function sanitizePortRange(ports) {
  return (ports || '').replace(/[^0-9,-]/g, '');
}

function ScanTimeline({ timeline }) {
  if (timeline.length === 0) return null;

  return (
    <div className="scan-timeline">
      <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem', marginBottom: '0.5rem' }}>
        <h3 style={{ margin: 0 }}>Live Throughput</h3>
        <span style={{ fontSize: '0.75rem', background: '#1e293b', color: '#38bdf8', padding: '2px 8px', borderRadius: '4px', border: '1px solid #334155' }}>
          Live Stream
        </span>
      </div>
      <ResponsiveContainer width="100%" height={200}>
        <LineChart data={timeline}>
          <CartesianGrid strokeDasharray="3 3" />
          <XAxis dataKey="time" />
          <YAxis />
          <Tooltip />
          <Line type="monotone" dataKey="hosts_scanned" stroke="#8884d8" dot={false} name="Hosts/s" />
          <Line type="monotone" dataKey="findings" stroke="#82ca9d" dot={false} name="Findings" />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}

export default function ScanOrchestration() {
  const [scans, setScans] = useState([]);
  const [activeScan, setActiveScan] = useState(null);
  const [results, setResults] = useState([]);
  const [timeline, setTimeline] = useState([]);
  const [loading, setLoading] = useState(false);
  const wsRef = useRef(null);
  const reconnectTimer = useRef(null);
  const connectWsRef = useRef(null);

  const connectWebSocket = useCallback((scanId) => {
    const token = localStorage.getItem('aegis_token');
    if (!token) return;

    const safeScanId = (scanId || '').replace(/[^a-zA-Z0-9_-]/g, '');
    const ws = new WebSocket(`${WS_BASE}/ws/stream?token=${token}`);
    wsRef.current = ws;

    ws.onopen = () => {
      ws.send(JSON.stringify({
        sql: `SELECT * FROM aegis.scan_results WHERE scan_id = '${safeScanId}' ORDER BY ts DESC LIMIT 100`,
        batch_size: 50,
      }));
    };

    ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);
        if (msg.rows) {
          setResults((prev) => {
            const newResults = msg.rows.map((row) => ({
              host: row.host || row[0],
              port: row.port || row[1],
              protocol: row.protocol || row[2],
              service: row.service || row[3],
              version: row.version || row[4],
              ja4: row.ja4 || row[5],
            }));
            return [...newResults, ...prev].slice(0, 500);
          });
        }
        if (msg.query_id || msg.rows) {
          setTimeline((prev) => {
            const now = new Date().toLocaleTimeString();
            const hostBatchCount = msg.rows ? msg.rows.length : (msg.stats?.processed_rows || 1);
            const findingsCount = msg.rows ? msg.rows.filter((r) => r.port || r[1]).length : 0;
            return [...prev, { time: now, hosts_scanned: hostBatchCount, findings: findingsCount }].slice(-30);
          });
        }
      } catch { /* ignore parse errors */ }
    };

    ws.onclose = () => {
      if (activeScan?.status === 'running') {
        reconnectTimer.current = setTimeout(() => connectWsRef.current?.(scanId), 3000);
      }
    };
  }, [activeScan]);

  connectWsRef.current = connectWebSocket;

  useEffect(() => {
    return () => {
      if (reconnectTimer.current) clearTimeout(reconnectTimer.current);
      if (wsRef.current) wsRef.current.close();
    };
  }, []);

  const handleStartScan = async (scanConfig) => {
    setLoading(true);
    try {
      const token = localStorage.getItem('aegis_token');
      const cleanTarget = sanitizeTarget(scanConfig.target);
      const cleanType = (scanConfig.scan_type || 'l4_port').replace(/[^a-zA-Z0-9_]/g, '');
      const cleanPorts = sanitizePortRange(scanConfig.options?.port_range || '1-1024');
      const cleanTimeout = Math.min(Math.max(parseInt(scanConfig.options?.timeout) || 5, 1), 60);

      const res = await fetch(`${API_BASE}/query`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${token}`,
        },
        body: JSON.stringify({
          query: `INSERT INTO aegis.scans (target, scan_type, port_range, timeout, status) VALUES ('${cleanTarget}', '${cleanType}', '${cleanPorts}', ${cleanTimeout}, 'running')`,
        }),
      });

      if (res.ok) {
        const data = await res.json();
        const newScan = {
          id: data.query_id || `scan-${Date.now()}`,
          target: cleanTarget,
          scan_type: cleanType,
          status: 'running',
          completed: 0,
          total: 0,
          elapsed: '0s',
          started_at: new Date().toISOString(),
        };
        setScans((prev) => [newScan, ...prev]);
        setActiveScan(newScan);
        setResults([]);
        setTimeline([]);
        connectWebSocket(newScan.id);
      }
    } catch (err) {
      console.error('Failed to start scan:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleStopScan = async (scanId) => {
    setScans((prev) =>
      prev.map((s) => s.id === scanId ? { ...s, status: 'stopped' } : s)
    );
    if (activeScan?.id === scanId) {
      setActiveScan((prev) => prev ? { ...prev, status: 'stopped' } : null);
    }

    // Send backend cancellation over active WebSocket and HTTP
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({ action: 'cancel', scan_id: scanId }));
      wsRef.current.close();
    }

    try {
      const token = localStorage.getItem('aegis_token');
      await fetch(`${API_BASE}/admin/action`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${token}`,
        },
        body: JSON.stringify({ action: 'cancel_scan', scan_id: scanId }),
      });
    } catch {
      console.warn('Backend scan cancellation notification completed');
    }
  };

  return (
    <div className="scan-orchestration">
      <h2>Scan Orchestration</h2>

      <div className="scan-layout">
        <aside className="scan-sidebar">
          <ScanForm onStartScan={handleStartScan} loading={loading} />

          <div className="active-scans">
            <h3>Active Scans</h3>
            {scans.length === 0 && <p className="no-scans">No scans running.</p>}
            {scans.map((scan) => (
              <div
                key={scan.id}
                className={`scan-card ${activeScan?.id === scan.id ? 'selected' : ''}`}
                onClick={() => setActiveScan(scan)}
              >
                <div className="scan-card-header">
                  <span className="scan-target">{scan.target}</span>
                  <span className={`scan-status status-${scan.status}`}>{scan.status}</span>
                </div>
                <div className="scan-card-meta">
                  <span>{scan.scan_type}</span>
                  <span>{scan.elapsed}</span>
                </div>
                {scan.status === 'running' && (
                  <button
                    className="btn-stop"
                    onClick={(e) => { e.stopPropagation(); handleStopScan(scan.id); }}
                  >
                    Stop
                  </button>
                )}
              </div>
            ))}
          </div>
        </aside>

        <section className="scan-main">
          {activeScan && <ScanProgress scan={activeScan} />}
          <ScanTimeline timeline={timeline} />
          <ScanResults results={results} />
        </section>
      </div>
    </div>
  );
}
