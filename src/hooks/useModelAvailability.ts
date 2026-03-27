import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useSettings } from '@/contexts/SettingsContext';
import type { ModelInfo } from '@/types';

interface ModelStatusResponse {
  models: ModelInfo[];
}

interface RecognitionAvailabilitySnapshot {
  whisper_available: boolean;
  parakeet_available: boolean;
  soniox_selected: boolean;
  soniox_ready: boolean;
  remote_selected: boolean;
  remote_available: boolean;
}

export function useModelAvailability() {
  const { settings } = useSettings();
  const [hasModels, setHasModels] = useState<boolean | null>(null);
  const [selectedModelAvailable, setSelectedModelAvailable] = useState<boolean | null>(null);
  const [remoteSelected, setRemoteSelected] = useState(false);
  const [remoteAvailable, setRemoteAvailable] = useState(false);
  const [isChecking, setIsChecking] = useState(false);

  const checkModels = useCallback(async () => {
    setIsChecking(true);
    try {
      const [status, availabilityResult] = await Promise.all([
        invoke<ModelStatusResponse>('get_model_status'),
        invoke<RecognitionAvailabilitySnapshot | null>('get_recognition_availability_snapshot'),
      ]);

      const availability = availabilityResult ?? {
        whisper_available: false,
        parakeet_available: false,
        soniox_selected: false,
        soniox_ready: false,
        remote_selected: false,
        remote_available: false,
      };

      const readyModels = status.models.filter(
        (model) => model.downloaded && !model.requires_setup
      );

      const hasAnyReadySource = readyModels.length > 0 || availability.remote_available;
      setHasModels(hasAnyReadySource);
      setRemoteSelected(availability.remote_selected);
      setRemoteAvailable(availability.remote_available);

      if (availability.remote_selected) {
        setSelectedModelAvailable(availability.remote_available);
        return;
      }

      const selectedModel = settings?.current_model;
      if (selectedModel) {
        const match = status.models.find((model) => model.name === selectedModel);
        const isAvailable =
          !!match && match.downloaded && !match.requires_setup;
        setSelectedModelAvailable(isAvailable);
      } else {
        setSelectedModelAvailable(false);
      }
    } catch (error) {
      console.error('Failed to check model availability:', error);
      setHasModels(false);
      setSelectedModelAvailable(false);
      setRemoteSelected(false);
      setRemoteAvailable(false);
    } finally {
      setIsChecking(false);
    }
  }, [settings]);

  useEffect(() => {
    checkModels();
  }, [checkModels, settings]);

  useEffect(() => {
    const unlistenDownloaded = listen('model-downloaded', () => {
      checkModels();
    });

    const unlistenDeleted = listen('model-deleted', () => {
      checkModels();
    });

    const unlistenModelChanged = listen('model-changed', () => {
      checkModels();
    });

    const unlistenCloudSaved = listen('stt-key-saved', () => {
      checkModels();
    });

    const unlistenCloudRemoved = listen('stt-key-removed', () => {
      checkModels();
    });

    const unlistenSharingChanged = listen('sharing-status-changed', () => {
      checkModels();
    });

    const unlistenRemoteStatusChanged = listen('remote-server-status-changed', () => {
      checkModels();
    });

    return () => {
      Promise.all([
        unlistenDownloaded,
        unlistenDeleted,
        unlistenModelChanged,
        unlistenCloudSaved,
        unlistenCloudRemoved,
        unlistenSharingChanged,
        unlistenRemoteStatusChanged,
      ]).then(unsubs => {
        unsubs.forEach(unsub => unsub());
      });
    };
  }, [checkModels]);

  return {
    hasModels,
    selectedModelAvailable,
    remoteSelected,
    remoteAvailable,
    isChecking,
    checkModels
  };
}
