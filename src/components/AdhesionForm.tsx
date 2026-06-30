import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Adhesion, CreateAdhesion, UpdateAdhesion } from "../types";

interface Props {
  personneId: number;
  adhesion?: Adhesion;
  onClose: () => void;
  onSaved: () => void;
}

export default function AdhesionForm({
  personneId,
  adhesion,
  onClose,
  onSaved,
}: Props) {
  const currentYear = new Date().getFullYear();
  const defaultAnnee = `${currentYear}-${currentYear + 1}`;

  const [anneeScolaire, setAnneeScolaire] = useState(
    adhesion?.annee_scolaire ?? defaultAnnee
  );
  const [reglee, setReglee] = useState(adhesion?.reglee ?? false);
  const [notePaiement, setNotePaiement] = useState(
    adhesion?.note_paiement ?? ""
  );
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");

    if (!anneeScolaire.trim()) {
      setError("L'année scolaire est requise");
      return;
    }

    if (!/^\d{4}-\d{4}$/.test(anneeScolaire)) {
      setError("Format d'année scolaire invalide (ex: 2025-2026)");
      return;
    }

    setLoading(true);
    try {
      if (adhesion) {
        await invoke<Adhesion>("modifier_adhesion", {
          id: adhesion.id,
          input: { reglee, note_paiement: notePaiement || null } as UpdateAdhesion,
        });
      } else {
        await invoke<Adhesion>("creer_adhesion", {
          input: {
            personne_id: personneId,
            annee_scolaire: anneeScolaire.trim(),
            reglee,
            note_paiement: notePaiement || null,
          } as CreateAdhesion,
        });
      }
      onSaved();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="mb-6 p-4 bg-gray-50 border border-gray-200 rounded-lg">
      <form onSubmit={handleSubmit}>
        <h4 className="font-medium text-gray-900 mb-3">
          {adhesion ? "Modifier l'adhésion" : "Nouvelle adhésion"}
        </h4>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Année scolaire *
            </label>
            <input
              type="text"
              value={anneeScolaire}
              onChange={(e) => setAnneeScolaire(e.target.value)}
              placeholder="2025-2026"
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              required
              disabled={!!adhesion}
            />
          </div>
          <div className="flex items-end pb-2">
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={reglee}
                onChange={(e) => setReglee(e.target.checked)}
                className="w-4 h-4 text-blue-600 rounded"
              />
              <span className="text-sm text-gray-700">Réglée</span>
            </label>
          </div>
          <div className="md:col-span-2">
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Note de paiement
            </label>
            <input
              type="text"
              value={notePaiement}
              onChange={(e) => setNotePaiement(e.target.value)}
              placeholder="Chèque n°, virement..."
              maxLength={255}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
        </div>

        {error && (
          <div className="mb-3 p-3 bg-red-50 border border-red-200 rounded-lg text-red-700 text-sm">
            {error}
          </div>
        )}

        <div className="flex justify-end gap-3">
          <button
            type="button"
            onClick={onClose}
            className="px-3 py-1.5 text-sm border border-gray-300 rounded hover:bg-gray-50"
          >
            Annuler
          </button>
          <button
            type="submit"
            disabled={loading}
            className="px-3 py-1.5 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
          >
            {loading ? "Enregistrement..." : "Enregistrer"}
          </button>
        </div>
      </form>
    </div>
  );
}
