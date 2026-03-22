import React from 'react';
import { Composition } from 'remotion';
import { HelloWorld } from './HelloWorld';
import { DynamicVideo } from './DynamicVideo';
import { SentinelManifest } from './types';
import { CyberCortex } from './CyberCortex';
import { LaEspiguita } from './LaEspiguita';

const defaultManifest: SentinelManifest = {
	title: "NEURAL SYNC",
	description: "System online...",
	clips: [
		{ type: 'text', text: "Analyzing Vector Space...", durationInSec: 3 },
		{ type: 'text', text: "Optimizing Neural Pathways...", durationInSec: 3 }
	]
};

export const RemotionRoot: React.FC = () => {
	return (
		<>
			<Composition
				id="HelloWorld"
				component={HelloWorld}
				durationInFrames={150}
				fps={30}
				width={1920}
				height={1080}
			/>
			<Composition
				id="SentinelMain"
				component={DynamicVideo}
				durationInFrames={300}
				fps={30}
				width={1920}
				height={1080}
				defaultProps={{
					manifest: defaultManifest
				}}
			/>
			<Composition
				id="CyberCortex"
				component={CyberCortex}
				durationInFrames={300}
				fps={30}
				width={1920}
				height={1080}
				defaultProps={{
					manifest: defaultManifest
				}}
			/>
			<Composition
				id="LaEspiguita"
				component={LaEspiguita}
				durationInFrames={150}
				fps={30}
				width={1920}
				height={1080}
			/>
		</>
	);
};
