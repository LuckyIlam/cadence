import { invoke } from "@tauri-apps/api/core";
import { relaunch } from "@tauri-apps/plugin-process";
import { useState } from "react";
import { erreurMessage } from "../errors";
import type { ConfigAffichee, ModeConnexion, ResultatSauvegarde } from "../types";
import { invaliderUtilisateur } from "../utilisateur";

interface Props {
  config: ConfigAffichee;
  onSauvegardee?: (resultat: ResultatSauvegarde) => void;
  onRedemarrageDiffere?: (resultat: ResultatSauvegarde) => void;
}

export default function ConnexionConfigForm({ config, onSauvegardee, onRedemarrageDiffere }: Props) {
  const [mode, setMode] = useState<ModeConnexion>(config.mode ?? "mono");
  const [url, setUrl] = useState(config.url ?? "");
  const [token, setToken] = useState("");
  const [utilisateur, setUtilisateur] = useState(config.utilisateur ?? "");
  const [saving, setSaving] = useState(false);
  const [testing, setTesting] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [erreur, setErreur] = useState<string | null>(null);
  const [testReussi, setTestReussi] = useState<boolean | null>(null);
  const [attenteRedemarrage, setAttenteRedemarrage] = useState<ResultatSauvegarde | null>(null);

  const testerConnexion = async () => {
    setTesting(true);
    setTestReussi(null);
    setMessage(null);
    setErreur(null);
    try {
      await invoke("tester_connexion", { url, token });
      setTestReussi(true);
    } catch (e) {
      setTestReussi(false);
      setErreur(erreurMessage(e));
    } finally {
      setTesting(false);
    }
  };

  const sauvegarder = async () => {
    setSaving(true);
    setMessage(null);
    setErreur(null);
    try {
      const resultat = await invoke<ResultatSauvegarde>("sauvegarder_config", {
        mode,
        url: mode === "mono" ? null : url,
        token: token.trim() === "" ? null : token,
        utilisateur,
      });
      invaliderUtilisateur();
      if (resultat.redemarrage_requis) {
        setAttenteRedemarrage(resultat);
      } else {
        setMessage("Configuration enregistrée.");
        onSauvegardee?.(resultat);
      }
    } catch (e) {
      setErreur(erreurMessage(e));
    } finally {
      setSaving(false);
    }
  };

  const redemarrer = async () => {
    try {
      await relaunch();
    } catch (e) {
      setAttenteRedemarrage(null);
      setErreur(erreurMessage(e));
    }
  };

  const reporter = () => {
    const resultat = attenteRedemarrage;
    setAttenteRedemarrage(null);
    if (resultat) {
      setMessage("Le changement sera appliqué au prochain démarrage de l'application.");
      onRedemarrageDiffere?.(resultat);
    }
  };

  const champ =
    "w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500";

  return (
    <div>
      <div className="grid grid-cols-2 gap-4 mb-4">
        <button
          type="button"
          onClick={() => setMode("mono")}
          className={`px-4 py-3 rounded-lg border text-sm font-medium transition-colors text-left ${
            mode === "mono"
              ? "border-blue-600 bg-blue-50 text-blue-700"
              : "border-gray-300 bg-white text-gray-600 hover:bg-gray-50"
          }`}
        >
          <span className="block font-semibold">Mono-utilisateur</span>
          <span className="block text-xs text-gray-500 mt-1">Base locale sur cet ordinateur</span>
        </button>
        <button
          type="button"
          onClick={() => setMode("multi")}
          className={`px-4 py-3 rounded-lg border text-sm font-medium transition-colors text-left ${
            mode === "multi"
              ? "border-blue-600 bg-blue-50 text-blue-700"
              : "border-gray-300 bg-white text-gray-600 hover:bg-gray-50"
          }`}
        >
          <span className="block font-semibold">Multi-utilisateurs</span>
          <span className="block text-xs text-gray-500 mt-1">Base partagée (Turso)</span>
        </button>
      </div>

      <div className="space-y-4">
        {mode === "multi" && (
          <>
            <div>
              <label htmlFor="config-url" className="block text-sm font-medium text-gray-700 mb-1">
                URL de la base
              </label>
              <input
                id="config-url"
                type="text"
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                placeholder="libsql://..."
                className={champ}
              />
            </div>
            <div>
              <label htmlFor="config-token" className="block text-sm font-medium text-gray-700 mb-1">
                Clé d'accès
              </label>
              <input
                id="config-token"
                type="password"
                value={token}
                onChange={(e) => setToken(e.target.value)}
                placeholder={config.a_une_cle ? "Laisser vide pour conserver la clé existante" : "Jeton d'accès"}
                className={champ}
              />
            </div>
          </>
        )}
        <div>
          <label htmlFor="config-utilisateur" className="block text-sm font-medium text-gray-700 mb-1">
            Nom d'utilisateur
          </label>
          <input
            id="config-utilisateur"
            type="text"
            value={utilisateur}
            onChange={(e) => setUtilisateur(e.target.value)}
            placeholder="Votre nom"
            className={champ}
          />
        </div>
      </div>

      {message && (
        <div className="bg-green-100 border border-green-300 text-green-700 px-3 py-2 rounded-lg mt-4 text-sm">
          {message}
        </div>
      )}
      {erreur && (
        <div className="bg-red-100 border border-red-300 text-red-700 px-3 py-2 rounded-lg mt-4 text-sm">{erreur}</div>
      )}
      {testReussi === true && (
        <div className="bg-green-100 border border-green-300 text-green-700 px-3 py-2 rounded-lg mt-4 text-sm">
          Connexion établie avec la base distante.
        </div>
      )}

      <div className="flex items-center gap-3 mt-6">
        {mode === "multi" && (
          <button
            type="button"
            onClick={testerConnexion}
            disabled={testing || saving}
            className="px-4 py-2 text-sm bg-gray-200 text-gray-800 rounded hover:bg-gray-300 transition-colors disabled:opacity-50"
          >
            {testing ? "Test en cours..." : "Tester la connexion"}
          </button>
        )}
        <button
          type="button"
          onClick={sauvegarder}
          disabled={saving}
          className="px-4 py-2 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors disabled:opacity-50"
        >
          {saving ? "Enregistrement..." : "Enregistrer"}
        </button>
      </div>

      {attenteRedemarrage && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white rounded-xl shadow-xl p-6 w-full max-w-md mx-4">
            <h3 className="text-lg font-semibold text-gray-900 mb-2">Redémarrage requis</h3>
            <p className="text-sm text-gray-600 mb-6">
              Le changement de mode ou de connexion sera appliqué au prochain démarrage de l'application. Redémarrer
              maintenant ?
            </p>
            <div className="flex justify-end gap-3">
              <button
                type="button"
                onClick={reporter}
                className="px-4 py-2 text-sm bg-gray-200 text-gray-800 rounded hover:bg-gray-300 transition-colors"
              >
                Plus tard
              </button>
              <button
                type="button"
                onClick={redemarrer}
                className="px-4 py-2 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
              >
                Redémarrer maintenant
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
