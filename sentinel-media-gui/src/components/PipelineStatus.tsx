import React, { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

type StageStatus = 'pending' | 'in_progress' | 'completed' | 'error';

interface PipelineStage {
  name: string;
  status: StageStatus;
}

const statusIcons = {
  pending: '⏳',
  in_progress: '⚙️',
  completed: '✅',
  error: '❌',
};

const PipelineStatus: React.FC = () => {
  const [stages, setStages] = useState<PipelineStage[]>([
    { name: 'Scanner', status: 'pending' },
    { name: 'Research', status: 'pending' },
    { name: 'Media', status: 'pending' },
    { name: 'Publisher', status: 'pending' },
  ]);
  const [gcpProject, setGcpProject] = useState<string>('Checking...');

  useEffect(() => {
    const unlisten = listen<any>('pipeline_status', (event) => {
      const { stage, status } = event.payload;
      setStages((prevStages) =>
        prevStages.map((s) => (s.name === stage ? { ...s, status } : s))
      );
    });

    invoke<string>('get_active_gcp_project')
      .then(setGcpProject)
      .catch((err) => setGcpProject(`Error: ${err}`));

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  return (
    <div className="p-4 bg-gray-800 text-white rounded-lg shadow-lg">
      <div className="flex justify-between items-center mb-4">
        <h2 className="text-xl font-bold">Pipeline Status</h2>
        <span className="px-2 py-1 text-xs font-semibold text-blue-800 bg-blue-200 rounded-full">
          GCP: {gcpProject}
        </span>
      </div>
      <div className="flex justify-between">
        {stages.map((stage, index) => (
          <React.Fragment key={stage.name}>
            <div className="flex flex-col items-center">
              <div className="text-3xl">{statusIcons[stage.status]}</div>
              <div className="mt-1 text-sm font-medium">{stage.name}</div>
            </div>
            {index < stages.length - 1 && (
              <div className="flex-grow flex items-center">
                <div className="w-full h-1 bg-gray-600 rounded-full"></div>
              </div>
            )}
          </React.Fragment>
        ))}
      </div>
    </div>
  );
};

export default PipelineStatus;
