import React from 'react';
import { useHealthCheck } from '../hooks/useQuery';

export default function HealthStatus() {
  const { data, isLoading, error } = useHealthCheck();

  if (isLoading) return <div className="status-loading">Checking gateway...</div>;
  if (error) return <div className="status-error">Gateway unreachable</div>;

  return (
    <div className="health-status">
      <span className="status-ok">● Connected</span>
      <span>Gateway v{data.version}</span>
    </div>
  );
}
