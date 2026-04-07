// hooks/useTransaction.ts
// A custom React hook that fetches a transaction from our Rust backend.
//
// WHY A CUSTOM HOOK?
// Custom hooks let you extract stateful logic from components.
// This hook manages: loading state, error state, and the transaction data.
// Any component can call useTransaction(txid) and get all three.

import { useState, useCallback } from 'react';
import type { Transaction } from '../types';

interface UseTransactionResult {
  transaction: Transaction | null;
  loading: boolean;
  error: string | null;
  fetch: (txid: string) => Promise<void>;
}

export function useTransaction(): UseTransactionResult {
  const [transaction, setTransaction] = useState<Transaction | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // useCallback memoizes the function so it doesn't get recreated on every render.
  // The empty dependency array [] means "never recreate this function".
  const fetch = useCallback(async (txid: string) => {
    setLoading(true);
    setError(null);
    setTransaction(null);

    try {
      const response = await window.fetch(`/api/tx/${txid.trim()}`);
      const data = await response.json();

      if (!response.ok) {
        setError(data.error || `Error ${response.status}`);
        return;
      }

      setTransaction(data as Transaction);
    } catch (e) {
      setError('Network error — is the backend running on port 3001?');
    } finally {
      setLoading(false);
    }
  }, []);

  return { transaction, loading, error, fetch };
}
