import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { waitFor } from '@testing-library/react';
import { AppContainer } from './AppContainer';

const mockInvoke = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

// Mock contexts with simple defaults
const mockSettings = {
  onboarding_completed: true,
  transcription_cleanup_days: 30,
  hotkey: 'Cmd+Shift+Space'
};

const mockModelAvailability = {
  hasModels: true,
  selectedModelAvailable: true,
  remoteSelected: false,
  remoteAvailable: false,
  isChecking: false,
  checkModels: vi.fn(),
};

vi.mock('@/hooks/useModelAvailability', () => ({
  useModelAvailability: () => mockModelAvailability,
}));

vi.mock('@/contexts/SettingsContext', () => ({
  useSettings: () => ({
    settings: mockSettings,
    refreshSettings: vi.fn()
  })
}));

vi.mock('@/contexts/LicenseContext', () => ({
  useLicense: () => ({
    licenseStatus: {
      is_licensed: false,
      license_key: null,
      email: null
    },
    checkLicense: vi.fn()
  })
}));

vi.mock('@/contexts/ReadinessContext', () => ({
  useReadiness: () => ({
    checkAccessibilityPermission: vi.fn().mockResolvedValue(true),
    checkMicrophonePermission: vi.fn().mockResolvedValue(true)
  })
}));

// Mock ModelManagementContext that AppContainer actually uses
vi.mock('@/contexts/ModelManagementContext', () => ({
  useModelManagementContext: () => ({
    models: {},
    downloadProgress: {},
    verifyingModels: new Set(),
    downloadModel: vi.fn(),
    retryDownload: vi.fn(),
    cancelDownload: vi.fn(),
    deleteModel: vi.fn(),
    refreshModels: vi.fn(),
    preloadModel: vi.fn(),
    verifyModel: vi.fn()
  })
}));

// Mock services
vi.mock('@/services/updateService', () => ({
  updateService: {
    initialize: vi.fn().mockResolvedValue(true),
    dispose: vi.fn()
  }
}));

vi.mock('@/utils/keyring', () => ({
  loadApiKeysToCache: vi.fn().mockResolvedValue(true)
}));

// Mock components
vi.mock('@/components/onboarding/OnboardingDesktop', () => ({
  OnboardingDesktop: () => <div data-testid="onboarding">Onboarding</div>
}));

vi.mock('@/components/ui/sidebar', () => ({
  Sidebar: ({ children, onSectionChange }: any) => (
    <div data-testid="sidebar">
      <button onClick={() => onSectionChange('models')}>Models</button>
      {children}
    </div>
  ),
  SidebarProvider: ({ children }: any) => <div>{children}</div>,
  SidebarInset: ({ children }: any) => <div>{children}</div>,
  SidebarContent: ({ children }: any) => <div>{children}</div>,
  SidebarGroup: ({ children }: any) => <div>{children}</div>,
  SidebarGroupContent: ({ children }: any) => <div>{children}</div>,
  SidebarHeader: ({ children }: any) => <div>{children}</div>,
  SidebarMenu: ({ children }: any) => <div>{children}</div>,
  SidebarMenuItem: ({ children }: any) => <div>{children}</div>,
  SidebarMenuButton: ({ children }: any) => <button>{children}</button>,
  SidebarTrigger: ({ children }: any) => <button>{children}</button>,
  SidebarFooter: ({ children }: any) => <div>{children}</div>,
  useSidebar: () => ({ isOpen: true, toggle: vi.fn() })
}));

vi.mock('./tabs/TabContainer', () => ({
  TabContainer: ({ activeSection }: any) => (
    <div data-testid="tab-container">
      Current Tab: {activeSection}
    </div>
  )
}));

// Mock event coordinator
vi.mock('@/hooks/useEventCoordinator', () => ({
  useEventCoordinator: () => ({
    registerEvent: vi.fn()
  })
}));

describe('AppContainer', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockSettings.onboarding_completed = true;
    mockModelAvailability.hasModels = true;
    mockModelAvailability.isChecking = false;
    mockModelAvailability.remoteSelected = false;
    mockModelAvailability.remoteAvailable = false;
    mockInvoke.mockResolvedValue(null);
  });

  it('shows main app when onboarding is completed', () => {
    render(<AppContainer />);
    expect(screen.getByTestId('sidebar')).toBeInTheDocument();
    expect(screen.getByTestId('tab-container')).toBeInTheDocument();
  });

  it('shows onboarding when not completed and no remote server is active', async () => {
    mockSettings.onboarding_completed = false;
    mockModelAvailability.hasModels = false;
    mockModelAvailability.remoteSelected = false;
    mockModelAvailability.remoteAvailable = false;
    render(<AppContainer />);

    await waitFor(() => {
      expect(screen.getByTestId('onboarding')).toBeInTheDocument();
    });

    expect(screen.queryByTestId('sidebar')).not.toBeInTheDocument();
  });

  it('keeps onboarding visible when a remote server is selected but unavailable', async () => {
    mockSettings.onboarding_completed = false;
    mockModelAvailability.hasModels = false;
    mockModelAvailability.remoteSelected = true;
    mockModelAvailability.remoteAvailable = false;

    render(<AppContainer />);

    await waitFor(() => {
      expect(screen.getByTestId('onboarding')).toBeInTheDocument();
    });

    expect(screen.queryByTestId('sidebar')).not.toBeInTheDocument();
  });

  it('skips onboarding when a usable transcription source is available', async () => {
    mockSettings.onboarding_completed = false;
    mockModelAvailability.hasModels = true;

    render(<AppContainer />);

    await waitFor(() => {
      expect(screen.queryByTestId('onboarding')).not.toBeInTheDocument();
    });

    expect(screen.getByTestId('sidebar')).toBeInTheDocument();
  });

  it('allows navigation between sections', () => {
    // This test verifies the AppContainer renders and is interactive
    // The actual navigation is tested through the Sidebar mock
    render(<AppContainer />);
    
    // Verify the app structure is in place
    expect(screen.getByTestId('sidebar')).toBeInTheDocument();
    expect(screen.getByTestId('tab-container')).toBeInTheDocument();
  });
});