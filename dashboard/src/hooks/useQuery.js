import { useQuery } from '@tanstack/react-query';
import { queryApi, healthApi } from '../api/client';

export function useTrinoQuery(sql, options = {}) {
  const { enabled = true, limit = 100 } = options;
  return useQuery({
    queryKey: ['trino-query', sql, limit],
    queryFn: async () => {
      const response = await queryApi.execute(sql, limit);
      return response.data;
    },
    enabled: enabled && !!sql,
    staleTime: 30000,
  });
}

export function useHealthCheck() {
  return useQuery({
    queryKey: ['health'],
    queryFn: async () => {
      const response = await healthApi.check();
      return response.data;
    },
    refetchInterval: 30000,
  });
}
