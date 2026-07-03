import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import AdhesionForm from "../components/AdhesionForm";
import PersonneForm from "../components/PersonneForm";
import {
  type Adhesion,
  ageFromDateNaissance,
  formatDate,
  getCurrentAnneeScolaire,
  type Personne,
  type PersonneDetail,
} from "../types";

export default function DetailPersonne() {
  const { id } = useParams<{ id: string }>();
  const [personne, setPersonne] = useState<Personne | null>(null);
  const [responsable, setResponsable] = useState<Personne | null>(null);
  const [adhesions, setAdhesions] = useState<Adhesion[]>([]);
  const [aAdhesionEnCours, setAAdhesionEnCours] = useState(false);
  const [showEditForm, setShowEditForm] = useState(false);
  const [showAdhesionForm, setShowAdhesionForm] = useState(false);
  const [editingAdhesion, setEditingAdhesion] = useState<Adhesion | null>(null);

  const anneeEnCours = getCurrentAnneeScolaire();

  const chargerDetail = useCallback(async () => {
    if (!id) return;
    try {
      const detail = await invoke<PersonneDetail>("obtenir_detail_personne", { id: Number(id) });
      setPersonne(detail.personne);
      setAdhesions(detail.adhesions);
      setAAdhesionEnCours(detail.a_adhesion_annee_cours);
      if (detail.personne.responsable_id) {
        const r = await invoke<Personne>("obtenir_personne", {
          id: detail.personne.responsable_id,
        });
        setResponsable(r);
      } else {
        setResponsable(null);
      }
    } catch (e) {
      console.error(e);
    }
  }, [id]);

  useEffect(() => {
    chargerDetail();
  }, [chargerDetail]);

  if (!personne) {
    return <p className="text-gray-500">Chargement...</p>;
  }

  return (
    <div>
      <Link to="/" className="text-blue-600 hover:text-blue-800 mb-4 inline-block">
        &larr; Retour à la liste
      </Link>

      <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6 mb-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-2xl font-bold text-gray-900">
            {personne.nom} {personne.prenom}
          </h2>
          <button
            onClick={() => setShowEditForm(true)}
            className="px-3 py-1 text-sm border border-gray-300 rounded hover:bg-gray-50 transition-colors"
          >
            Modifier
          </button>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
          <div>
            <span className="text-gray-500">Date de naissance :</span>
            <span className="ml-2">{formatDate(personne.date_naissance)}</span>
          </div>
          <div>
            <span className="text-gray-500">Âge :</span>
            <span className="ml-2">{ageFromDateNaissance(personne.date_naissance)} ans</span>
          </div>
          {personne.email && (
            <div>
              <span className="text-gray-500">Email :</span>
              <span className="ml-2">{personne.email}</span>
            </div>
          )}
          {personne.telephone && (
            <div>
              <span className="text-gray-500">Téléphone :</span>
              <span className="ml-2">{personne.telephone}</span>
            </div>
          )}
          {responsable && (
            <div className="md:col-span-2">
              <span className="text-gray-500">Responsable légal :</span>
              <Link to={`/personnes/${responsable.id}`} className="ml-2 text-blue-600 hover:text-blue-800">
                {responsable.nom} {responsable.prenom}
              </Link>
            </div>
          )}
        </div>
      </div>

      {showEditForm && (
        <PersonneForm
          personne={personne}
          onClose={() => setShowEditForm(false)}
          onSaved={() => {
            setShowEditForm(false);
            chargerDetail();
          }}
        />
      )}

      <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-gray-900">Adhésions</h3>
          <div className="relative group">
            <button
              onClick={() => !aAdhesionEnCours && (setEditingAdhesion(null), setShowAdhesionForm(true))}
              disabled={aAdhesionEnCours}
              className="px-3 py-1 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors disabled:bg-gray-400 disabled:cursor-not-allowed"
            >
              Ajouter une adhésion
            </button>
            {aAdhesionEnCours && (
              <div className="absolute bottom-full mb-2 right-0 bg-gray-800 text-white text-xs rounded px-3 py-1.5 whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none">
                Une adhésion existe déjà pour l'année {anneeEnCours}
              </div>
            )}
          </div>
        </div>

        {showAdhesionForm && (
          <AdhesionForm
            personneId={personne.id}
            adhesion={editingAdhesion ?? undefined}
            onClose={() => {
              setShowAdhesionForm(false);
              setEditingAdhesion(null);
            }}
            onSaved={() => {
              setShowAdhesionForm(false);
              setEditingAdhesion(null);
              chargerDetail();
            }}
          />
        )}

        {adhesions.length === 0 ? (
          <p className="text-gray-500 text-center py-4">Aucune adhésion</p>
        ) : (
          <div className="divide-y divide-gray-200">
            {adhesions.map((a) => (
              <div key={a.id} className="py-3 flex items-center justify-between">
                <div>
                  <span className="font-medium">{a.annee_scolaire}</span>
                  <span
                    className={`ml-3 px-2 py-0.5 rounded text-xs ${
                      a.reglee ? "bg-green-100 text-green-800" : "bg-yellow-100 text-yellow-800"
                    }`}
                  >
                    {a.reglee ? "Réglée" : "En attente"}
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  {a.note_paiement && <span className="text-sm text-gray-500">{a.note_paiement}</span>}
                  <button
                    onClick={() => {
                      setEditingAdhesion(a);
                      setShowAdhesionForm(true);
                    }}
                    className="px-2 py-0.5 text-xs border border-gray-300 rounded hover:bg-gray-50 transition-colors"
                  >
                    Modifier
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
