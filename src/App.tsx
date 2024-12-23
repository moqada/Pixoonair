import React, {useCallback, useEffect, useState} from 'react';
import {invoke} from '@tauri-apps/api/core';
import './App.css';

const GIF_FILE_TYPES = ['id', 'url'] as const;

type AppSettings = {
  gifFileId: string;
  gifFileUrl: string;
  gifFileType: (typeof GIF_FILE_TYPES)[number];
  targetDeviceName: string;
};

const loadSettings = async (): Promise<AppSettings> => {
  const settings: AppSettings = await invoke('load_settings');
  return settings;
};

const saveSettings = async (settings: AppSettings): Promise<void> => {
  await invoke('save_settings', settings);
};

function App() {
  const [targetDeviceName, setTargetDeviceName] = useState('');
  const [gifFileId, setGifFileId] = useState('');
  const [gifFileUrl, setGifFileUrl] = useState('');
  const [gifFileType, setGifFileType] = useState<'id' | 'url'>('id');
  const onChangeTargetDeviceName = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      setTargetDeviceName(e.currentTarget.value);
    },
    []
  );
  const onChangeGifFileId = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      setGifFileId(e.currentTarget.value);
    },
    []
  );
  const onChangeGifFileUrl = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      setGifFileUrl(e.currentTarget.value);
    },
    []
  );
  const onChangeGifFileType = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      setGifFileType(e.currentTarget.value as 'id' | 'url');
    },
    []
  );
  const onSubmit = useCallback(
    (e: React.FormEvent<HTMLFormElement>) => {
      e.preventDefault();
      saveSettings({
        gifFileId,
        gifFileType,
        gifFileUrl,
        targetDeviceName,
      });
    },
    [gifFileId, gifFileType, gifFileUrl, targetDeviceName]
  );
  useEffect(() => {
    loadSettings().then((settings) => {
      setTargetDeviceName(settings.targetDeviceName);
      setGifFileId(settings.gifFileId);
      setGifFileUrl(settings.gifFileUrl);
      setGifFileType(settings.gifFileType);
    });
  }, []);

  return (
    <main className="container">
      <form className="row" onSubmit={onSubmit}>
        <label htmlFor="input-target-device-name">Target Device Name</label>
        <input
          id="input-target-device-name"
          onChange={onChangeTargetDeviceName}
          defaultValue={targetDeviceName}
          placeholder="Enter target device name..."
        />
        <fieldset>
          <legend>GIF File Type</legend>
          {GIF_FILE_TYPES.map((type) => (
            <React.Fragment key={type}>
              <input
                type="radio"
                name="gif-type"
                id={`input-gif-type-${type}`}
                onChange={onChangeGifFileType}
                checked={gifFileType === type}
                value={type}
              />
              <label htmlFor={`input-gif-type-${type}`}>
                {type.toUpperCase()}
              </label>
            </React.Fragment>
          ))}
        </fieldset>
        {gifFileType === 'id' && (
          <>
            <label htmlFor="input-gif-file-id">GIF File ID</label>
            <input
              id="input-gif-file-id"
              onChange={onChangeGifFileId}
              defaultValue={gifFileId}
              placeholder="Enter GIF File ID..."
            />
          </>
        )}
        {gifFileType === 'url' && (
          <>
            <label htmlFor="input-gif-file-url">GIF File URL</label>
            <input
              id="input-file-url"
              onChange={onChangeGifFileUrl}
              defaultValue={gifFileUrl}
              placeholder="Enter GIF File URL..."
            />
          </>
        )}
        <button type="submit">Save</button>
      </form>
    </main>
  );
}

export default App;
