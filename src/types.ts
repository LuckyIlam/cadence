export interface Personne {
  id: number;
  nom: string;
  prenom: string;
  date_naissance: string;
  email: string | null;
  telephone: string | null;
  responsable_id: number | null;
}

export interface CreatePersonne {
  nom: string;
  prenom: string;
  date_naissance: string;
  email: string | null;
  telephone: string | null;
  responsable_id: number | null;
}

export interface UpdatePersonne {
  nom: string;
  prenom: string;
  date_naissance: string;
  email: string | null;
  telephone: string | null;
  responsable_id: number | null;
}

export interface Adhesion {
  id: number;
  personne_id: number;
  annee_scolaire: string;
  reglee: boolean;
  note_paiement: string | null;
}

export interface CreateAdhesion {
  personne_id: number;
  annee_scolaire: string;
  reglee: boolean;
  note_paiement: string | null;
}

export interface UpdateAdhesion {
  reglee: boolean;
  note_paiement: string | null;
}

export interface PersonneDetail {
  personne: Personne;
  adhesions: Adhesion[];
  a_adhesion_annee_cours: boolean;
}

export interface CriteresRecherchePersonnes {
  texte_libre: string | null;
  adherent_uniquement: boolean;
}

export interface Pagination {
  page: number;
  par_page: number;
}

export interface ResultatRecherchePersonnes {
  donnees: Personne[];
  total: number;
  page: number;
  pages: number;
}

export function ageFromDateNaissance(dateNaissance: string): number {
  const today = new Date();
  const birth = new Date(dateNaissance);
  let age = today.getFullYear() - birth.getFullYear();
  const m = today.getMonth() - birth.getMonth();
  if (m < 0 || (m === 0 && today.getDate() < birth.getDate())) {
    age--;
  }
  return age;
}

export function estMineur(dateNaissance: string): boolean {
  return ageFromDateNaissance(dateNaissance) < 18;
}

export function formatDate(dateIso: string): string {
  if (!dateIso) return "";
  const [y, m, d] = dateIso.split("-");
  if (!y || !m || !d) return dateIso;
  return `${d}/${m}/${y}`;
}

export function getCurrentYear(): number {
  return new Date().getFullYear();
}

export function getCurrentAnneeScolaire(): string {
  const y = getCurrentYear();
  const m = new Date().getMonth();
  return m >= 8 ? `${y}-${y + 1}` : `${y - 1}-${y}`;
}

export function estAnneeScolaireValide(annee: string): boolean {
  return /^\d{4}-\d{4}$/.test(annee);
}

export function dateEstValide(dateIso: string): boolean {
  if (!dateIso) return false;
  const d = new Date(dateIso);
  return !isNaN(d.getTime());
}

export function dateNaissanceEstValide(dateIso: string): { valide: boolean; erreur?: string } {
  if (!dateIso) return { valide: false, erreur: "Date requise" };
  const d = new Date(dateIso);
  if (isNaN(d.getTime())) return { valide: false, erreur: "Date invalide" };
  const annee = d.getFullYear();
  if (annee < 1920) return { valide: false, erreur: "La date doit être après 1920" };
  if (d > new Date()) return { valide: false, erreur: "La date ne peut pas être dans le futur" };
  return { valide: true };
}
