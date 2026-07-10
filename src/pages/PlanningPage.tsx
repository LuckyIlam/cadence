import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import PlanningHebdo from "../components/PlanningHebdo";
import { formatDateISO, getCurrentAnneeScolaire, getLundiSemaine, type Personne, type PlanningCreneau } from "../types";

export default function PlanningPage() {
  const { personneId } = useParams<{ personneId: string }>();
  const [personnes, setPersonnes] = useState<Personne[]>([]);
  const [selectedPersonneId, setSelectedPersonneId] = useState<number | null>(null);
  const [creneaux, setCreneaux] = useState<PlanningCreneau[]>([]);
  const [anneeScolaire, setAnneeScolaire] = useState(getCurrentAnneeScolaire());
  const [dateLundi, setDateLundi] = useState(() => getLundiSemaine(new Date()));
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const chargerPersonnes = async () => {
      try {
        const r = await invoke<{ donnees: Personne[] }>("rechercher_personnes", {
          criteres: { texte_libre: null, adherent_uniquement: false },
          pagination: { page: 1, par_page: 500 },
        });
        setPersonnes(r.donnees);
        if (personneId) {
          setSelectedPersonneId(Number(personneId));
        } else if (r.donnees.length > 0) {
          setSelectedPersonneId(r.donnees[0]?.id ?? null);
        }
      } catch (e) {
        console.error(e);
      }
    };
    chargerPersonnes();
  }, [personneId]);

  const chargerPlanning = useCallback(async () => {
    if (!selectedPersonneId) return;
    setLoading(true);
    setError(null);
    try {
      const r = await invoke<PlanningCreneau[]>("planning_personne", {
        personneId: selectedPersonneId,
        dateLundi: formatDateISO(dateLundi),
        anneeScolaire: anneeScolaire,
      });
      setCreneaux(r);
    } catch (e) {
      setError(e as string);
    } finally {
      setLoading(false);
    }
  }, [selectedPersonneId, dateLundi, anneeScolaire]);

  useEffect(() => {
    chargerPlanning();
  }, [chargerPlanning]);

  const handleSemainePrecedente = () => {
    const d = new Date(dateLundi);
    d.setDate(d.getDate() - 7);
    setDateLundi(d);
  };

  const handleSemaineSuivante = () => {
    const d = new Date(dateLundi);
    d.setDate(d.getDate() + 7);
    setDateLundi(d);
  };

  return (
    <div>
      <h2 className="text-2xl font-bold text-gray-900 mb-6">Planning</h2>

      <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-4 mb-6">
        <div className="flex items-center gap-4">
          <div className="flex-1">
            <label htmlFor="select-personne" className="block text-sm font-medium text-gray-700 mb-1">
              Personne
            </label>
            <select
              id="select-personne"
              value={selectedPersonneId ?? ""}
              onChange={(e) => setSelectedPersonneId(Number(e.target.value))}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
            >
              {personnes.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.nom} {p.prenom}
                </option>
              ))}
            </select>
          </div>
          <div>
            <label htmlFor="select-annee" className="block text-sm font-medium text-gray-700 mb-1">
              Année scolaire
            </label>
            <select
              id="select-annee"
              value={anneeScolaire}
              onChange={(e) => setAnneeScolaire(e.target.value)}
              className="px-3 py-2 border border-gray-300 rounded-lg text-sm"
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
        </div>
      </div>

      {error && <div className="bg-red-100 border border-red-300 text-red-700 px-4 py-3 rounded-lg mb-6">{error}</div>}

      {loading ? (
        <p className="text-gray-500 text-center py-8">Chargement...</p>
      ) : (
        <PlanningHebdo
          creneaux={creneaux}
          dateLundi={dateLundi}
          onSemainePrecedente={handleSemainePrecedente}
          onSemaineSuivante={handleSemaineSuivante}
        />
      )}
    </div>
  );
}
