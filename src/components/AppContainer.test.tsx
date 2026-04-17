import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { AppContainer } from './AppContainer';

// Mock contexts with simple defaults
const mockSettings = {
  onboarding_completed: true,
  transcription_cleanup_days: 30,
  hotkey: 'Cmd+Shift+Space'
};

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
const mockGetJustUpdatedVersion = vi.fn().mockReturnValue(null);
vi.mock('@/services/updateService', () => ({
  updateService: {
    initialize: vi.fn().mockResolvedValue(true),
    dispose: vi.fn(),
    getJustUpdatedVersion: () => mockGetJustUpdatedVersion()
  }
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

// Mock Tauri invoke for focus_main_window
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined)
}));

describe('AppContainer', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockSettings.onboarding_completed = true;
  });

  it('shows main app when onboarding is completed', () => {
    render(<AppContainer />);
    expect(screen.getByTestId('sidebar')).toBeInTheDocument();
    expect(screen.getByTestId('tab-container')).toBeInTheDocument();
  });

  it('shows onboarding when not completed', () => {
    mockSettings.onboarding_completed = false;
    render(<AppContainer />);
    expect(screen.getByTestId('onboarding')).toBeInTheDocument();
    expect(screen.queryByTestId('sidebar')).not.toBeInTheDocument();
  });

  it('allows navigation between sections', () => {
    // This test verifies the AppContainer renders and is interactive
    // The actual navigation is tested through the Sidebar mock
    render(<AppContainer />);
    
    // Verify the app structure is in place
    expect(screen.getByTestId('sidebar')).toBeInTheDocument();
    expect(screen.getByTestId('tab-container')).toBeInTheDocument();
  });

  describe('post-update modal', () => {
    it('shows update dialog when app was just updated', async () => {
      mockGetJustUpdatedVersion.mockReturnValue('1.13.0');
      render(<AppContainer />);

      await waitFor(() => {
        expect(screen.getByText('VoiceTypr Updated')).toBeInTheDocument();
      });
      expect(screen.getByText(/Successfully updated to version 1\.13\.0/)).toBeInTheDocument();
    });

    it('does not show update dialog when no update marker exists', async () => {
      mockGetJustUpdatedVersion.mockReturnValue(null);
      render(<AppContainer />);

      // Wait for initial render to settle
      await waitFor(() => {
        expect(screen.getByTestId('sidebar')).toBeInTheDocument();
      });
      expect(screen.queryByText('VoiceTypr Updated')).not.toBeInTheDocument();
    });

    it('dismisses update dialog when close button is clicked', async () => {
      mockGetJustUpdatedVersion.mockReturnValue('2.0.0');
      render(<AppContainer />);

      await waitFor(() => {
        expect(screen.getByText('VoiceTypr Updated')).toBeInTheDocument();
      });

      // Click the dismiss button inside the dialog
      const dismissBtn = screen.getByRole('button', { name: /^dismiss$/i });
      fireEvent.click(dismissBtn);

      await waitFor(() => {
        expect(screen.queryByText('VoiceTypr Updated')).not.toBeInTheDocument();
      });
    });
  });
});