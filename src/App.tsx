import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import { Route, Routes } from "react-router-dom";
import ConnexionConfigForm from "./components/ConnexionConfigForm";
import Nav from "./components/Nav";
import Activites from "./pages/Activites";
import DetailActivite from "./pages/DetailActivite";
import DetailPersonne from "./pages/DetailPersonne";
import ListePersonnes from "./pages/ListePersonnes";
import ParametresPage from "./pages/ParametresPage";
import PlanningPage from "./pages/PlanningPage";
import type { Compatibilite, ConfigAffichee } from "./types";

function EcranVersionObsolete({ compat }: { compat: Compatibilite }) {
  return (
    <div className="min-h-screen bg-gray-50 flex items-center justify-center px-4">
      <div className="bg-white rounded-xl shadow-lg border border-red-200 p-8 w-full max-w-lg text-center">
        <h1 className="text-2xl font-bold text-red-700 mb-3">Version obsolète</h1>
        <p className="text-sm text-gray-700 leading-relaxed">
          Votre version de Cadence ({compat.version_installee}) est obsolète. Cette base de données a été mise à jour
          par une version plus récente de l'application. Mettez à jour Cadence pour continuer à l'utiliser.
        </p>
      </div>
    </div>
  );
}

function EcranPremierLancement({ onConfigChange }: { onConfigChange: () => void }) {
  const [config, setConfig] = useState<ConfigAffichee | null>(null);
  const [erreur, setErreur] = useState<string | null>(null);

  useEffect(() => {
    invoke<ConfigAffichee>("obtenir_config")
      .then(setConfig)
      .catch((e) => setErreur(typeof e === "string" ? e : "Impossible de lire la configuration."));
  }, []);

  if (erreur) {
    return <p className="text-red-600">{erreur}</p>;
  }

  return (
    <div className="min-h-screen bg-gray-50 flex items-center justify-center px-4">
      <div className="bg-white rounded-xl shadow-lg border border-gray-200 p-8 w-full max-w-lg">
        <h1 className="text-2xl font-bold text-gray-900 mb-2">Bienvenue dans Cadence</h1>
        <p className="text-sm text-gray-600 mb-6">
          Pour commencer, choisissez le mode de fonctionnement de l'application.
        </p>
        {config && (
          <ConnexionConfigForm config={config} onSauvegardee={onConfigChange} onRedemarrageDiffere={onConfigChange} />
        )}
      </div>
    </div>
  );
}

export default function App() {
  const [etat, setEtat] = useState<{ config: ConfigAffichee; compat: Compatibilite } | null>(null);

  const chargerEtat = useCallback(async () => {
    try {
      const [config, compat] = await Promise.all([
        invoke<ConfigAffichee>("obtenir_config"),
        invoke<Compatibilite>("obtenir_compatibilite"),
      ]);
      setEtat({ config, compat });
    } catch (e) {
      console.error(e);
    }
  }, []);

  useEffect(() => {
    chargerEtat();
  }, [chargerEtat]);

  if (!etat) {
    return <p className="min-h-screen bg-gray-50 flex items-center justify-center text-gray-500">Chargement...</p>;
  }

  if (!etat.compat.compatible) {
    return <EcranVersionObsolete compat={etat.compat} />;
  }

  if (!etat.config.configuree) {
    return <EcranPremierLancement onConfigChange={chargerEtat} />;
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <header className="bg-white shadow-sm border-b border-gray-200">
        <div className="max-w-5xl mx-auto px-4 py-3 flex items-center justify-between">
          <h1 className="text-xl font-semibold text-gray-800">Cadence</h1>
          <Nav />
        </div>
      </header>
      <main className="max-w-5xl mx-auto px-4 py-6">
        <Routes>
          <Route path="/" element={<ListePersonnes />} />
          <Route path="/personnes/:id" element={<DetailPersonne />} />
          <Route path="/activites" element={<Activites />} />
          <Route path="/activites/:id" element={<DetailActivite />} />
          <Route path="/planning" element={<PlanningPage />} />
          <Route path="/planning/:personneId" element={<PlanningPage />} />
          <Route path="/parametres" element={<ParametresPage />} />
        </Routes>
      </main>
    </div>
  );
}
