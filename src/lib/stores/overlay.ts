import { writable } from "svelte/store";

export interface OverlayState {
	appId: string | null;
}

export const overlayState = writable<OverlayState>({
	appId: null,
});

export function openAppDetails(appId: string) {
	overlayState.set({ appId });
}

export function closeOverlay() {
	overlayState.set({ appId: null });
}

