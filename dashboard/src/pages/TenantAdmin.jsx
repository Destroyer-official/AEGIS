import React, { useState, useEffect, useCallback } from 'react';

const API_BASE = import.meta.env.VITE_API_URL || 'http://localhost:3000';

function useAuthHeaders() {
  const token = localStorage.getItem('aegis_token') || '';
  return { Authorization: `Bearer ${token}`, 'Content-Type': 'application/json' };
}

function TenantOverview({ tenant }) {
  return (
    <div className="tenant-card">
      <h3>{tenant.name}</h3>
      <div className="tenant-meta">
        <span className={`badge ${tenant.enabled ? 'badge-green' : 'badge-red'}`}>
          {tenant.enabled ? 'Active' : 'Disabled'}
        </span>
        <span className="tenant-id">{tenant.id}</span>
      </div>
      <div className="quota-grid">
        <div className="quota-item">
          <span className="quota-label">Rate Limit</span>
          <span className="quota-value">{tenant.max_rate_rps} req/s</span>
        </div>
        <div className="quota-item">
          <span className="quota-label">Scans/Day</span>
          <span className="quota-value">{tenant.max_scans_per_day}</span>
        </div>
        <div className="quota-item">
          <span className="quota-label">Queries/Hour</span>
          <span className="quota-value">{tenant.max_queries_per_hour}</span>
        </div>
        <div className="quota-item">
          <span className="quota-label">Data Retention</span>
          <span className="quota-value">{tenant.data_retention_days} days</span>
        </div>
      </div>
    </div>
  );
}

function UserInviteForm({ onInvite }) {
  const [email, setEmail] = useState('');
  const [role, setRole] = useState('viewer');

  const handleSubmit = (e) => {
    e.preventDefault();
    if (!email.trim()) return;
    onInvite({ email, role });
    setEmail('');
    setRole('viewer');
  };

  return (
    <form className="invite-form" onSubmit={handleSubmit}>
      <h3>Invite User</h3>
      <div className="form-row">
        <label>Email</label>
        <input
          type="email"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          placeholder="user@example.com"
          required
        />
      </div>
      <div className="form-row">
        <label>Role</label>
        <select value={role} onChange={(e) => setRole(e.target.value)}>
          <option value="viewer">Viewer — Read-only access</option>
          <option value="analyst">Analyst — Query and scan access</option>
          <option value="admin">Admin — Full access</option>
        </select>
      </div>
      <button type="submit" className="btn btn-primary">Send Invite</button>
    </form>
  );
}

