import React, { useState } from 'react';
import { useTrinoQuery } from '../hooks/useQuery';

export default function QueryEditor() {
  const [sql, setSql] = useState('SELECT 1 AS test');
  const { data, isLoading, error, refetch } = useTrinoQuery(sql, { enabled: false });

  const handleExecute = () => {
    refetch();
  };

  return (
    <div className="query-editor">
      <h3>Query Editor</h3>
      <textarea
        value={sql}
        onChange={(e) => setSql(e.target.value)}
        placeholder="Enter SQL query..."
        rows={4}
      />
      <button onClick={handleExecute} disabled={isLoading}>
        {isLoading ? 'Executing...' : 'Execute'}
      </button>
      
      {error && (
        <div className="query-error">
          Error: {error.response?.data?.detail || error.message}
        </div>
      )}
      
      {data && (
        <div className="query-results">
          <p>Query ID: {data.query_id}</p>
          <p>Rows: {data.total_rows}</p>
          <table>
            <thead>
              <tr>
                {data.columns.map((col) => (
                  <th key={col.name}>{col.name}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {data.rows.slice(0, 100).map((row, i) => (
                <tr key={i}>
                  {row.map((cell, j) => (
                    <td key={j}>{cell}</td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
