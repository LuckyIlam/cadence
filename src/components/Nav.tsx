import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Link, useLocation } from "react-router-dom";
import { abonnerUtilisateur, invaliderUtilisateur } from "../utilisateur";

const links = [
  { to: "/", label: "Personnes" },
  { to: "/planning", label: "Planning" },
  { to: "/activites", label: "Activités" },
  { to: "/parametres", label: "Paramètres" },
];

export default function Nav() {
  const location = useLocation();
  const [utilisateur, setUtilisateur] = useState<string | null>(null);

  useEffect(() => {
    let actif = true;
    const rafraichir = () => {
      invaliderUtilisateur();
      invoke<{ utilisateur: string | null }>("obtenir_config")
        .then((c) => {
          if (actif) setUtilisateur(c.utilisateur ?? null);
        })
        .catch(() => {
          if (actif) setUtilisateur(null);
        });
    };
    rafraichir();
    const desabonner = abonnerUtilisateur(rafraichir);
    return () => {
      actif = false;
      desabonner();
    };
  }, []);

  const estActif = (to: string) => {
    if (to === "/") return location.pathname === "/";
    return location.pathname.startsWith(to);
  };

  return (
    <nav className="flex items-center gap-4">
      <div className="flex gap-1">
        {links.map((link) => (
          <Link
            key={link.to}
            to={link.to}
            className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
              estActif(link.to) ? "bg-blue-100 text-blue-700" : "text-gray-600 hover:text-gray-900 hover:bg-gray-100"
            }`}
          >
            {link.label}
          </Link>
        ))}
      </div>
      {utilisateur && <span className="text-sm text-gray-500 px-3 py-2 rounded-lg bg-gray-100">{utilisateur}</span>}
    </nav>
  );
}
