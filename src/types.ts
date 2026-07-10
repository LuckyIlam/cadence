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
  return !Number.isNaN(d.getTime());
}

export function dateNaissanceEstValide(dateIso: string): { valide: boolean; erreur?: string } {
  if (!dateIso) return { valide: false, erreur: "Date requise" };
  const d = new Date(dateIso);
  if (Number.isNaN(d.getTime())) return { valide: false, erreur: "Date invalide" };
  const annee = d.getFullYear();
  if (annee < 1920) return { valide: false, erreur: "La date doit être après 1920" };
  if (d > new Date()) return { valide: false, erreur: "La date ne peut pas être dans le futur" };
  return { valide: true };
}

export interface Activite {
  id: number;
  nom: string;
  description: string | null;
  capacite_max: number | null;
}

export interface CreateActivite {
  nom: string;
  description: string | null;
  capacite_max: number | null;
  annee_scolaire: string | null;
  tarif: number | null;
}

export interface UpdateActivite {
  nom: string;
  description: string | null;
  capacite_max: number | null;
}

export interface CreateTarifActivite {
  activite_id: number;
  annee_scolaire: string;
  tarif: number;
}

export interface CreateLiaisonActivitePersonne {
  activite_id: number;
  personne_id: number;
  annee_scolaire: string;
  role: "encadrant" | "participant";
}

export interface PersonneActivite {
  id: number;
  nom: string;
  prenom: string;
}

export interface DetailActivite {
  activite: Activite;
  tarif: number | null;
  encadrants: PersonneActivite[];
  participants: PersonneActivite[];
}

export interface ActivitePersonne {
  activite: Activite;
  role: string;
}

export interface CreneauActivite {
  id: number;
  activite_id: number;
  jour_semaine: number;
  heure_debut: string;
  heure_fin: string;
  annee_scolaire: string;
}

export interface CreateCreneau {
  activite_id: number;
  jour_semaine: number;
  heure_debut: string;
  heure_fin: string;
  annee_scolaire: string;
}

export interface SemaineBanalisee {
  id: number;
  activite_id: number;
  date_debut: string;
  motif: string | null;
  annee_scolaire: string;
}

export interface CreateSemaineBanalisee {
  activite_id: number;
  date_debut: string;
  motif: string | null;
  annee_scolaire: string;
}

export interface PlanningCreneau {
  creneau: CreneauActivite;
  activite: Activite;
  role: string;
}

const JOURS_SEMAIRE = ["Lundi", "Mardi", "Mercredi", "Jeudi", "Vendredi", "Samedi", "Dimanche"] as const;

export function jourSemaineTexte(jour: number): string {
  return JOURS_SEMAIRE[jour - 1] ?? `Jour ${jour}`;
}

export function getNumeroSemaineISO(date: Date): number {
  const d = new Date(Date.UTC(date.getFullYear(), date.getMonth(), date.getDate()));
  const dayNum = d.getUTCDay() || 7;
  d.setUTCDate(d.getUTCDate() + 4 - dayNum);
  const yearStart = new Date(Date.UTC(d.getUTCFullYear(), 0, 1));
  return Math.ceil(((d.getTime() - yearStart.getTime()) / 86_400_000 + 1) / 7);
}

export function getLundiSemaine(date: Date): Date {
  const d = new Date(date);
  const day = d.getDay();
  const diff = day === 0 ? -6 : 1 - day;
  d.setDate(d.getDate() + diff);
  d.setHours(0, 0, 0, 0);
  return d;
}

export function formatDateISO(date: Date): string {
  return date.toISOString().split("T")[0] ?? "";
}
