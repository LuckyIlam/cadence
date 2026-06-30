import { useState, useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { Personne, ageFromDateNaissance } from "../types";
import PersonneForm from "../components/PersonneForm";

export default function ListePersonnes() {
  const [personnes, setPersonnes] = useState<Personne[]>([]);
  const [recherche, setRecherche] = useState("");
  const [showForm, setShowForm] = useState(false);
  const navigate = useNavigate();

  const chargerPersonnes = useCallback(async () => {
    try {
      const result = await invoke<Personne[]>("lister_personnes");
      setPersonnes(result);
    } catch (e) {
      console.error(e);
    }
  }, []);

  useEffect(() => {
    chargerPersonnes();
  }, [chargerPersonnes]);

  useEffect(() => {
    if (!recherche.trim()) {
      chargerPersonnes();
      return;
    }
    const timer = setTimeout(async () => {
      try {
        const result = await invoke<Personne[]>("rechercher_personnes", {
          query: recherche,
        });
        setPersonnes(result);
      } catch (e) {
        console.error(e);
      }
    }, 300);
    return () => clearTimeout(timer);
  }, [recherche, chargerPersonnes]);

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold text-gray-900">Personnes</h2>
        <button
          onClick={() => setShowForm(true)}
          className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
        >
          Nouvelle personne
        </button>
      </div>

      <div className="mb-6">
        <input
          type="text"
          placeholder="Rechercher par nom ou prénom..."
          value={recherche}
          onChange={(e) => setRecherche(e.target.value)}
          className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
        />
      </div>

      {showForm && (
        <PersonneForm
          onClose={() => setShowForm(false)}
          onSaved={() => {
            setShowForm(false);
            chargerPersonnes();
          }}
        />
      )}

      <div className="grid gap-3">
        {personnes.map((p) => (
          <div
            key={p.id}
            onClick={() => navigate(`/personnes/${p.id}`)}
            className="bg-white rounded-lg shadow-sm border border-gray-200 p-4 hover:shadow-md hover:border-gray-300 transition-all cursor-pointer"
          >
            <div className="flex items-center justify-between">
              <div>
                <span className="font-semibold text-gray-900">
                  {p.nom} {p.prenom}
                </span>
                <span className="text-gray-500 ml-3">
                  {ageFromDateNaissance(p.date_naissance)} ans
                </span>
              </div>
              <div className="text-sm text-gray-500">
                {p.email && <span>{p.email}</span>}
              </div>
            </div>
          </div>
        ))}
        {personnes.length === 0 && (
          <p className="text-center text-gray-500 py-8">
            Aucune personne trouvée
          </p>
        )}
      </div>
    </div>
  );
}
