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
import type { ConfigAffichee } from "./types";

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
  const [config, setConfig] = useState<ConfigAffichee | null>(null);
  const [chargement, setChargement] = useState(true);

  const chargerConfig = useCallback(async () => {
    try {
      const c = await invoke<ConfigAffichee>("obtenir_config");
      setConfig(c);
    } catch (e) {
      setConfig(null);
      console.error(e);
    } finally {
      setChargement(false);
    }
  }, []);

  useEffect(() => {
    chargerConfig();
  }, [chargerConfig]);

  if (chargement) {
    return <p className="min-h-screen bg-gray-50 flex items-center justify-center text-gray-500">Chargement...</p>;
  }

  if (!config?.configuree) {
    return <EcranPremierLancement onConfigChange={chargerConfig} />;
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
