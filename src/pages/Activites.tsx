import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import type { Activite } from "../types";

type ActiviteAvecTarif = [Activite, number | null, number];

function getCreationAnnees(): string[] {
  const y = new Date().getFullYear();
  const m = new Date().getMonth();
  const current = m >= 8 ? `${y}-${y + 1}` : `${y - 1}-${y}`;
  const parts = current.split("-");
  if (!parts[0]) return [current];
  const an = Number.parseInt(parts[0], 10);
  return [current, `${an + 1}-${an + 2}`];
}

export default function Activites() {
  const navigate = useNavigate();
  const [anneesDisponibles, setAnneesDisponibles] = useState<string[]>([]);
  const [anneeScolaire, setAnneeScolaire] = useState("");
  const [activites, setActivites] = useState<ActiviteAvecTarif[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [newNom, setNewNom] = useState("");
  const [newDescription, setNewDescription] = useState("");
  const [newCapacite, setNewCapacite] = useState("");
  const creationAnnees = getCreationAnnees();
  const [newAnnee, setNewAnnee] = useState(creationAnnees[0] ?? "");
  const [newTarif, setNewTarif] = useState("");

  useEffect(() => {
    invoke<string[]>("lister_annees_activites")
      .then((annees) => {
        setAnneesDisponibles(annees);
        setAnneeScolaire(annees[0] ?? "");
      })
      .catch(console.error);
  }, []);

  const chargerActivites = useCallback(async (annee: string) => {
    try {
      const r = await invoke<ActiviteAvecTarif[]>("lister_activites", { anneeScolaire: annee });
      setActivites(r);
    } catch (e) {
      console.error(e);
    }
  }, []);

  useEffect(() => {
    if (anneeScolaire) chargerActivites(anneeScolaire);
  }, [anneeScolaire, chargerActivites]);

  const handleCreer = async () => {
    if (!newNom.trim()) return;
    try {
      await invoke("creer_activite", {
        input: {
          nom: newNom.trim(),
          description: newDescription.trim() || null,
          capacite_max: newCapacite ? Number(newCapacite) : null,
          annee_scolaire: newAnnee,
          tarif: newTarif ? Number.parseFloat(newTarif) : null,
        },
      });
      setShowForm(false);
      setNewNom("");
      setNewDescription("");
      setNewCapacite("");
      setNewTarif("");
      if (anneeScolaire !== newAnnee) {
        setAnneeScolaire(newAnnee);
      } else {
        chargerActivites(anneeScolaire);
      }
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold text-gray-900">Activités</h2>
        <button
          type="button"
          onClick={() => setShowForm(true)}
          className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
        >
          Nouvelle activité
        </button>
      </div>

      {anneesDisponibles.length > 0 && (
        <div className="mb-4">
          <select
            value={anneeScolaire}
            onChange={(e) => setAnneeScolaire(e.target.value)}
            className="px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          >
            {anneesDisponibles.map((a) => (
              <option key={a} value={a}>
                Année {a}
              </option>
            ))}
          </select>
        </div>
      )}

      {showForm && (
        <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-4 mb-6">
          <div className="grid gap-3 max-w-md">
            <input
              type="text"
              placeholder="Nom de l'activité"
              value={newNom}
              onChange={(e) => setNewNom(e.target.value)}
              className="px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <input
              type="text"
              placeholder="Description (optionnelle)"
              value={newDescription}
              onChange={(e) => setNewDescription(e.target.value)}
              className="px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <input
              type="number"
              placeholder="Capacité max (optionnelle)"
              value={newCapacite}
              onChange={(e) => setNewCapacite(e.target.value)}
              min="1"
              className="px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <div className="grid grid-cols-2 gap-3">
              <select
                value={newAnnee}
                onChange={(e) => setNewAnnee(e.target.value)}
                className="px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                {getCreationAnnees().map((a) => (
                  <option key={a} value={a}>
                    Année {a}
                  </option>
                ))}
              </select>
              <input
                type="number"
                step="0.01"
                placeholder="Tarif (€, optionnel)"
                value={newTarif}
                onChange={(e) => setNewTarif(e.target.value)}
                min="0"
                className="px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <div className="flex gap-2">
              <button
                type="button"
                onClick={handleCreer}
                className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
              >
                Créer
              </button>
              <button
                type="button"
                onClick={() => setShowForm(false)}
                className="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors"
              >
                Annuler
              </button>
            </div>
          </div>
        </div>
      )}

      <div className="grid gap-3">
        {activites.map((a) => (
          <button
            key={a[0].id}
            type="button"
            onClick={() => navigate(`/activites/${a[0].id}?annee=${anneeScolaire}`)}
            className="block w-full text-left bg-white rounded-lg shadow-sm border border-gray-200 p-4 hover:shadow-md hover:border-gray-300 transition-all cursor-pointer"
          >
            <div className="flex items-center justify-between">
              <div>
                <span className="font-semibold text-gray-900">{a[0].nom}</span>
                {a[1] !== null && <span className="text-gray-500 ml-3">{a[1].toFixed(2)} €</span>}
              </div>
              <div className="text-sm text-gray-500">
                {a[2]} participant{a[2] !== 1 ? "s" : ""}
                {a[0].capacite_max !== null && ` / ${a[0].capacite_max}`}
              </div>
            </div>
            {a[0].description && <div className="text-sm text-gray-500 mt-1">{a[0].description}</div>}
          </button>
        ))}
        {activites.length === 0 && (
          <p className="text-center text-gray-500 py-8">Aucune activité pour l'année {anneeScolaire}</p>
        )}
      </div>
    </div>
  );
}
