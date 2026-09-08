import { ActivityIdentity } from '@focusdrive/shared-types';

export interface ActivityResolutionRule {
  matchPattern: string; // Executable name or domain substring/regex
  displayName: string;
  category: string;
  type: string;
}

export function formatDuration(seconds: number): string {
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const remMinutes = minutes % 60;
  if (hours > 0) {
    return `${hours}h ${remMinutes}m`;
  }
  return `${minutes}m`;
}
