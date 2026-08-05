import { invoke } from "@tauri-apps/api/core";

let cache: string | null | undefined;

async function charger(): Promise<string> {
  const config = await invoke<{ utilisateur: string | null }>("obtenir_config");
  cache = config.utilisateur ?? "";
  return cache ?? "";
}

export async function utilisateurCourant(): Promise<string> {
  if (cache !== undefined) return cache ?? "";
  return charger();
}
