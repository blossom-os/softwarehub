import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";
import { convertFileSrc } from "@tauri-apps/api/core";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function convertIconPath(iconPath: string | undefined | null): string | undefined {
	if (!iconPath) return undefined;
	if (iconPath.startsWith("data:") || iconPath.startsWith("tauri://") || iconPath.startsWith("http://") || iconPath.startsWith("https://")) {
		return iconPath;
	}
	return iconPath;
}