function ApiKeyList({ keys, onRevoke }) {
  if (keys.length === 0) {
    return <p className="empty-state">No API keys configured.</p>;
  }

  return (
    <div className="api-key-list">
      <h3>API Keys</h3>
      <table>
        <thead>
          <tr>
            <th>Name</th>
            <th>Scopes</th>
            <th>Created</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {keys.map((key) => (
            <tr key={key.id}>
              <td>{key.name}</td>
              <td>{key.scopes.join(', ')}</td>
              <td>{new Date(key.created_at).toLocaleDateString()}</td>
              <td>
                <button className="btn btn-danger btn-sm" onClick={() => onRevoke(key.id)}>
                  Revoke
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function AuditLogViewer({ entries }) {
  if (entries.length === 0) {
    return <p className="empty-state">No audit log entries found.</p>;
  }

  return (
    <div className="audit-log-viewer">
      <h3>Audit Log ({entries.length} entries)</h3>
      <div className="audit-log-scroll">
        <table>
          <thead>
            <tr>
              <th>Timestamp</th>
              <th>User</th>
              <th>Method</th>
              <th>Path</th>
              <th>Status</th>
              <th>Response Time</th>
            </tr>
          </thead>
          <tbody>
            {entries.slice(-50).reverse().map((entry, i) => (
              <tr key={i}>
                <td>{new Date(entry.timestamp).toLocaleString()}</td>
                <td>{entry.user_id}</td>
                <td>
                  <span className={`method-badge method-${entry.method.toLowerCase()}`}>
                    {entry.method}
                  </span>
                </td>
                <td className="path-cell">{entry.path}</td>
                <td>
                  <span className={`status-badge status-${entry.status < 400 ? 'ok' : 'err'}`}>
                    {entry.status}
                  </span>
                </td>
                <td>{entry.response_time_ms}ms</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function TenantAdmin() {
  const [tenants, setTenants] = useState([]);
  const [selectedTenant, setSelectedTenant] = useState(null);
  const [apiKeys, setApiKeys] = useState([]);
  const [auditEntries, setAuditEntries] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [activeTab, setActiveTab] = useState('overview');
  const headers = useAuthHeaders();

  const fetchTenants = useCallback(async () => {
    try {
      const res = await fetch(`${API_BASE}/admin/tenants`, { headers });
      if (!res.ok) throw new Error(`Failed to fetch tenants: ${res.status}`);
      const data = await res.json();
      setTenants(data);
      if (data.length > 0 && !selectedTenant) {
        setSelectedTenant(data[0]);
      }
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  }, [headers, selectedTenant]);

  const fetchApiKeys = useCallback(async (tenantId) => {
    try {
      const res = await fetch(`${API_BASE}/admin/tenants/${tenantId}/api-keys`, { headers });
      if (!res.ok) return;
      const data = await res.json();
      setApiKeys(data);
    } catch {
      setApiKeys([]);
    }
  }, [headers]);

  const fetchAuditLog = useCallback(async (tenantId) => {
    try {
      const res = await fetch(`${API_BASE}/admin/tenants/${tenantId}/audit-log`, { headers });
      if (!res.ok) return;
      const data = await res.json();
      setAuditEntries(data);
    } catch {
      setAuditEntries([]);
    }
  }, [headers]);

  useEffect(() => {
    fetchTenants();
  }, [fetchTenants]);

  useEffect(() => {
    if (selectedTenant) {
      fetchApiKeys(selectedTenant.id);
      fetchAuditLog(selectedTenant.id);
    }
  }, [selectedTenant, fetchApiKeys, fetchAuditLog]);

  const handleInvite = async ({ email, role }) => {
    if (!selectedTenant) return;
    try {
      const res = await fetch(`${API_BASE}/admin/tenants/${selectedTenant.id}/invite`, {
        method: 'POST',
        headers,
        body: JSON.stringify({ email, role }),
      });
      if (!res.ok) throw new Error(`Invite failed: ${res.status}`);
      alert(`Invitation sent to ${email}`);
    } catch (err) {
      alert(`Error: ${err.message}`);
    }
  };

  const handleRevokeKey = async (keyId) => {
    if (!selectedTenant) return;
    if (!window.confirm('Revoke this API key?')) return;
    try {
      const res = await fetch(`${API_BASE}/admin/tenants/${selectedTenant.id}/api-keys/${keyId}`, {
        method: 'DELETE',
        headers,
      });
      if (!res.ok) throw new Error(`Revoke failed: ${res.status}`);
      fetchApiKeys(selectedTenant.id);
    } catch (err) {
      alert(`Error: ${err.message}`);
    }
  };

  const handleCreateApiKey = async () => {
    if (!selectedTenant) return;
    const name = window.prompt('API key name:');
    if (!name) return;
    try {
      const res = await fetch(`${API_BASE}/admin/tenants/${selectedTenant.id}/api-keys`, {
        method: 'POST',
        headers,
        body: JSON.stringify({ name, scopes: ['read', 'query'] }),
      });
      if (!res.ok) throw new Error(`Create failed: ${res.status}`);
      fetchApiKeys(selectedTenant.id);
    } catch (err) {
      alert(`Error: ${err.message}`);
    }
  };

  const handleDownloadAuditLog = async () => {
    if (!selectedTenant) return;
    try {
      const res = await fetch(`${API_BASE}/admin/tenants/${selectedTenant.id}/audit-log/export`, {
        headers,
      });
      if (!res.ok) throw new Error(`Export failed: ${res.status}`);
      const blob = await res.blob();
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `audit-log-${selectedTenant.id}-${Date.now()}.json`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (err) {
      alert(`Error: ${err.message}`);
    }
  };

  if (loading) return <div className="loading">Loading tenants...</div>;
  if (error) return <div className="error">Error: {error}</div>;

  return (
    <div className="page tenant-admin">
      <h2>Tenant Administration</h2>

      <div className="tenant-sidebar">
        <h3>Tenants</h3>
        <ul className="tenant-list">
          {tenants.map((t) => (
            <li
              key={t.id}
              className={`tenant-item ${selectedTenant?.id === t.id ? 'active' : ''}`}
              onClick={() => setSelectedTenant(t)}
            >
              <span className="tenant-name">{t.name}</span>
              <span className={`badge badge-sm ${t.enabled ? 'badge-green' : 'badge-red'}`}>
                {t.enabled ? 'On' : 'Off'}
              </span>
            </li>
          ))}
        </ul>
      </div>

      <div className="tenant-detail">
        {selectedTenant ? (
          <>
            <div className="tab-bar">
              <button
                className={`tab ${activeTab === 'overview' ? 'active' : ''}`}
                onClick={() => setActiveTab('overview')}
              >
                Overview
              </button>
              <button
                className={`tab ${activeTab === 'users' ? 'active' : ''}`}
                onClick={() => setActiveTab('users')}
              >
                Users
              </button>
              <button
                className={`tab ${activeTab === 'api-keys' ? 'active' : ''}`}
                onClick={() => setActiveTab('api-keys')}
              >
                API Keys
              </button>
              <button
                className={`tab ${activeTab === 'audit' ? 'active' : ''}`}
                onClick={() => setActiveTab('audit')}
              >
                Audit Log
              </button>
            </div>

            {activeTab === 'overview' && <TenantOverview tenant={selectedTenant} />}

            {activeTab === 'users' && (
              <div className="users-section">
                <UserInviteForm onInvite={handleInvite} />
              </div>
            )}

            {activeTab === 'api-keys' && (
              <div className="api-keys-section">
                <button className="btn btn-primary" onClick={handleCreateApiKey}>
                  Generate New Key
                </button>
                <ApiKeyList keys={apiKeys} onRevoke={handleRevokeKey} />
              </div>
            )}

            {activeTab === 'audit' && (
              <div className="audit-section">
                <button className="btn btn-secondary" onClick={handleDownloadAuditLog}>
                  Download Audit Log
                </button>
                <AuditLogViewer entries={auditEntries} />
              </div>
            )}
          </>
        ) : (
          <p className="empty-state">Select a tenant to manage.</p>
        )}
      </div>
    </div>
  );
}

export default TenantAdmin;
