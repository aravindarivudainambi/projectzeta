"use client";

import { useState, useCallback } from "react";
import {
  approveRun as apiApproveRun,
  rejectRun as apiRejectRun,
} from "@api-client";

/**
 * Manages approval modal state and dispatches approve/reject calls to the backend.
 *
 * Pass `null` when no run is active.
 */
export function useApproval(runId: string | null = null) {
  const [isOpen, setIsOpen] = useState(false);
  const [pendingAction, setPendingAction] = useState("");

  const requestApproval = useCallback((action: string) => {
    setPendingAction(action);
    setIsOpen(true);
  }, []);

  const approve = useCallback(async () => {
    if (!runId) return;
    await apiApproveRun(runId);
    setIsOpen(false);
  }, [runId]);

  const reject = useCallback(async () => {
    if (!runId) return;
    await apiRejectRun(runId);
    setIsOpen(false);
  }, [runId]);

  return { isOpen, pendingAction, requestApproval, approve, reject };
}
