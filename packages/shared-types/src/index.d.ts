export type EmployeeState = 'LOGGED_OUT' | 'SHIFT_READY' | 'WORKING' | 'BREAK' | 'SHIFT_SUMMARY';
export type ActivitySource = 'desktop_application' | 'browser';
export interface DeviceInfo {
    deviceId: string;
    employeeId?: string;
    deviceName: string;
    operatingSystem: string;
    agentVersion: string;
    lastSeenAt: string;
    createdAt: string;
}
export interface ActivityIdentity {
    identifier: string;
    displayName: string;
    type: string;
    category: string;
    source: ActivitySource;
    executable?: string;
    domain?: string;
}
export interface WorkSession {
    id: string;
    employeeId: string;
    deviceId: string;
    startedAt: string;
    endedAt?: string;
    status: 'active' | 'completed' | 'recovered';
    createdAt: string;
}
export interface BreakSession {
    id: string;
    workSessionId: string;
    employeeId: string;
    type: 'manual' | 'automatic_idle';
    startedAt: string;
    endedAt?: string;
    durationSeconds: number;
    createdAt: string;
}
export interface ActivityUsageAggregate {
    id: string;
    workSessionId: string;
    employeeId: string;
    activityCategoryId: string;
    source: ActivitySource;
    durationSeconds: number;
    firstSeenAt: string;
    lastSeenAt: string;
}
