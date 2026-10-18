import React, { useState } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { BrowserRouter, Routes, Route, Link } from 'react-router-dom';
import HealthStatus from './components/HealthStatus';
import QueryEditor from './components/QueryEditor';
import { HostTimelineChart, VulnerabilityChart } from './components/Charts';
import ScanOrchestration from './pages/ScanOrchestration';
import TenantAdmin from './pages/TenantAdmin';
import './App.css';

const queryClient = new QueryClient();

function Dashboard() {
  return (
    <div className="dashboard">
      <header>
        <h1>AEGIS Dashboard</h1>
        <HealthStatus />
      </header>
      
      <nav className="nav-tabs">
        <Link to="/">Overview</Link>
        <Link to="/query">Query</Link>
        <Link to="/scans">Scans</Link>
        <Link to="/assets">Assets</Link>
        <Link to="/admin/tenants">Tenants</Link>
      </nav>
      
      <main>
        <div className="charts-grid">
          <HostTimelineChart />
          <VulnerabilityChart />
        </div>
      </main>
    </div>
  );
}

function QueryPage() {
  return (
    <div className="page">
      <h2>Query Interface</h2>
      <QueryEditor />
    </div>
  );
}

function AssetsPage() {
  return (
    <div className="page">
      <h2>Asset Explorer</h2>
      <p>Asset lookup and filtering will be implemented here.</p>
    </div>
  );
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/query" element={<QueryPage />} />
          <Route path="/scans" element={<ScanOrchestration />} />
          <Route path="/assets" element={<AssetsPage />} />
          <Route path="/admin/tenants" element={<TenantAdmin />} />
        </Routes>
      </BrowserRouter>
    </QueryClientProvider>
  );
}

export default App;
