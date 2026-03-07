export type SseEventHandler<T> = (event: T) => void;

export interface SseClient {
  close: () => void;
}

/**
 * Opens an SSE connection to the given URL and dispatches parsed events.
 *
 * Wraps native EventSource with JSON parsing, typed callbacks, and a
 * close handle for cleanup in React useEffect return functions.
 */
export function createSseClient<T = unknown>(
  url: string,
  options: {
    onEvent: SseEventHandler<T>;
    onError?: (error: Event) => void;
    onClose?: () => void;
  },
): SseClient {
  const source = new EventSource(url);

  source.onmessage = (event) => {
    try {
      const parsed = JSON.parse(event.data) as T;
      options.onEvent(parsed);
    } catch {
      // Non-JSON data — ignore
    }
  };

  source.onerror = (err) => {
    options.onError?.(err);
    if (source.readyState === EventSource.CLOSED) {
      options.onClose?.();
    }
  };

  return {
    close() {
      source.close();
      options.onClose?.();
    },
  };
}
