export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export async function checkForUpdate() {
  if (!isTauriRuntime()) return null;
  const { check } = await import("@tauri-apps/plugin-updater");
  return check();
}

export async function installUpdate(): Promise<boolean> {
  if (!isTauriRuntime()) return false;
  const update = await checkForUpdate();
  if (!update) return false;
  await update.downloadAndInstall();
  const { relaunch } = await import("@tauri-apps/plugin-process");
  await relaunch();
  return true;
}
