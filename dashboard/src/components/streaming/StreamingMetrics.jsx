import React, { useState, useEffect, useRef, useCallback } from 'react';
import {
  LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip,
  ResponsiveContainer,
} from 'recharts';

const WS_BASE = import.meta.env.VITE_WS_URL || 'ws://localhost:3000';

export default function StreamingMetrics({ sql = null }) {
  const [data, setData] = useState([]);
  const [connected, setConnected] = useState(false);
  const [error, setError] = useState(null);
  const wsRef = useRef(null);
  const reconnectTimer = useRef(null);
  const connectRef = useRef(null);

  const connect = useCallback(() => {
    const token = localStorage.getItem('aegis_token');
    if (!token) {
      setError('Not authenticated');
      return;
    }

    const url = `${WS_BASE}/ws/stream?token=${token}`;
    const ws = new WebSocket(url);
    wsRef.current = ws;

    ws.onopen = () => {
      setConnected(true);
      setError(null);
      if (sql) {
        ws.send(JSON.stringify({ sql, batch_size: 100 }));
      }
    };

    ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);
        if (msg.error) {
          setError(msg.error);
          return;
        }
        if (msg.rows) {
          setData((prev) => {
            const next = prev.concat(
              msg.rows.map((row, i) => ({
                ...row,
                id: `${Date.now()}-${i}`,
                timestamp: new Date().toISOString(),
              }))
            );
            return next.slice(-200);
          });
        }
      } catch {
        setError('Failed to parse message');
      }
    };

    ws.onclose = () => {
      setConnected(false);
      reconnectTimer.current = setTimeout(() => connectRef.current?.(), 3000);
    };

    ws.onerror = () => {
      setError('WebSocket connection failed');
      ws.close();
    };
  }, [sql]);

  connectRef.current = connect;

  useEffect(() => {
    connect();
    return () => {
      if (reconnectTimer.current) clearTimeout(reconnectTimer.current);
      if (wsRef.current) wsRef.current.close();
    };
  }, [connect]);

  const sendQuery = useCallback(
    (newSql) => {
      if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
        wsRef.current.send(JSON.stringify({ sql: newSql, batch_size: 100 }));
      }
    },
    []
  );

  const chartData = data.slice(-50).map((d, i) => ({
    index: i,
    ...d,
  }));

  return (
    <div className="streaming-metrics">
      <div className="streaming-header">
        <h3>Live Stream Metrics</h3>
        <span className={connected ? 'status-connected' : 'status-disconnected'}>
          {connected ? '● Connected' : '○ Disconnected'}
        </span>
      </div>

      {error && <div className="streaming-error">{error}</div>}

      {chartData.length > 0 && (
        <ResponsiveContainer width="100%" height={300}>
          <LineChart data={chartData}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="index" />
            <YAxis />
            <Tooltip />
            <Line type="monotone" dataKey="count" stroke="#8884d8" dot={false} />
          </LineChart>
        </ResponsiveContainer>
      )}

      <div className="streaming-data">
        {data.length === 0 && connected && <p>Waiting for data...</p>}
        {data.slice(-10).reverse().map((row) => (
          <div key={row.id} className="stream-row">
            {JSON.stringify(row)}
          </div>
        ))}
      </div>

      <button onClick={() => sendQuery(sql || 'SELECT 1')}>Refresh</button>
    </div>
  );
}
