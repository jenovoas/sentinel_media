export interface SentinelClip {
    type: 'video' | 'image' | 'text';
    src?: string;
    text?: string;
    durationInSec: number;
}

export interface SentinelManifest {
    title: string;
    description?: string;
    clips: SentinelClip[];
    audioUrl?: string; // Optional background music
}
