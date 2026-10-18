import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
    vus: 1,
    iterations: 1,
};

const TRINO_URL = 'http://localhost:8080/v1/statement';
const HEADERS = {
    'Content-Type': 'text/plain',
    'X-Trino-User': 'aegis',
    'X-Trino-Catalog': 'aegis',
    'X-Trino-Schema': 'aegis',
};

export default function () {
    const res = http.post(TRINO_URL, 'SELECT 1', { headers: HEADERS });
    check(res, {
        'Trino responds 200': (r) => r.status === 200,
    });
}
