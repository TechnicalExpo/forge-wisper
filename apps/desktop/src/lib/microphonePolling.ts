export type StopMicrophonePolling = () => void;

export const startMicrophoneLevelPolling = (
  readLevel: () => Promise<number>,
  onLevel: (level: number) => void,
  intervalMs = 80,
): StopMicrophonePolling => {
  let active = true;
  let timeoutId: ReturnType<typeof setTimeout> | undefined;
  const delay = Math.max(80, intervalMs);

  const poll = async () => {
    try {
      const level = await readLevel();
      if (active) onLevel(level);
    } catch {
      // The recording lifecycle owns retry behavior; a transient meter read is ignored.
    } finally {
      if (active) timeoutId = setTimeout(poll, delay);
    }
  };

  void poll();

  return () => {
    active = false;
    if (timeoutId !== undefined) clearTimeout(timeoutId);
  };
};
