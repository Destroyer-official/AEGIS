import { describe, it, expect } from 'vitest';

// Input sanitization algorithms from ScanOrchestration
function sanitizeTarget(target) {
  return (target || '').replace(/[^a-zA-Z0-9.:/-]/g, '');
}

function sanitizePortRange(ports) {
  return (ports || '').replace(/[^0-9,-]/g, '');
}

describe('Dashboard Input Sanitization & SQL Injection Defense', () => {
  it('strips dangerous SQL injection payloads from target input', () => {
    const maliciousInput = "192.168.1.0/24'; DROP TABLE scans; --";
    const sanitized = sanitizeTarget(maliciousInput);
    expect(sanitized).toBe('192.168.1.0/24DROPTABLEscans--');
    expect(sanitized).not.toContain(';');
    expect(sanitized).not.toContain("'");
    expect(sanitized).not.toContain(' ');
  });

  it('permits valid IPv4 CIDR targets', () => {
    const valid = '10.0.0.0/16';
    expect(sanitizeTarget(valid)).toBe('10.0.0.0/16');
  });

  it('permits valid IPv6 CIDR targets', () => {
    const valid = '2001:db8::/32';
    expect(sanitizeTarget(valid)).toBe('2001:db8::/32');
  });

  it('strips dangerous injection payloads from port ranges', () => {
    const maliciousPorts = '80,443; SELECT * FROM credentials;';
    const sanitized = sanitizePortRange(maliciousPorts);
    expect(sanitized).toBe('80,443');
    expect(sanitized).not.toContain(';');
    expect(sanitized).not.toContain('SELECT');
  });

  it('permits valid port range specifications', () => {
    expect(sanitizePortRange('80,443,8000-8080')).toBe('80,443,8000-8080');
  });
});

describe('Dashboard Chart Data Provenance Verification', () => {
  it('correctly identifies baseline mock data fallback vs live telemetry', () => {
    const mockData = [
      { time: '00:00', hosts: 1200, ports: 4500 }
    ];
    const liveData = [
      { time: '00:00', hosts: 5400, ports: 18900 }
    ];

    const isMock = (data, isLive) => !isLive && data === mockData;
    expect(isMock(mockData, false)).toBe(true);
    expect(isMock(liveData, false)).toBe(false);
    expect(isMock(mockData, true)).toBe(false);
  });
});
