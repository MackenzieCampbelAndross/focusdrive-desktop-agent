import { DeviceInfo, EmployeeProfile } from '../types';

interface HeaderProps {
  currentUser: EmployeeProfile | null;
  deviceInfo: DeviceInfo | null;
  isDevMode: boolean;
  stateLabel: string;
}

export function Header({ currentUser, deviceInfo, isDevMode, stateLabel }: HeaderProps) {
  return (
    <header style={{
      display: 'flex',
      justifyContent: 'space-between',
      alignItems: 'center',
      padding: '0.75rem 1rem',
      backgroundColor: '#1e293b',
      color: '#f8fafc',
      borderRadius: '8px',
      marginBottom: '1rem',
      fontSize: '0.875rem'
    }}>
      <div>
        <div style={{ fontWeight: 'bold', fontSize: '1rem' }}>FocusDrive Desktop Agent</div>
        <div style={{ color: '#94a3b8', fontSize: '0.75rem' }}>
          Device: {deviceInfo ? deviceInfo.device_name : 'Loading...'} | OS: Windows | v0.1.0
        </div>
      </div>

      <div style={{ textAlign: 'right' }}>
        {isDevMode && (
          <span style={{
            backgroundColor: '#fbbf24',
            color: '#78350f',
            padding: '2px 8px',
            borderRadius: '4px',
            fontSize: '0.7rem',
            fontWeight: 'bold',
            marginRight: '8px'
          }}>
            DEV MODE
          </span>
        )}

        <span style={{
          backgroundColor: '#3b82f6',
          color: '#ffffff',
          padding: '2px 8px',
          borderRadius: '4px',
          fontSize: '0.75rem',
          fontWeight: 'bold'
        }}>
          {stateLabel}
        </span>

        {currentUser && (
          <div style={{ color: '#cbd5e1', fontSize: '0.75rem', marginTop: '2px' }}>
            {currentUser.name} ({currentUser.email})
          </div>
        )}
      </div>
    </header>
  );
}
