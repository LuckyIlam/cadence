import { invoke } from "@tauri-apps/api/core";

let cache: string | null | undefined;
const auditeurs = new Set<() => void>();

async function charger(): Promise<string> {
  const config = await invoke<{ utilisateur: string | null }>("obtenir_config");
  cache = config.utilisateur ?? "";
  return cache ?? "";
}

export async function utilisateurCourant(): Promise<string> {
  if (cache !== undefined) return cache ?? "";
  return charger();
}

export function invaliderUtilisateur(): void {
  cache = undefined;
  for (const fn of auditeurs) {
    fn();
  }
}

export function abonnerUtilisateur(fn: () => void): () => void {
  auditeurs.add(fn);
  return () => auditeurs.delete(fn);
}
