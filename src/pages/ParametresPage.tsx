import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import type { ParametresPlanning } from "../types";

export default function ParametresPage() {
  const [parametres, setParametres] = useState<ParametresPlanning | null>(null);
  const [ouverture, setOuverture] = useState("08:00");
  const [fermeture, setFermeture] = useState("20:00");
  const [message, setMessage] = useState<string | null>(null);
  const [erreur, setErreur] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    invoke<ParametresPlanning>("obtenir_parametres_planning")
      .then((p) => {
        setParametres(p);
        setOuverture(p.heure_ouverture);
        setFermeture(p.heure_fermeture);
      })
      .catch(console.error);
  }, []);

  const handleSauvegarder = async () => {
    setSaving(true);
    setMessage(null);
    setErreur(null);
    try {
      const p = await invoke<ParametresPlanning>("modifier_plage_horaire", {
        heureOuverture: ouverture,
        heureFermeture: fermeture,
      });
      setParametres(p);
      setOuverture(p.heure_ouverture);
      setFermeture(p.heure_fermeture);
      setMessage("Plage horaire enregistrée.");
    } catch (e) {
      setErreur(e as string);
    } finally {
      setSaving(false);
    }
  };

  if (!parametres) {
    return <p className="text-gray-500">Chargement...</p>;
  }

  return (
    <div>
      <h2 className="text-2xl font-bold text-gray-900 mb-6">Paramètres</h2>

      <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6 max-w-lg">
        <h3 className="text-lg font-semibold text-gray-900 mb-2">Plage horaire des activités</h3>
        <p className="text-sm text-gray-600 mb-4">
          Les créneaux horaires des activités doivent être compris dans cette plage. Elle est aussi utilisée pour
          l'affichage des plannings hebdomadaires.
        </p>

        <div className="grid grid-cols-2 gap-4 mb-4">
          <div>
            <label htmlFor="heure-ouverture" className="block text-sm font-medium text-gray-700 mb-1">
              Heure d'ouverture
            </label>
            <input
              id="heure-ouverture"
              type="time"
              value={ouverture}
              onChange={(e) => setOuverture(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
            />
          </div>
          <div>
            <label htmlFor="heure-fermeture" className="block text-sm font-medium text-gray-700 mb-1">
              Heure de fermeture
            </label>
            <input
              id="heure-fermeture"
              type="time"
              value={fermeture}
              onChange={(e) => setFermeture(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
            />
          </div>
        </div>

        {message && (
          <div className="bg-green-100 border border-green-300 text-green-700 px-3 py-2 rounded-lg mb-3 text-sm">
            {message}
          </div>
        )}
        {erreur && (
          <div className="bg-red-100 border border-red-300 text-red-700 px-3 py-2 rounded-lg mb-3 text-sm">
            {erreur}
          </div>
        )}

        <button
          type="button"
          onClick={handleSauvegarder}
          disabled={saving}
          className="px-4 py-2 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors disabled:opacity-50"
        >
          {saving ? "Enregistrement..." : "Enregistrer"}
        </button>
      </div>
    </div>
  );
}
