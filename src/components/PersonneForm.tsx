import { invoke } from "@tauri-apps/api/core";
import { useEffect, useMemo, useRef, useState } from "react";
import {
  ageFromDateNaissance,
  type CreatePersonne,
  dateNaissanceEstValide,
  estMineur,
  type Personne,
  type ResultatRecherchePersonnes,
  type UpdatePersonne,
} from "../types";

interface Props {
  personne?: Personne;
  onClose: () => void;
  onSaved: () => void;
}

export default function PersonneForm({ personne, onClose, onSaved }: Props) {
  const [nom, setNom] = useState(personne?.nom ?? "");
  const [prenom, setPrenom] = useState(personne?.prenom ?? "");
  const [dateNaissance, setDateNaissance] = useState(personne?.date_naissance ?? "");
  const [email, setEmail] = useState(personne?.email ?? "");
  const [telephone, setTelephone] = useState(personne?.telephone ?? "");
  const [responsableId, setResponsableId] = useState<number | null>(personne?.responsable_id ?? null);
  const [responsableNom, setResponsableNom] = useState("");
  const [toutLeMonde, setToutLeMonde] = useState<Personne[]>([]);
  const [rechercheResponsable, setRechercheResponsable] = useState("");
  const [dropdownOuvert, setDropdownOuvert] = useState(false);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    invoke<ResultatRecherchePersonnes>("rechercher_personnes", {
      criteres: { texte_libre: null, adherent_uniquement: false },
      pagination: { page: 1, par_page: 0 },
    })
      .then((r) => setToutLeMonde(r.donnees))
      .catch(() => {});
  }, []);

  useEffect(() => {
    if (!responsableId || toutLeMonde.length === 0) return;
    const r = toutLeMonde.find((p) => p.id === responsableId);
    if (r) setResponsableNom(`${r.prenom} ${r.nom}`);
  }, [responsableId, toutLeMonde]);

  useEffect(() => {
    if (!dropdownOuvert) return;
    const handler = (e: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setDropdownOuvert(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [dropdownOuvert]);

  const majeurs = useMemo(
    () => toutLeMonde.filter((p) => !estMineur(p.date_naissance) && p.id !== personne?.id),
    [toutLeMonde, personne],
  );

  const majeursFiltres = useMemo(() => {
    if (!rechercheResponsable.trim()) return majeurs;
    const q = rechercheResponsable.toLowerCase();
    return majeurs.filter((p) => p.nom.toLowerCase().includes(q) || p.prenom.toLowerCase().includes(q));
  }, [majeurs, rechercheResponsable]);

  const isMineur = dateNaissance && estMineur(dateNaissance);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");

    if (!nom.trim() || !prenom.trim() || !dateNaissance) {
      setError("Nom, prénom et date de naissance sont requis");
      return;
    }

    const dateCheck = dateNaissanceEstValide(dateNaissance);
    if (!dateCheck.valide) {
      setError(dateCheck.erreur ?? "Date invalide");
      return;
    }

    if (isMineur && !responsableId) {
      setError("Un mineur doit avoir un responsable légal");
      return;
    }

    setLoading(true);
    try {
      const input = {
        nom: nom.trim(),
        prenom: prenom.trim(),
        date_naissance: dateNaissance,
        email: email.trim() || null,
        telephone: telephone.trim() || null,
        responsable_id: responsableId,
      };

      if (personne) {
        await invoke<Personne>("modifier_personne", {
          id: personne.id,
          input: input as UpdatePersonne,
        });
      } else {
        await invoke<Personne>("creer_personne", {
          input: input as CreatePersonne,
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
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <form
        onSubmit={handleSubmit}
        className="bg-white rounded-xl shadow-xl p-6 w-full max-w-lg mx-4 max-h-[90vh] overflow-y-auto"
      >
        <h3 className="text-lg font-semibold mb-4">{personne ? "Modifier la personne" : "Nouvelle personne"}</h3>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Nom *</label>
            <input
              type="text"
              value={nom}
              onChange={(e) => setNom(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              required
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Prénom *</label>
            <input
              type="text"
              value={prenom}
              onChange={(e) => setPrenom(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              required
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Date de naissance *</label>
            <input
              type="date"
              value={dateNaissance}
              onChange={(e) => setDateNaissance(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              required
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Téléphone</label>
            <input
              type="tel"
              value={telephone}
              onChange={(e) => setTelephone(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
          <div className="md:col-span-2">
            <label className="block text-sm font-medium text-gray-700 mb-1">Email</label>
            <input
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
          <div className="md:col-span-2 relative" ref={dropdownRef}>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              {isMineur ? "Responsable légal * (mineur obligatoire)" : "Responsable légal"}
            </label>
            <input
              type="text"
              value={responsableNom || rechercheResponsable}
              onChange={(e) => {
                setRechercheResponsable(e.target.value);
                setResponsableNom("");
                setResponsableId(null);
                setDropdownOuvert(true);
              }}
              onFocus={() => setDropdownOuvert(true)}
              placeholder={isMineur ? "Chercher un responsable..." : "Laisser vide si majeur"}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            {dropdownOuvert && (
              <div className="absolute z-10 mt-1 w-full bg-white border border-gray-300 rounded-lg shadow-lg max-h-48 overflow-y-auto">
                {majeursFiltres.length === 0 ? (
                  <p className="px-3 py-2 text-sm text-gray-500">Aucun majeur trouvé</p>
                ) : (
                  majeursFiltres.map((p) => (
                    <button
                      key={p.id}
                      type="button"
                      onClick={() => {
                        setResponsableId(p.id);
                        setResponsableNom(`${p.prenom} ${p.nom}`);
                        setRechercheResponsable("");
                        setDropdownOuvert(false);
                      }}
                      className="w-full text-left px-3 py-2 text-sm hover:bg-blue-50 transition-colors"
                    >
                      {p.prenom} {p.nom}
                      <span className="text-gray-400 ml-2">{ageFromDateNaissance(p.date_naissance)} ans</span>
                    </button>
                  ))
                )}
              </div>
            )}
          </div>
        </div>

        {error && (
          <div className="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg text-red-700 text-sm">{error}</div>
        )}

        <div className="flex justify-end gap-3">
          <button
            type="button"
            onClick={onClose}
            className="px-4 py-2 text-sm border border-gray-300 rounded-lg hover:bg-gray-50"
          >
            Annuler
          </button>
          <button
            type="submit"
            disabled={loading}
            className="px-4 py-2 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
          >
            {loading ? "Enregistrement..." : "Enregistrer"}
          </button>
        </div>
      </form>
    </div>
  );
}
