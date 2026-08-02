import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import { Link, useParams, useSearchParams } from "react-router-dom";
import {
  type Activite,
  type CreateCreneau,
  type CreateSemaineBanalisee,
  type CreneauActivite,
  formatDate,
  getCurrentAnneeScolaire,
  type PersonneActivite,
  type SemaineBanalisee,
} from "../types";

export default function DetailActivite() {
  const { id } = useParams<{ id: string }>();
  const [searchParams] = useSearchParams();
  const anneeUrl = searchParams.get("annee") || getCurrentAnneeScolaire();

  const [activite, setActivite] = useState<Activite | null>(null);
  const [tarif, setTarif] = useState<number | null>(null);
  const [encadrants, setEncadrants] = useState<PersonneActivite[]>([]);
  const [participants, setParticipants] = useState<PersonneActivite[]>([]);
  const [anneeScolaire, setAnneeScolaire] = useState(anneeUrl);

  const [editTarif, setEditTarif] = useState(false);
  const [newTarif, setNewTarif] = useState("");

  const [editNom, setEditNom] = useState(false);
  const [editDescription, setEditDescription] = useState(false);
  const [editCapacite, setEditCapacite] = useState(false);
  const [formNom, setFormNom] = useState("");
  const [formDescription, setFormDescription] = useState("");
  const [formCapacite, setFormCapacite] = useState("");

  const [creneaux, setCreneaux] = useState<CreneauActivite[]>([]);
  const [semainesBanalisees, setSemainesBanalisees] = useState<SemaineBanalisee[]>([]);
  const [nbInscrits, setNbInscrits] = useState(0);
  const [showCreneauForm, setShowCreneauForm] = useState(false);
  const [newCreneauJour, setNewCreneauJour] = useState(1);
  const [newCreneauDebut, setNewCreneauDebut] = useState("08:00");
  const [newCreneauFin, setNewCreneauFin] = useState("10:00");
  const [showSemaineForm, setShowSemaineForm] = useState(false);
  const [newSemaineDate, setNewSemaineDate] = useState("");
  const [newSemaineMotif, setNewSemaineMotif] = useState("");

  const [showAddPersonne, setShowAddPersonne] = useState(false);
  const [searchTexte, setSearchTexte] = useState("");
  const [searchResults, setSearchResults] = useState<Array<{ id: number; nom: string; prenom: string }>>([]);
  const [addTarget, setAddTarget] = useState<"encadrant" | "participant">("participant");

  const chargerDetail = useCallback(async () => {
    if (!id) return;
    try {
      const detail = await invoke<{
        activite: Activite;
        tarif: number | null;
        encadrants: PersonneActivite[];
        participants: PersonneActivite[];
      }>("obtenir_detail_activite", { id: Number(id), anneeScolaire });
      setActivite(detail.activite);
      setTarif(detail.tarif);
      setEncadrants(detail.encadrants);
      setParticipants(detail.participants);
    } catch (e) {
      console.error(e);
    }
  }, [id, anneeScolaire]);

  useEffect(() => {
    chargerDetail();
  }, [chargerDetail]);

  useEffect(() => {
    if (!id) return;
    invoke<CreneauActivite[]>("lister_creneaux", {
      activiteId: Number(id),
      anneeScolaire: anneeScolaire,
    })
      .then(setCreneaux)
      .catch(console.error);
    invoke<SemaineBanalisee[]>("lister_semaines_banalisees", {
      activiteId: Number(id),
    })
      .then(setSemainesBanalisees)
      .catch(console.error);
  }, [id, anneeScolaire]);

  useEffect(() => {
    setNbInscrits(encadrants.length + participants.length);
  }, [encadrants, participants]);

  const handleSaveTarif = async () => {
    if (!id) return;
    try {
      await invoke("definir_tarif_activite", {
        input: {
          activite_id: Number(id),
          annee_scolaire: anneeScolaire,
          tarif: Number.parseFloat(newTarif),
        },
      });
      setEditTarif(false);
      chargerDetail();
    } catch (e) {
      console.error(e);
    }
  };

  const handleEditActivite = async (field: string, value: string) => {
    if (!id || !activite) return;
    try {
      const input: { nom: string; description: string | null; capacite_max: number | null } = {
        nom: activite.nom,
        description: activite.description,
        capacite_max: activite.capacite_max,
      };
      if (field === "nom") input.nom = value;
      if (field === "description") input.description = value || null;
      if (field === "capacite_max") input.capacite_max = value ? Number(value) : null;

      const updated = await invoke<Activite>("modifier_activite", { id: Number(id), input });
      setActivite(updated);
      setEditNom(false);
      setEditDescription(false);
      setEditCapacite(false);
    } catch (e) {
      console.error(e);
    }
  };

  const handleSearch = useCallback(async (texte: string) => {
    if (!texte.trim()) {
      setSearchResults([]);
      return;
    }
    try {
      const r = await invoke<{ donnees: Array<{ id: number; nom: string; prenom: string }> }>("rechercher_personnes", {
        criteres: { texte_libre: texte, adherent_uniquement: false },
        pagination: { page: 1, par_page: 20 },
      });
      setSearchResults(r.donnees);
    } catch (e) {
      console.error(e);
    }
  }, []);

  useEffect(() => {
    const t = setTimeout(() => handleSearch(searchTexte), 300);
    return () => clearTimeout(t);
  }, [searchTexte, handleSearch]);

  const handleAjouterPersonne = async (personneId: number) => {
    if (!id) return;
    try {
      await invoke("ajouter_personne_activite", {
        input: {
          activite_id: Number(id),
          personne_id: personneId,
          annee_scolaire: anneeScolaire,
          role: addTarget,
        },
      });
      setShowAddPersonne(false);
      setSearchTexte("");
      setSearchResults([]);
      chargerDetail();
    } catch (e) {
      const msg = e as string;
      alert(msg);
    }
  };

  const handleAjouterCreneau = async () => {
    if (!id) return;
    try {
      const input: CreateCreneau = {
        activite_id: Number(id),
        jour_semaine: newCreneauJour,
        heure_debut: newCreneauDebut,
        heure_fin: newCreneauFin,
        annee_scolaire: anneeScolaire,
      };
      await invoke("ajouter_creneau", { input });
      setShowCreneauForm(false);
      setNewCreneauJour(1);
      setNewCreneauDebut("08:00");
      setNewCreneauFin("10:00");
      const c = await invoke<CreneauActivite[]>("lister_creneaux", {
        activiteId: Number(id),
        anneeScolaire,
      });
      setCreneaux(c);
    } catch (e) {
      alert(e as string);
    }
  };

  const handleSupprimerCreneau = async (creneauId: number) => {
    if (!id) return;
    try {
      await invoke("supprimer_creneau", {
        id: creneauId,
        activiteId: Number(id),
        anneeScolaire,
      });
      const c = await invoke<CreneauActivite[]>("lister_creneaux", {
        activiteId: Number(id),
        anneeScolaire,
      });
      setCreneaux(c);
    } catch (e) {
      alert(e as string);
    }
  };

  const handleAjouterSemaine = async () => {
    if (!id) return;
    try {
      const input: CreateSemaineBanalisee = {
        activite_id: Number(id),
        date_debut: newSemaineDate,
        motif: newSemaineMotif || null,
        annee_scolaire: anneeScolaire,
      };
      await invoke("ajouter_semaine_banalisee", { input });
      setShowSemaineForm(false);
      setNewSemaineDate("");
      setNewSemaineMotif("");
      const sb = await invoke<SemaineBanalisee[]>("lister_semaines_banalisees", {
        activiteId: Number(id),
      });
      setSemainesBanalisees(sb);
    } catch (e) {
      alert(e as string);
    }
  };

  const handleSupprimerSemaine = async (semaineId: number) => {
    if (!id) return;
    try {
      await invoke("supprimer_semaine_banalisee", { id: semaineId });
      const sb = await invoke<SemaineBanalisee[]>("lister_semaines_banalisees", {
        activiteId: Number(id),
      });
      setSemainesBanalisees(sb);
    } catch (e) {
      alert(e as string);
    }
  };

  const handleRetirerPersonne = async (personneId: number) => {
    if (!id) return;
    try {
      await invoke("retirer_personne_activite", {
        activiteId: Number(id),
        personneId: personneId,
        anneeScolaire: anneeScolaire,
      });
      chargerDetail();
    } catch (e) {
      console.error(e);
    }
  };

  if (!activite) {
    return <p className="text-gray-500">Chargement...</p>;
  }

  return (
    <div>
      <Link to="/activites" className="text-blue-600 hover:text-blue-800 mb-4 inline-block">
        &larr; Retour aux activités
      </Link>

      <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6 mb-6">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-2">
            {editNom ? (
              <input
                type="text"
                value={formNom}
                onChange={(e) => setFormNom(e.target.value)}
                onBlur={() => handleEditActivite("nom", formNom)}
                onKeyDown={(e) => e.key === "Enter" && handleEditActivite("nom", formNom)}
                className="text-2xl font-bold px-2 py-1 border border-gray-300 rounded"
              />
            ) : (
              <button
                type="button"
                className="text-2xl font-bold text-gray-900 hover:text-blue-600 text-left"
                onClick={() => {
                  setFormNom(activite.nom);
                  setEditNom(true);
                }}
              >
                {activite.nom}
              </button>
            )}
          </div>
          <select
            value={anneeScolaire}
            onChange={(e) => setAnneeScolaire(e.target.value)}
            className="px-3 py-1 text-sm border border-gray-300 rounded-lg"
          >
            {(() => {
              const debut = anneeScolaire.split("-")[0];
              if (!debut) return null;
              const an = Number.parseInt(debut, 10);
              return Array.from({ length: 4 }, (_, i) => an - 2 + i).map((a) => (
                <option key={a} value={`${a}-${a + 1}`}>
                  {`${a}-${a + 1}`}
                </option>
              ));
            })()}
          </select>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm mb-4">
          <div>
            <span className="text-gray-500">Description :</span>
            {editDescription ? (
              <input
                type="text"
                value={formDescription}
                onChange={(e) => setFormDescription(e.target.value)}
                onBlur={() => handleEditActivite("description", formDescription)}
                onKeyDown={(e) => e.key === "Enter" && handleEditActivite("description", formDescription)}
                className="ml-2 px-2 py-1 border border-gray-300 rounded"
              />
            ) : (
              <button
                type="button"
                className="ml-2 hover:text-blue-600 text-left"
                onClick={() => {
                  setFormDescription(activite.description || "");
                  setEditDescription(true);
                }}
              >
                {activite.description || "—"}
              </button>
            )}
          </div>
          <div>
            <span className="text-gray-500">Capacité max :</span>
            {editCapacite ? (
              <input
                type="number"
                value={formCapacite}
                onChange={(e) => setFormCapacite(e.target.value)}
                onBlur={() => handleEditActivite("capacite_max", formCapacite)}
                onKeyDown={(e) => e.key === "Enter" && handleEditActivite("capacite_max", formCapacite)}
                className="ml-2 px-2 py-1 border border-gray-300 rounded w-20"
              />
            ) : (
              <button
                type="button"
                className="ml-2 hover:text-blue-600 text-left"
                onClick={() => {
                  setFormCapacite(activite.capacite_max?.toString() || "");
                  setEditCapacite(true);
                }}
              >
                {activite.capacite_max !== null ? activite.capacite_max : "—"}
              </button>
            )}
          </div>
          <div>
            <span className="text-gray-500">Tarif ({anneeScolaire}) :</span>
            {editTarif ? (
              <span className="ml-2">
                <input
                  type="number"
                  step="0.01"
                  value={newTarif}
                  onChange={(e) => setNewTarif(e.target.value)}
                  className="px-2 py-1 border border-gray-300 rounded w-24"
                />
                <button
                  type="button"
                  onClick={handleSaveTarif}
                  className="ml-2 px-2 py-1 text-xs bg-blue-600 text-white rounded"
                >
                  OK
                </button>
                <button
                  type="button"
                  onClick={() => setEditTarif(false)}
                  className="ml-1 px-2 py-1 text-xs border border-gray-300 rounded"
                >
                  Annuler
                </button>
              </span>
            ) : (
              <button
                type="button"
                className="ml-2 hover:text-blue-600 text-left"
                onClick={() => {
                  setNewTarif(tarif?.toString() || "");
                  setEditTarif(true);
                }}
              >
                {tarif !== null ? `${tarif.toFixed(2)} €` : "Non défini"}
              </button>
            )}
          </div>
        </div>
      </div>

      <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6 mb-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-gray-900">Créneaux hebdomadaires</h3>
          <button
            type="button"
            onClick={() => setShowCreneauForm(!showCreneauForm)}
            className="px-3 py-1 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
          >
            Ajouter un créneau
          </button>
        </div>

        {nbInscrits > 0 && (
          <p className="text-sm text-amber-700 bg-amber-50 border border-amber-200 rounded px-3 py-2 mb-3">
            Des personnes sont inscrites à cette activité pour {anneeScolaire}. Les créneaux existants sont verrouillés
            (modification et suppression impossible).
          </p>
        )}

        {showCreneauForm && (
          <div className="mb-4 p-3 bg-gray-50 rounded-lg border border-gray-200">
            <div className="grid grid-cols-4 gap-2 mb-2">
              <select
                value={newCreneauJour}
                onChange={(e) => setNewCreneauJour(Number(e.target.value))}
                className="px-2 py-1 border border-gray-300 rounded text-sm"
              >
                {[1, 2, 3, 4, 5, 6, 7].map((j) => (
                  <option key={j} value={j}>
                    {["Lundi", "Mardi", "Mercredi", "Jeudi", "Vendredi", "Samedi", "Dimanche"][j - 1]}
                  </option>
                ))}
              </select>
              <input
                type="time"
                value={newCreneauDebut}
                onChange={(e) => setNewCreneauDebut(e.target.value)}
                className="px-2 py-1 border border-gray-300 rounded text-sm"
              />
              <input
                type="time"
                value={newCreneauFin}
                onChange={(e) => setNewCreneauFin(e.target.value)}
                className="px-2 py-1 border border-gray-300 rounded text-sm"
              />
              <div className="flex gap-1">
                <button
                  type="button"
                  onClick={handleAjouterCreneau}
                  className="px-3 py-1 text-sm bg-green-600 text-white rounded hover:bg-green-700 transition-colors"
                >
                  OK
                </button>
                <button
                  type="button"
                  onClick={() => setShowCreneauForm(false)}
                  className="px-3 py-1 text-sm border border-gray-300 rounded hover:bg-gray-50 transition-colors"
                >
                  Annuler
                </button>
              </div>
            </div>
          </div>
        )}

        {creneaux.length === 0 ? (
          <p className="text-gray-500 text-center py-4">Aucun créneau défini pour {anneeScolaire}</p>
        ) : (
          <div className="divide-y divide-gray-200">
            {creneaux.map((c) => (
              <div key={c.id} className="py-2 flex items-center justify-between">
                <span className="text-sm">
                  {["Lundi", "Mardi", "Mercredi", "Jeudi", "Vendredi", "Samedi", "Dimanche"][c.jour_semaine - 1]}{" "}
                  {c.heure_debut}–{c.heure_fin}
                </span>
                <button
                  type="button"
                  onClick={() => handleSupprimerCreneau(c.id)}
                  disabled={nbInscrits > 0}
                  className="text-xs text-red-600 hover:text-red-800 disabled:text-gray-400 disabled:cursor-not-allowed"
                >
                  Supprimer
                </button>
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6 mb-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-gray-900">Semaines banalisées</h3>
          <button
            type="button"
            onClick={() => setShowSemaineForm(!showSemaineForm)}
            className="px-3 py-1 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
          >
            Ajouter une semaine
          </button>
        </div>

        {showSemaineForm && (
          <div className="mb-4 p-3 bg-gray-50 rounded-lg border border-gray-200">
            <div className="grid grid-cols-3 gap-2 mb-2">
              <input
                type="date"
                value={newSemaineDate}
                onChange={(e) => setNewSemaineDate(e.target.value)}
                className="px-2 py-1 border border-gray-300 rounded text-sm"
              />
              <input
                type="text"
                value={newSemaineMotif}
                onChange={(e) => setNewSemaineMotif(e.target.value)}
                placeholder="Motif (optionnel)"
                className="px-2 py-1 border border-gray-300 rounded text-sm"
              />
              <div className="flex gap-1">
                <button
                  type="button"
                  onClick={handleAjouterSemaine}
                  className="px-3 py-1 text-sm bg-green-600 text-white rounded hover:bg-green-700 transition-colors"
                >
                  OK
                </button>
                <button
                  type="button"
                  onClick={() => setShowSemaineForm(false)}
                  className="px-3 py-1 text-sm border border-gray-300 rounded hover:bg-gray-50 transition-colors"
                >
                  Annuler
                </button>
              </div>
            </div>
          </div>
        )}

        {semainesBanalisees.length === 0 ? (
          <p className="text-gray-500 text-center py-4">Aucune semaine banalisée</p>
        ) : (
          <div className="divide-y divide-gray-200">
            {semainesBanalisees.map((sb) => (
              <div key={sb.id} className="py-2 flex items-center justify-between">
                <span className="text-sm">
                  {formatDate(sb.date_debut)}
                  {sb.motif && <span className="text-gray-500 ml-2">— {sb.motif}</span>}
                </span>
                <button
                  type="button"
                  onClick={() => handleSupprimerSemaine(sb.id)}
                  className="text-xs text-red-600 hover:text-red-800"
                >
                  Supprimer
                </button>
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-semibold text-gray-900">Encadrants</h3>
            <button
              type="button"
              onClick={() => {
                const same = addTarget === "encadrant" && showAddPersonne;
                setAddTarget("encadrant");
                setShowAddPersonne(!same);
              }}
              className="px-3 py-1 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
            >
              Ajouter
            </button>
          </div>

          {showAddPersonne && addTarget === "encadrant" && (
            <AjoutPersonnePanel
              searchTexte={searchTexte}
              onSearchTexteChange={setSearchTexte}
              searchResults={searchResults}
              onSelect={handleAjouterPersonne}
              onCancel={() => {
                setShowAddPersonne(false);
                setSearchTexte("");
                setSearchResults([]);
              }}
            />
          )}

          {encadrants.length === 0 ? (
            <p className="text-gray-500 text-center py-4">Aucun encadrant</p>
          ) : (
            <div className="divide-y divide-gray-200">
              {encadrants.map((p) => (
                <div key={p.id} className="py-2 flex items-center justify-between">
                  <Link to={`/personnes/${p.id}`} className="text-blue-600 hover:text-blue-800">
                    {p.nom} {p.prenom}
                  </Link>
                  <button
                    type="button"
                    onClick={() => handleRetirerPersonne(p.id)}
                    className="text-xs text-red-600 hover:text-red-800"
                  >
                    Retirer
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-semibold text-gray-900">Participants</h3>
            <button
              type="button"
              onClick={() => {
                const same = addTarget === "participant" && showAddPersonne;
                setAddTarget("participant");
                setShowAddPersonne(!same);
              }}
              className="px-3 py-1 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
            >
              Ajouter
            </button>
          </div>

          {showAddPersonne && addTarget === "participant" && (
            <AjoutPersonnePanel
              searchTexte={searchTexte}
              onSearchTexteChange={setSearchTexte}
              searchResults={searchResults}
              onSelect={handleAjouterPersonne}
              onCancel={() => {
                setShowAddPersonne(false);
                setSearchTexte("");
                setSearchResults([]);
              }}
            />
          )}

          {participants.length === 0 ? (
            <p className="text-gray-500 text-center py-4">Aucun participant</p>
          ) : (
            <div className="divide-y divide-gray-200">
              {participants.map((p) => (
                <div key={p.id} className="py-2 flex items-center justify-between">
                  <Link to={`/personnes/${p.id}`} className="text-blue-600 hover:text-blue-800">
                    {p.nom} {p.prenom}
                  </Link>
                  <button
                    type="button"
                    onClick={() => handleRetirerPersonne(p.id)}
                    className="text-xs text-red-600 hover:text-red-800"
                  >
                    Retirer
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function AjoutPersonnePanel({
  searchTexte,
  onSearchTexteChange,
  searchResults,
  onSelect,
  onCancel,
}: {
  searchTexte: string;
  onSearchTexteChange: (v: string) => void;
  searchResults: Array<{ id: number; nom: string; prenom: string }>;
  onSelect: (id: number) => void;
  onCancel: () => void;
}) {
  return (
    <div className="mb-4 p-3 bg-gray-50 rounded-lg border border-gray-200">
      <div className="flex items-center justify-between mb-2">
        <input
          type="text"
          placeholder="Rechercher une personne..."
          value={searchTexte}
          onChange={(e) => onSearchTexteChange(e.target.value)}
          className="flex-1 px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
        <button type="button" onClick={onCancel} className="ml-2 px-2 py-1 text-xs text-gray-600 hover:text-gray-900">
          Annuler
        </button>
      </div>
      {searchResults.length > 0 && (
        <div className="max-h-40 overflow-y-auto divide-y divide-gray-200 border border-gray-200 rounded-lg bg-white">
          {searchResults.map((p) => (
            <button
              key={p.id}
              type="button"
              onClick={() => onSelect(p.id)}
              className="w-full text-left px-3 py-2 text-sm hover:bg-gray-100 transition-colors"
            >
              {p.nom} {p.prenom}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
