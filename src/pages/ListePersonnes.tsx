import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import PersonneForm from "../components/PersonneForm";
import {
  ageFromDateNaissance,
  type CriteresRecherchePersonnes,
  type Pagination,
  type ResultatRecherchePersonnes,
} from "../types";

export default function ListePersonnes() {
  const [resultat, setResultat] = useState<ResultatRecherchePersonnes | null>(null);
  const [texteLibre, setTexteLibre] = useState("");
  const [adherentUniquement, setAdherentUniquement] = useState(false);
  const [page, setPage] = useState(1);
  const [showForm, setShowForm] = useState(false);
  const navigate = useNavigate();
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const chargerPersonnes = useCallback(async (criteres: CriteresRecherchePersonnes, pagination: Pagination) => {
    try {
      const r = await invoke<ResultatRecherchePersonnes>("rechercher_personnes", {
        criteres,
        pagination,
      });
      setResultat(r);
    } catch (e) {
      console.error(e);
    }
  }, []);

  useEffect(() => {
    chargerPersonnes({ texte_libre: null, adherent_uniquement: false }, { page: 1, par_page: 20 });
  }, [chargerPersonnes]);

  useEffect(() => {
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => {
      setPage(1);
      chargerPersonnes(
        {
          texte_libre: texteLibre.trim() || null,
          adherent_uniquement: adherentUniquement,
        },
        { page: 1, par_page: 20 },
      );
    }, 300);
    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    };
  }, [texteLibre, adherentUniquement, chargerPersonnes]);

  useEffect(() => {
    chargerPersonnes(
      {
        texte_libre: texteLibre.trim() || null,
        adherent_uniquement: adherentUniquement,
      },
      { page, par_page: 20 },
    );
  }, [page, texteLibre.trim, chargerPersonnes, adherentUniquement]);

  const handleTexteLibreChange = (value: string) => {
    setTexteLibre(value);
  };

  const handleAdherentChange = () => {
    setAdherentUniquement((prev) => !prev);
  };

  const personnes = resultat?.donnees ?? [];
  const total = resultat?.total ?? 0;
  const pages = resultat?.pages ?? 0;

  const numerosPages = () => {
    const nums: number[] = [];
    for (let i = 1; i <= pages; i++) {
      nums.push(i);
    }
    return nums;
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold text-gray-900">Personnes</h2>
        <button
          type="button"
          onClick={() => setShowForm(true)}
          className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
        >
          Nouvelle personne
        </button>
      </div>

      <div className="mb-4 flex gap-4 items-center">
        <input
          type="text"
          placeholder="Rechercher par nom, prénom, email ou téléphone..."
          value={texteLibre}
          onChange={(e) => handleTexteLibreChange(e.target.value)}
          className="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
        />
        <label className="flex items-center gap-2 text-sm text-gray-700 cursor-pointer whitespace-nowrap">
          <input
            type="checkbox"
            checked={adherentUniquement}
            onChange={handleAdherentChange}
            className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
          />
          Adhérent·e·s uniquement
        </label>
      </div>

      {showForm && (
        <PersonneForm
          onClose={() => setShowForm(false)}
          onSaved={() => {
            setShowForm(false);
            chargerPersonnes(
              {
                texte_libre: texteLibre.trim() || null,
                adherent_uniquement: adherentUniquement,
              },
              { page: 1, par_page: 20 },
            );
          }}
        />
      )}

      <div className="grid gap-3">
        {personnes.map((p) => (
          <button
            key={p.id}
            type="button"
            onClick={() => navigate(`/personnes/${p.id}`)}
            className="block w-full text-left bg-white rounded-lg shadow-sm border border-gray-200 p-4 hover:shadow-md hover:border-gray-300 transition-all cursor-pointer"
          >
            <div className="flex items-center justify-between">
              <div>
                <span className="font-semibold text-gray-900">
                  {p.nom} {p.prenom}
                </span>
                <span className="text-gray-500 ml-3">{ageFromDateNaissance(p.date_naissance)} ans</span>
              </div>
              <div className="text-sm text-gray-500">{p.email && <span>{p.email}</span>}</div>
            </div>
          </button>
        ))}
        {personnes.length === 0 && <p className="text-center text-gray-500 py-8">Aucune personne trouvée</p>}
      </div>

      {pages > 1 && (
        <div className="flex items-center justify-center gap-2 mt-6 text-sm">
          <button
            type="button"
            onClick={() => setPage((p) => Math.max(1, p - 1))}
            disabled={page <= 1}
            className="px-3 py-1 rounded border border-gray-300 disabled:opacity-40 disabled:cursor-not-allowed hover:bg-gray-100 transition-colors"
          >
            ← Précédent
          </button>

          {numerosPages().map((n) => (
            <button
              key={n}
              type="button"
              onClick={() => setPage(n)}
              className={`px-3 py-1 rounded border transition-colors ${
                n === page ? "bg-blue-600 text-white border-blue-600" : "border-gray-300 hover:bg-gray-100"
              }`}
            >
              {n}
            </button>
          ))}

          <button
            type="button"
            onClick={() => setPage((p) => Math.min(pages, p + 1))}
            disabled={page >= pages}
            className="px-3 py-1 rounded border border-gray-300 disabled:opacity-40 disabled:cursor-not-allowed hover:bg-gray-100 transition-colors"
          >
            Suivant →
          </button>

          <span className="text-gray-500 ml-2">
            Page {page}/{pages} — {total} résultat{total !== 1 ? "s" : ""}
          </span>
        </div>
      )}
    </div>
  );
}
