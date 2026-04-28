import { useAccessibilityPermission } from './useAccessibilityPermission';
import { useMicrophonePermission } from './useMicrophonePermission';
import { useModelAvailability } from './useModelAvailability';
import { useSettings } from '@/contexts/SettingsContext';

/**
 * Computed hook that combines all domain-specific hooks to provide derived readiness values.
 */
export function useAppReadiness() {
  const { settings } = useSettings();
  const onboardingCompleted = settings?.onboarding_completed === true;

  const accessibility = useAccessibilityPermission({ checkOnMount: onboardingCompleted });
  const microphone = useMicrophonePermission({ checkOnMount: onboardingCompleted });
  const models = useModelAvailability();

  const canRecord = Boolean(
    microphone.hasPermission &&
    models.hasModels &&
    models.selectedModelAvailable
  );

  const canAutoInsert = Boolean(accessibility.hasPermission);

  const isFullyReady = Boolean(
    microphone.hasPermission &&
    accessibility.hasPermission &&
    models.hasModels &&
    models.selectedModelAvailable
  );

  const isLoading = (
    accessibility.isChecking ||
    microphone.isChecking ||
    models.isChecking ||
    accessibility.hasPermission === null ||
    microphone.hasPermission === null ||
    models.hasModels === null
  );

  return {
    hasAccessibilityPermission: accessibility.hasPermission,
    hasMicrophonePermission: microphone.hasPermission,
    hasModels: models.hasModels,
    selectedModelAvailable: models.selectedModelAvailable,
    licenseValid: true,
    canRecord,
    canAutoInsert,
    isFullyReady,
    isLoading,
    requestAccessibilityPermission: accessibility.requestPermission,
    requestMicrophonePermission: microphone.requestPermission,
    checkAccessibilityPermission: accessibility.checkPermission,
    checkMicrophonePermission: microphone.checkPermission,
    checkModels: models.checkModels,
    checkLicense: async () => {},
  };
}
