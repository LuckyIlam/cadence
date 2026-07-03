import { Link, useLocation } from "react-router-dom";

const links = [
  { to: "/", label: "Personnes" },
  { to: "/activites", label: "Activités" },
];

export default function Nav() {
  const location = useLocation();

  const estActif = (to: string) => {
    if (to === "/") return location.pathname === "/";
    return location.pathname.startsWith(to);
  };

  return (
    <nav className="flex gap-1">
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
    </nav>
  );
}
