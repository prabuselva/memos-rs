import { useState, useEffect, useCallback } from 'react';
import { vectorDbApi, type VectorDBStatus } from '../lib/api';

export const useVectorDBStatus = () => {
  const [status, setStatus] = useState<VectorDBStatus | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const checkStatus = useCallback(async () => {
    try {
      setIsLoading(true);
      const result = await vectorDbApi.getStatus();
      setStatus(result);
      setError(null);
    } catch (err) {
      console.error('Failed to check Vector DB status:', err);
      setError('Failed to check Vector DB status');
      setStatus({
        enabled: false,
        available: false,
        message: 'Unable to connect to server',
      });
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    checkStatus();
  }, [checkStatus]);

  return {
    status,
    isLoading,
    error,
    checkStatus,
    isAvailable: status?.available ?? false,
    isEnabled: status?.enabled ?? false,
  };
};
